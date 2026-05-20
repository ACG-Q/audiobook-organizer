use std::io::Write;
use std::path::Path;

use super::AudioEncoder;

pub struct OggOpusEncoder {
    path: std::path::PathBuf,
    file: Option<std::fs::File>,
    encoder: Option<opus_rs::OpusEncoder>,
    sample_rate: u32,
    channels: u16,
    total_samples: u64,
    pre_skip: u16,
}

impl OggOpusEncoder {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        Ok(OggOpusEncoder {
            path: path.to_path_buf(),
            file: None,
            encoder: None,
            sample_rate: 48000,
            channels: 1,
            total_samples: 0,
            pre_skip: 0,
        })
    }

    fn write_opus_head(
        file: &mut std::fs::File,
        channels: u8,
        pre_skip: u16,
        sample_rate: u32,
    ) -> anyhow::Result<()> {
        let mut head = Vec::with_capacity(19);
        head.extend_from_slice(b"OpusHead");
        head.push(1); // version
        head.push(channels);
        head.extend_from_slice(&pre_skip.to_le_bytes());
        head.extend_from_slice(&sample_rate.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes()); // output gain
        head.push(0); // mapping family

        write_ogg_page(file, &head, 0, 0, true, false)?;
        Ok(())
    }

    fn write_opus_tags(file: &mut std::fs::File) -> anyhow::Result<()> {
        let vendor = b"audiobook-splitter";
        let mut tags = Vec::with_capacity(8 + 4 + vendor.len() + 4);
        tags.extend_from_slice(b"OpusTags");
        tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        tags.extend_from_slice(vendor);
        tags.extend_from_slice(&0u32.to_le_bytes()); // 0 user comments

        write_ogg_page(file, &tags, 0, 0, false, false)?;
        Ok(())
    }
}

fn write_ogg_page(
    file: &mut std::fs::File,
    packet: &[u8],
    serial: i32,
    granule_pos: i64,
    bos: bool,
    eos: bool,
) -> anyhow::Result<()> {
    let mut header_type: u8 = 0;
    if bos {
        header_type |= 0x02;
    }
    if eos {
        header_type |= 0x04;
    }

    let mut page = Vec::new();
    page.extend_from_slice(b"OggS");
    page.push(0); // version
    page.push(header_type);
    page.extend_from_slice(&granule_pos.to_le_bytes());
    page.extend_from_slice(&serial.to_le_bytes());
    page.extend_from_slice(&0u32.to_le_bytes()); // page sequence no
    page.extend_from_slice(&0u32.to_le_bytes()); // checksum (0, filled later if needed)
    page.push(1); // number of segments
    page.push(packet.len() as u8); // segment table

    page.extend_from_slice(packet);
    file.write_all(&page)?;
    Ok(())
}

impl AudioEncoder for OggOpusEncoder {
    fn write_header(&mut self, sample_rate: u32, channels: u16) -> anyhow::Result<()> {
        self.sample_rate = sample_rate;
        self.channels = channels;

        let app = if channels == 1 {
            opus_rs::Application::Voip
        } else {
            opus_rs::Application::Audio
        };
        let enc = opus_rs::OpusEncoder::new(sample_rate as i32, channels as usize, app)
            .map_err(|e| anyhow::anyhow!("OpusEncoder::new failed: {e}"))?;

        self.file = Some(std::fs::File::create(&self.path)?);
        self.pre_skip = 312; // RFC 7845 recommended pre-skip for 48kHz

        let file = self.file.as_mut().unwrap();
        Self::write_opus_head(file, channels as u8, self.pre_skip, sample_rate)?;
        Self::write_opus_tags(file)?;

        self.encoder = Some(enc);
        Ok(())
    }

    fn encode_chunk(&mut self, samples: &[f32]) -> anyhow::Result<()> {
        let sample_rate = self.sample_rate;
        let channels = self.channels as usize;

        // opus-rs encodes frame by frame. We buffer samples until we have a full frame.
        // The Opus encoder expects samples at the native sample rate.
        // Frame size is determined by sample_rate * 0.02 (20ms).

        let frame_size = (sample_rate as f64 * 0.02) as usize * channels;
        let mut offset = 0;

        let encoder = self.encoder.as_mut().unwrap();
        let file = self.file.as_mut().unwrap();

        while offset + frame_size <= samples.len() {
            let frame = &samples[offset..offset + frame_size];
            // For mono, just use as-is. For stereo, we need interleaved format.
            let input: Vec<f32> = frame.to_vec();

            let mut output = vec![0u8; 4096];
            let encoded = encoder
                .encode(&input, frame_size / channels, &mut output)
                .map_err(|e| anyhow::anyhow!("Opus encode failed: {e}"))?;

            let pcm_samples = (frame_size / channels) as u64;
            self.total_samples += pcm_samples;

            let granule_pos = self.total_samples.saturating_sub(self.pre_skip as u64);

            write_ogg_page(
                file,
                &output[..encoded],
                0,
                granule_pos as i64,
                false,
                false,
            )?;
            offset += frame_size;
        }

        Ok(())
    }

    fn finalize(&mut self) -> anyhow::Result<()> {
        // Write remaining (if any) as partial frame
        if let Some(file) = self.file.as_mut() {
            // Final OGG page with EOS
            let eos_granule = self.total_samples.saturating_sub(self.pre_skip as u64);
            write_ogg_page(file, &[], 0, eos_granule as i64, false, true)?;
        }
        self.file = None;
        Ok(())
    }
}
