use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use futures::executor::block_on;
use lambda_http::{
	http::{Response, StatusCode},
	run, service_fn, Error, IntoResponse, Request, RequestPayloadExt,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Error> {
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::INFO)
		.with_target(false)
		.without_time()
		.init();

	let cfg = aws_config::load_defaults(BehaviorVersion::latest()).await;
	let db = Client::new(&cfg);

	info!("Starting dbimporter");
	run(service_fn(|event: Request| async {
		handler(&db, event).await
	}))
	.await
}

pub async fn handler(db: &Client, event: Request) -> Result<impl IntoResponse, Error> {
	info!("Received request: {:?}", event);

	let cnt: i32 = event
		.payload::<Vec<StockPriceItem>>()?
		.iter()
		.flatten()
		.map(|d| async {
			warn!("{:?}", d.clone());
			add_item(d, &db).await
		})
		.fold(0, |acc, x| acc + block_on(x).unwrap_or(0));

	let resp: Response<String> = Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", "application/json")
		.body(format!("{} messages inserted into DynamoDB", cnt).into())
		.map_err(Box::new)?;

	info!("Responded with: {:?}", &resp);
	Ok(resp)
}

async fn add_item(d: &StockPriceItem, db: &Client) -> Result<i32, Error> {
	db.put_item()
		.table_name("StockPriceItems")
		.item("symbol", AttributeValue::S(d.symbol.to_owned()))
		.item("time", AttributeValue::N(d.time.to_string()))
		.item("prices", AttributeValue::S(d.prices.to_owned())) // TODO: convert to B blob type
		.send()
		.await?;

	info!("Stored StockPriceItem");

	Ok(1)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StockPriceItem {
	pub symbol: String,
	pub time: u32,
	pub prices: String,
}
