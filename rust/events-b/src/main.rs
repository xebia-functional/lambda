use aws_lambda_events::event::kinesis::KinesisEvent;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use data::Datum;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use tracing::{debug, info, trace};

////////////////////////////////////////////////////////////////////////////////
///                               Entry point.                               ///
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Error> {
	// Initialize the logger. Disable per-line module name and time printing,
	// since CloudWatch will take care of this.
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::TRACE)
		.with_target(false)
		.without_time()
		.init();

	// Create the DynamoDB client using configuration data supplied through the
	// environment.
	let config = aws_config::load_from_env().await;
	info!("Loaded configuration: {:?}", config);
	let db = Client::new(&config);

	run(service_fn(|event: LambdaEvent<KinesisEvent>| async {
		handle_request(&db, event).await
	}))
	.await
}

////////////////////////////////////////////////////////////////////////////////
///                                Endpoints.                                ///
////////////////////////////////////////////////////////////////////////////////

/// Process an incoming Kinetic [event](KinesisEvent) by storing it into
/// DynamoDB. Incoming messages are JSON serializations of [`Data`](Datum).
async fn handle_request(
	db: &Client,
	event: LambdaEvent<KinesisEvent>
) -> Result<(), Error> {
	debug!("Received event: {:?}", event);
	let write = std::env::var(WRITE_TABLE)?;
	debug!("Writing messages to DynamoDB table: {}", write);
	let mut records = vec![];
	event.payload.records.iter().for_each(|record| {
		let data = String::from_utf8_lossy(&record.kinesis.data.0);
		let datum: Datum = serde_json::from_str(&data).unwrap();
		trace!("Incoming datum: {:?}", datum);
		records.push(datum);
	});
	debug!("Writing records to DynamoDB: {}", records.len());
	for record in records {
		add_datum(db, &write, record).await?;
	}
	debug!("Wrote records to DynamoDB");
	Ok(())
}

////////////////////////////////////////////////////////////////////////////////
///                                Utilities.                                ///
////////////////////////////////////////////////////////////////////////////////

/// Add a [`Datum`] to the specified DynamoDB table.
pub async fn add_datum(
	db: &Client,
	table: &str,
	d: Datum
) -> Result<(), Error> {
	trace!("Storing datum: {:?}", &d);
	let uuid = AttributeValue::S(d.uuid.to_string());
	let doc = AttributeValue::S(d.doc);
	let hashes = AttributeValue::N(d.hashes.to_string());
	let hash = AttributeValue::S(d.hash.unwrap().to_string());
	let request = db
		.put_item()
		.table_name(table)
		.item("uuid", uuid)
		.item("doc", doc)
		.item("hashes", hashes)
		.item("hash", hash);
	request.send().await?;
	trace!("Stored datum");
	Ok(())
}

////////////////////////////////////////////////////////////////////////////////
///                                Constants.                                ///
////////////////////////////////////////////////////////////////////////////////

/// The name of the environment variable that specifies the name of the DynamoDB
/// table wherein messages should be recorded. This environment exists in the
/// Lambda execution environment, not in the local development environment.
const WRITE_TABLE: &str = "DYNAMODB_WRITE_TABLE";
