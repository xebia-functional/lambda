use data::Datum;

use aws_sdk_dynamodb::Client;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
// use serde::{Deserialize, Serialize};
use serde_dynamo::to_attribute_value;
use sha3::{Digest, Sha3_512};
use tracing::info;
use uuid::Uuid;
// use serde_json;

fn read_param<T: std::str::FromStr>(event: &Request, name: &str, dflt: T) -> T {
	event
		.query_string_parameters_ref()
		.and_then(|params| params.first(name))
		.and_then(|c| c.parse::<T>().ok())
		.unwrap_or(dflt)
}

async fn handle_request(db_client: &Client, event: Request) -> Result<Response<Body>, Error> {
	let chars = read_param(&event, "chars", 1024usize);
	let hashes = read_param(&event, "hashes", 100u16);
	let msgs = read_param(&event, "msgs", 64usize);

	// info!(payload = %s, "JSON Payload received");

	// (0..msgs).

	let mut rng = thread_rng();
	let doc: String = (&mut rng)
		.sample_iter(Alphanumeric)
		.take(chars)
		.map(char::from)
		.collect();

	let hash = Some((0..hashes).into_iter().fold(doc.clone(), |a, _| {
		Sha3_512::digest(a)
			.iter()
			.map(|b| format!("{b:02X}"))
			.collect()
	}));

	let datum = Datum {
		uuid: Uuid::new_v4(),
		doc,
		hashes,
		hash,
	};

	let msg = serde_json::to_string(&datum).unwrap();

	// add_datum(db_client, datum.clone(), "Datum").await?;

	// Return something that implements IntoResponse.
	// It will be serialized to the right response event automatically by the runtime
	let resp = Response::builder()
		.status(200)
		.header("content-type", "text/html")
		.body(msg.into())
		.map_err(Box::new)?;
	Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		// disable printing the name of the module in every log line.
		.with_target(false)
		// disabling time is handy because CloudWatch will add the ingestion time.
		.without_time()
		.init();

	//Get config from environment.
	let config = aws_config::load_from_env().await;
	//Create the DynamoDB client.
	let client = Client::new(&config);

	run(service_fn(|event: Request| async {
		handle_request(&client, event).await
	}))
	.await
}

// TODO: Add a datum to a table.
pub async fn add_datum(client: &Client, d: Datum, table: &str) -> Result<(), Error> {
	// let uuid_av = to_attribute_value(d.uuid)?;
	// let doc_av = to_attribute_value(d.doc)?;
	// let hashes_av = to_attribute_value(d.hashes)?;
	// let hash_av = to_attribute_value(d.hash)?;

	// let request = client
	// .put_item()
	// .table_name(table)
	// .item("uuid", uuid_av)
	// .item("doc", doc_av)
	// .item("hashes", hashes_av)
	// .item("hash", hash_av);

	// info!("adding item to DynamoDB");

	// let _resp = request.send().await?;

	Ok(())
}
