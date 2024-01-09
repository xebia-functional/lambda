use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_512};
use std::fmt::Write;
use uuid::Uuid;

/// `Datum` represents an arbitrary document. Instances are generated
/// pseudo-randomly by the `httpd-a` service and placed into Kinesis for
/// consumption by the `events-a` service. `events-a` compute an iterative
/// SHA-512 hash of the document prior to re-injecting it into Kinesis. Finally,
/// `events-b` consumes the document and stores it in DynamoDB.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Datum {
	/// The unique identifier for this document, used as a partition key in
	/// DynamoDB.
	pub uuid: Uuid,

	/// The pseudo-randomly generated document, produced by `httpd-a`.
	pub doc: String,

	/// The target iteration count for the SHA-512 hash.
	pub hashes: u16,

	/// The SHA-512 hash of the document, computed by `events-a`.
	pub hash: Option<String>,
}

////////////////////////////////////////////////////////////////////////////////
///                               Generation.                                ///
////////////////////////////////////////////////////////////////////////////////

impl Datum {
	/// Generate a new `Datum` with the given number of random characters and
	/// target number of hash iterations.
	pub fn random(chars: usize, hashes: u16) -> Self {
		Self {
			uuid: Uuid::new_v4(),
			doc: {
				let mut rng = thread_rng();
				(&mut rng)
					.sample_iter(Alphanumeric)
					.take(chars)
					.map(char::from)
					.collect()
			},
			hashes,
			hash: None,
		}
	}
}

////////////////////////////////////////////////////////////////////////////////
///                                 Hashing.                                 ///
////////////////////////////////////////////////////////////////////////////////

impl Datum {
	/// Lazily compute the iterative SHA-512 hash of this [`Datum`]'s document,
	/// storing it within as necessary.
	pub fn hash(&mut self) {
		match self.hash {
			Some(_) => (),
			None => {
				self.hash = Some((0..self.hashes).fold(self.doc.clone(), |a, _| {
					Sha3_512::digest(a)
						.iter()
						.fold(
							String::new(),
							|mut a, b| {
								write!(&mut a, "{b:02X}").unwrap();
								a
							}
						)
				}));
			}
		}
	}
}

////////////////////////////////////////////////////////////////////////////////
///                           JSON serialization.                            ///
////////////////////////////////////////////////////////////////////////////////

impl Datum {
	/// Serialize this [`Datum`] as a JSON string.
	#[inline]
	pub fn to_json(&self) -> Result<String, serde_json::Error> {
		serde_json::to_string(self)
	}

	/// Deserialize a [`Datum`] from the supplied JSON string.
	#[inline]
	pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
		serde_json::from_str(s)
	}
}
