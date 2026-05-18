use std::path::PathBuf;

#[test]
fn test_parse_wav_metadata() {
    let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.wav"));
    let meta = audiobook_scanner::read_metadata(&path).unwrap();
    assert_eq!(meta.ext, "wav");
    assert_eq!(meta.name, "sample");
    assert_eq!(meta.stem, "sample");
}

#[test]
fn test_parse_missing_fields() {
    let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/empty.wav"));
    let meta = audiobook_scanner::read_metadata(&path).unwrap();
    assert_eq!(meta.ext, "wav");
    assert_eq!(meta.name, "empty");
    assert_eq!(meta.stem, "empty");
}

#[test]
fn test_parse_nonexistent_file() {
    let path = PathBuf::from("tests/fixtures/nonexistent.mp3");
    let err = audiobook_scanner::read_metadata(&path).unwrap_err();
    assert!(err.to_string().contains("No such file") || err.to_string().contains("系统找不到"));
}
