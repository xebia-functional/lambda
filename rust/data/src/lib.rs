use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, Send)]
pub struct Datum {
	pub uuid: Uuid,
	pub doc: String,
	pub hashes: u16,
	pub hash: Option<String>,
}
