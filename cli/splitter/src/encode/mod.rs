use std::path::Path;

pub mod flac;
pub mod ogg;
pub mod wav;

pub trait AudioEncoder {
    fn write_header(&mut self, sample_rate: u32, channels: u16) -> anyhow::Result<()>;
    fn encode_chunk(&mut self, samples: &[f32]) -> anyhow::Result<()>;
    fn finalize(&mut self) -> anyhow::Result<()>;
}

pub fn create_encoder(path: &Path, format: &str) -> anyhow::Result<Box<dyn AudioEncoder>> {
    match format {
        "wav" => Ok(Box::new(wav::WavEncoder::new(path)?)),
        "flac" => Ok(Box::new(flac::FlacEncoder::new(path)?)),
        "ogg" => Ok(Box::new(ogg::OggOpusEncoder::new(path)?)),
        other => Err(anyhow::anyhow!(
            "不支持的编码格式: \"{other}\"\n支持的格式: wav, flac, ogg\n\n注意: mp3 和 m4a(aac) 编码暂无纯 Rust 实现，暂不支持。"
        )),
    }
}
