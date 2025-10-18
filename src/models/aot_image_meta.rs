use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct AotImageMeta {
    pub day: u32,
    pub taken_at: DateTime<Utc>,
    pub location: Option<String>,
}
