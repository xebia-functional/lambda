use data::Datum;

use aws_lambda_events::event::kinesis::KinesisEvent;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
// use serde::{Deserialize, Serialize};
use serde_dynamo::to_attribute_value;
use tracing::info;
// use serde_json;

async fn function_handler(
	db_client: &Client,
	event: LambdaEvent<KinesisEvent>,
) -> Result<(), Error> {
	let datum=serde_json::from_str(event.)

	add_datum(db_client, datum.clone(), "CostOptLambdaData").await?;

	Ok(())
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

	run(service_fn(|event: LambdaEvent<KinesisEvent>| async {
		function_handler(&client, event).await
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
