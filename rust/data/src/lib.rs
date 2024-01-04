use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Datum {
    pub uuid: Uuid,
    pub doc: String,
    pub hashes: u16,
    pub hash: Option<String>,
}
