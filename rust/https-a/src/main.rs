use std::str::FromStr;

use aws_sdk_dynamodb::primitives::Blob;
use data::Datum;

use aws_sdk_kinesis::{Client, types::builders::PutRecordsRequestEntryBuilder};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

////////////////////////////////////////////////////////////////////////////////
///                               Entry point.                               ///
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Error> {
	// Initialize the logger. Disable per-line module name and time printing,
	// since CloudWatch will take care of this.
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.with_target(false)
		.without_time()
		.init();

	// Create the Kinesis client using configuration data supplied through the
	// environment.
	let config = aws_config::load_from_env().await;
	let kinesis = Client::new(&config);

	run(service_fn(|event: Request| async {
		handle_request(&kinesis, event).await
	}))
	.await
}

////////////////////////////////////////////////////////////////////////////////
///                                Endpoints.                                ///
////////////////////////////////////////////////////////////////////////////////

/// Process an incoming web [request](Request) by generating a batch of random
/// messages and posting them to Kinesis. The noteworthy query parameters are:
/// - `chars`: The number of random characters to generate per message.
/// - `hashes`: The number of hash iterations to perform per message.
/// - `msgs`: The number of messages to post to Kinesis.
async fn handle_request(kinesis: &Client, event: Request) -> Result<Response<Body>, Error> {
	// Extract the query parameters from the request.
	let chars = param_or_default(&event, LENGTH_PARAM, 1024usize);
	let hashes = param_or_default(&event, HASHES_PARAM, 100u16);
	let messages = param_or_default(&event, MESSAGES_PARAM, 64usize);

	// Produce the requested number of random messages.
	let mut batch = Vec::with_capacity(messages);
	for _ in 0..messages {
		batch.push(Datum::random(chars, hashes));
	}

	// Post the messages to Kinesis.
	let succeeded = post_data(kinesis, batch).await?;

	// Respond with a simple affirmation.
	let resp = Response::builder()
		.status(200)
		.header("content-type", "text/html")
		.body(format!("{} messages posted to Kinesis.", succeeded).into())
		.map_err(Box::new)?;
	Ok(resp)
}

////////////////////////////////////////////////////////////////////////////////
///                                Utilities.                                ///
////////////////////////////////////////////////////////////////////////////////

/// Read the first occurrence of the named query parameter from the
/// [request](Request), returning a default value if it is not present or cannot
/// be parsed as a `T`.
#[must_use]
fn param_or_default<T: FromStr>(event: &Request, name: &str, default: T) -> T {
	event
		.query_string_parameters_ref()
		.and_then(|params| params.first(name))
		.and_then(|c| c.parse::<T>().ok())
		.unwrap_or(default)
}

// Post the given batch of messages to the Kinesis stream designated by the
// environment.
async fn post_data(
	kinesis: &Client,
	batch: impl IntoIterator<Item = Datum>
) -> Result<usize, Error> {
	let write = std::env::var(WRITE_STREAM)?;
	let entries = batch
		.into_iter()
		.flat_map(|datum| {
			let s = datum.to_json().unwrap();
			let blob = Blob::new(s.into_bytes());
			PutRecordsRequestEntryBuilder::default()
				.data(blob)
				.partition_key(datum.uuid.to_string())
				.build()
		})
		.collect();
	let output = kinesis
		.put_records()
		.stream_name(write)
		.set_records(Some(entries))
		.send()
		.await?;
	let failed = output.failed_record_count.unwrap_or_default();
	let succeeded = output.records().len() - failed as usize;
	Ok(succeeded)
}

////////////////////////////////////////////////////////////////////////////////
///                                Constants.                                ///
////////////////////////////////////////////////////////////////////////////////

/// The name of the query parameter that specifies the number of random
/// characters to generate.
const LENGTH_PARAM: &str = "chars";

/// The name of the query parameter that specifies the number of hash iterations
/// to perform.
const HASHES_PARAM: &str = "hashes";

/// The name of the query parameter that specifies the number of messages to
/// post to Kinesis.
const MESSAGES_PARAM: &str = "msgs";

/// The name of the environment variable that specifies the name of the Kinesis
/// stream to which messages should be posted. This environment exists in the
/// Lambda execution environment, not in the local development environment.
const WRITE_STREAM: &str = "KINESIS_STREAM_A";
