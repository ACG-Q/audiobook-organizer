use serde::Serialize;

pub trait Emit {
    fn emit(&self);
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum StreamEvent<T: Serialize = serde_json::Value> {
    #[serde(rename = "start")]
    Start { data: T },
    #[serde(rename = "progress")]
    Progress { current: usize, total: usize },
    #[serde(rename = "item")]
    Item { data: T },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "done")]
    Done { summary: T },
}

impl<T: Serialize> Emit for StreamEvent<T> {
    fn emit(&self) {
        println!("{}", serde_json::to_string(self).unwrap());
    }
}

impl<T: Serialize> StreamEvent<T> {
    pub fn emit_progress(current: usize, total: usize) {
        StreamEvent::<serde_json::Value>::Progress { current, total }.emit();
    }
}
