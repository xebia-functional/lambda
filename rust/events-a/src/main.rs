use aws_lambda_events::event::kinesis::KinesisEvent;
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_kinesis::{types::builders::PutRecordsRequestEntryBuilder, Client};
use base64::{Engine, engine::general_purpose::STANDARD};
use data::Datum;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};


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

	run(service_fn(|event: LambdaEvent<KinesisEvent>| async {
		handle_request(&kinesis, event).await
	}))
	.await
}

////////////////////////////////////////////////////////////////////////////////
///                                Endpoints.                                ///
////////////////////////////////////////////////////////////////////////////////

/// Process an incoming Kinetic [event](KinesisEvent) by computing the hash for
/// each message and then re-posting the augmented messages to another Kinesis
/// stream. Incoming messages are JSON serializations of [`Data`](Datum).
async fn handle_request(
	kinesis: &Client,
	event: LambdaEvent<KinesisEvent>
) -> Result<(), Error> {
	let write = std::env::var(WRITE_STREAM)?;
	let mut entries = vec![];
	event.payload.records.iter().for_each(|record| {
		let data = STANDARD.decode(&record.kinesis.data.0).unwrap();
		let data = String::from_utf8(data).unwrap();
		let mut data: Datum = serde_json::from_str(&data).unwrap();
		data.hash();
		let data = data.to_json().unwrap();
		let blob = Blob::new(data.into_bytes());
		let entry = PutRecordsRequestEntryBuilder::default()
			.data(blob)
			.partition_key(record.kinesis.partition_key.clone().unwrap())
			.build()
			.unwrap();
		entries.push(entry);
	});
	kinesis
		.put_records()
		.stream_name(write)
		.set_records(Some(entries))
		.send()
		.await?;
	Ok(())
}

////////////////////////////////////////////////////////////////////////////////
///                                Constants.                                ///
////////////////////////////////////////////////////////////////////////////////

/// The name of the environment variable that specifies the name of the Kinesis
/// stream to which messages should be posted. This environment exists in the
/// Lambda execution environment, not in the local development environment.
const WRITE_STREAM: &str = "KINESIS_STREAM_B";
