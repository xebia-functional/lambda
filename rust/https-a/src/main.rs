use data::Datum;

// use aws_sdk_dynamodb::Client; //, Error};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json;
use sha3::{Digest, Sha3_512};
use uuid::Uuid;

fn read_param<T: std::str::FromStr>(event: &Request, name: &str, dflt: T) -> T {
	event
		.query_string_parameters_ref()
		.and_then(|params| params.first(name))
		.and_then(|c| c.parse::<T>().ok())
		.unwrap_or(dflt)
}

/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
	// let shared_config = aws_config::load_from_env().await;
	// let client = Client::new(&shared_config);
	// let req = client.list_tables().limit(10);
	// let resp = req.send().await?;
	// println!("Current DynamoDB tables: {:?}", resp.table_names);
	// let message = format!("Current DynamoDB tables: {:?}", resp.table_names);

	// Extract some useful information from the request
	//
	//
	let chars = read_param(&event, "chars", 1024usize);
	let hashes = read_param(&event, "hashes", 100u16);
	let msgs = read_param(&event, "msgs", 64usize);

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

	let msg = serde_json::to_string(&Datum {
		uuid: Uuid::new_v4(),
		doc,
		hashes,
		hash,
	})
	.unwrap();

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

	run(service_fn(function_handler)).await
}
