use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track: Option<u32>,
    pub disc: Option<u32>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub duration: Option<f64>,
    pub ext: String,
    pub name: String,
    pub stem: String,
}

impl Serialize for AudioMetadata {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry(
            "artist",
            &self.artist.as_ref().map_or("unknown", |s| s.as_str()),
        )?;
        map.serialize_entry(
            "title",
            &self.title.as_ref().map_or("unknown", |s| s.as_str()),
        )?;
        map.serialize_entry(
            "album",
            &self.album.as_ref().map_or("unknown", |s| s.as_str()),
        )?;
        map.serialize_entry("track", &self.track.unwrap_or(0))?;
        map.serialize_entry("disc", &self.disc.unwrap_or(0))?;
        map.serialize_entry(
            "genre",
            &self.genre.as_ref().map_or("unknown", |s| s.as_str()),
        )?;
        map.serialize_entry(
            "date",
            &self.date.as_ref().map_or("unknown", |s| s.as_str()),
        )?;
        map.serialize_entry("duration", &self.duration.unwrap_or(0.0))?;
        map.serialize_entry("ext", &self.ext)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("stem", &self.stem)?;
        map.end()
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AudioFile {
    pub path: PathBuf,
    pub metadata: AudioMetadata,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RenameReport {
    pub success: usize,
    pub failed: usize,
    pub dry_run: bool,
    pub errors: Vec<(PathBuf, String)>,
    pub moves: Vec<(PathBuf, PathBuf)>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Transcript {
    pub text: String,
    pub segments: Vec<(f64, f64, String)>,
    pub language: Option<String>,
}
