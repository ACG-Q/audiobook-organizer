use std::io::BufWriter;
use std::path::Path;

use super::AudioEncoder;

pub struct WavEncoder {
    path: std::path::PathBuf,
    writer: Option<hound::WavWriter<BufWriter<std::fs::File>>>,
}

impl WavEncoder {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        Ok(WavEncoder {
            path: path.to_path_buf(),
            writer: None,
        })
    }
}

impl AudioEncoder for WavEncoder {
    fn write_header(&mut self, sample_rate: u32, channels: u16) -> anyhow::Result<()> {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let writer = hound::WavWriter::create(&self.path, spec)?;
        self.writer = Some(writer);
        Ok(())
    }

    fn encode_chunk(&mut self, samples: &[f32]) -> anyhow::Result<()> {
        let writer = self.writer.as_mut().unwrap();
        for &s in samples {
            let sample = (s * i16::MAX as f32).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(sample)?;
        }
        Ok(())
    }

    fn finalize(&mut self) -> anyhow::Result<()> {
        if let Some(writer) = self.writer.take() {
            writer.finalize()?;
        }
        Ok(())
    }
}
