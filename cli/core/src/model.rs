use std::path::PathBuf;

fn model_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("audiobook-organizer/models")
}

pub fn list_models() -> anyhow::Result<Vec<String>> {
    let dir = model_dir();
    let mut models = Vec::new();
    if dir.is_dir() {
        for e in std::fs::read_dir(dir)?.flatten() {
            if e.file_type()?.is_file() {
                models.push(e.file_name().into_string().unwrap_or_default());
            }
        }
    }
    Ok(models)
}

pub fn model_path(name: &str) -> PathBuf {
    model_dir().join(name)
}

pub fn download_model(name: &str) -> anyhow::Result<()> {
    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        name
    );
    let dest = model_path(name);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let resp = ureq::get(&url).call()?;
    let mut out = std::fs::File::create(dest)?;
    std::io::copy(&mut resp.into_reader(), &mut out)?;
    Ok(())
}
