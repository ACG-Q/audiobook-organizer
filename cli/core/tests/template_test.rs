use audiobook_organizer_core::AudioMetadata;
use audiobook_organizer_core::render;

#[test]
fn test_basic_template_render() {
    let meta = AudioMetadata {
        artist: Some("Author".into()),
        title: Some("Book Title".into()),
        track: Some(5),
        ext: "mp3".into(),
        ..Default::default()
    };
    let result = render("{{artist}}/{{title}}.{{ext}}", &meta).unwrap();
    assert_eq!(result, "Author/Book Title.mp3");
}

#[test]
fn test_padding_format() {
    let meta = AudioMetadata { track: Some(3), ..Default::default() };
    let result = render("{{format track \"02\"}} - {{title}}", &meta).unwrap();
    assert_eq!(result, "03 - unknown");
}

#[test]
fn test_missing_field_uses_default() {
    let meta = AudioMetadata::default();
    let result = render("{{artist}}/{{title}}", &meta).unwrap();
    assert_eq!(result, "unknown/unknown");
}
