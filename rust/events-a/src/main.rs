use aws_lambda_events::event::kinesis::KinesisEvent;
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_kinesis::{types::builders::PutRecordsRequestEntryBuilder, Client};
use data::Datum;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use tracing::{debug, error, info, trace};

////////////////////////////////////////////////////////////////////////////////
///                               Entry point.                               ///
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Error> {
	// Initialize the logger. Disable per-line module name and time printing,
	// since CloudWatch will take care of this.
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::WARN)
		.with_target(false)
		.without_time()
		.init();

	// Create the Kinesis client using configuration data supplied through the
	// environment.
	let config = aws_config::load_from_env().await;
	info!("Loaded configuration: {:?}", config);
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
async fn handle_request(kinesis: &Client, event: LambdaEvent<KinesisEvent>) -> Result<(), Error> {
	debug!("Received event: {:?}", event);
	let write = std::env::var(WRITE_STREAM)?;
	debug!("Posting messages to Kinesis stream: {}", write);
	let mut entries = vec![];
	event.payload.records.iter().for_each(|record| {
		trace!("Incoming record: {:?}", record);
		let data = String::from_utf8_lossy(&record.kinesis.data.0);
		trace!("JSON: {:?}", data);
		let mut data: Datum = match serde_json::from_str(&data) {
			Ok(data) => data,
			Err(e) => {
				error!("Failed to deserialize datum: {e}");
				return;
			}
		};
		trace!("Deserialized datum: {:?}", data);
		data.hash();
		trace!("Outgoing datum: {:?}", data);
		let data = data.to_json().unwrap();
		let blob = Blob::new(data.into_bytes());
		let entry = PutRecordsRequestEntryBuilder::default()
			.data(blob)
			.partition_key(record.kinesis.partition_key.clone().unwrap())
			.build()
			.unwrap();
		entries.push(entry);
	});
	debug!("Posting messages: {}", entries.len());
	let resp = kinesis
		.put_records()
		.stream_arn(write)
		.set_records(Some(entries))
		.send()
		.await?;
	debug!("Posted messages: {:?}", resp);
	Ok(())
}

////////////////////////////////////////////////////////////////////////////////
///                                Constants.                                ///
////////////////////////////////////////////////////////////////////////////////

/// The name of the environment variable that specifies the name of the Kinesis
/// stream to which messages should be posted. This environment exists in the
/// Lambda execution environment, not in the local development environment.
const WRITE_STREAM: &str = "KINESIS_EVENT_B";
