use std::path::Path;

use super::AudioEncoder;

pub struct FlacEncoder {
    path: std::path::PathBuf,
    writer: Option<flac_codec::encode::FlacSampleWriter<std::fs::File>>,
}

impl FlacEncoder {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        Ok(FlacEncoder {
            path: path.to_path_buf(),
            writer: None,
        })
    }
}

impl AudioEncoder for FlacEncoder {
    fn write_header(&mut self, sample_rate: u32, channels: u16) -> anyhow::Result<()> {
        let file = std::fs::File::create(&self.path)?;
        let writer = flac_codec::encode::FlacSampleWriter::new(
            file,
            flac_codec::encode::Options::default(),
            sample_rate,
            16,
            channels as u8,
            None,
        )?;
        self.writer = Some(writer);
        Ok(())
    }

    fn encode_chunk(&mut self, samples: &[f32]) -> anyhow::Result<()> {
        let writer = self.writer.as_mut().unwrap();
        let int_samples: Vec<i32> = samples
            .iter()
            .map(|&s| (s * i32::MAX as f32).clamp(-2147483648.0, 2147483647.0) as i32)
            .collect();
        writer.write(&int_samples)?;
        Ok(())
    }

    fn finalize(&mut self) -> anyhow::Result<()> {
        if let Some(writer) = self.writer.take() {
            writer.finalize()?;
        }
        Ok(())
    }
}
