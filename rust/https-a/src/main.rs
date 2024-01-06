use std::str::FromStr;

use data::Datum;

use aws_sdk_kinesis::Client;
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

	// Create the DynamoDB client using configuration data supplied through the
	// environment.
	let config = aws_config::load_from_env().await;
	let client = Client::new(&config);

	run(service_fn(|event: Request| async {
		handle_request(&client, event).await
	}))
	.await
}

////////////////////////////////////////////////////////////////////////////////
///                                Endpoints.                                ///
////////////////////////////////////////////////////////////////////////////////

async fn handle_request(
	kinesis: &Client,
	event: Request
) -> Result<Response<Body>, Error> {
	// Extract the query parameters from the request.
	let chars = param_or_default(&event, LENGTH_PARAM, 1024usize);
	let hashes = param_or_default(&event, HASHES_PARAM, 100u16);
	let messages = param_or_default(&event, MESSAGES_PARAM, 64usize);

	// Produce the requested number of random messages.
	let mut batch = Vec::with_capacity(messages);
	for _ in 0 .. messages {
		batch.push(Datum::random(chars, hashes).to_json()?);
	}

	// Post the messages to Kinesis.
	post_data(kinesis, batch).await?;

	// Respond with a simple affirmation.
	let resp = Response::builder()
		.status(200)
		.header("content-type", "text/html")
		.body(format!("{} messages posted to Kinesis.", messages).into())
		.map_err(Box::new)?;
	Ok(resp)
}

////////////////////////////////////////////////////////////////////////////////
///                                Utilities.                                ///
////////////////////////////////////////////////////////////////////////////////

/// Read the first occurrence of the named query parameter from the
/// [request](Request), returning a default value if it is not present or cannot
/// be parsed as a `T`.
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
	client: &Client,
	batch: impl IntoIterator<Item = String>
) -> Result<(), Error> {
	// TODO: Implement this.
	Ok(())
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
