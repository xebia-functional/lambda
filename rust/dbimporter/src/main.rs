use aws_sdk_dynamodb::{types::AttributeValue, Client};
use futures::executor::block_on;
use lambda_http::{
	http::{Response, StatusCode},
	run, service_fn, Error, IntoResponse, Request, RequestPayloadExt,
};
use serde::{Deserialize, Serialize};
// use serde_json::json;
// use std::str::FromStr;
use tracing::{debug, info, trace};

#[tokio::main]
async fn main() -> Result<(), Error> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::TRACE)
		.with_target(false)
		.without_time()
		.init();

	let cfg = aws_config::load_from_env().await;
	info!("Loaded configuration: {:?}", cfg);
	let db = Client::new(&cfg);

	run(service_fn(|event: Request| async {
		handler(&db, event).await
	}))
	.await
}

pub async fn handler(db: &Client, event: Request) -> Result<impl IntoResponse, Error> {
	debug!("Received request: {:?}", event);

	let cnt: i32 = event
		.payload::<Vec<StockPriceItem>>()?
		.iter()
		.flatten()
		.map(|d| async { add_item(d, &db).await })
		.fold(0, |acc, x| acc + block_on(x).unwrap_or(0));

	let resp: Response<String> = Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", "application/json")
		.body(format!("{} messages inserted into DynamoDB", cnt).into())
		.map_err(Box::new)?;

	trace!("Responded with: {:?}", &resp);
	Ok(resp)
}

async fn add_item(d: &StockPriceItem, db: &Client) -> Result<i32, Error> {
	db.put_item()
		.table_name("StockPriceItems")
		.item("symbol", AttributeValue::S(d.symbol.to_owned()))
		.item("time", AttributeValue::S(d.time.to_owned()))
		.item("prices", AttributeValue::S(d.prices.to_owned()))
		.send()
		.await?;

	trace!("Stored StockPriceItem");

	Ok(1)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StockPriceItem {
	pub symbol: String,
	pub time: String,
	pub prices: String,
}
