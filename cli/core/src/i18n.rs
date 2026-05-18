pub enum Lang {
    Zh,
    En,
}

pub fn detect_lang() -> Lang {
    match std::env::var("AUDIOBOOK_LANG").as_deref() {
        Ok("en") | Ok("EN") | Ok("en_US") | Ok("en-US") => Lang::En,
        _ => Lang::Zh,
    }
}
