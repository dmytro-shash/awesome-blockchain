use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Response {
    is_ok: bool,
    content: String,
}

impl Response {
    pub fn new(is_ok: bool,
               content: String) -> Self {
        Response {
            is_ok,
            content,
        }
    }
}