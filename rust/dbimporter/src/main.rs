use data::Datum;

use std::str::FromStr;
// import com.amazonaws.services.lambda.runtime.events.{APIGatewayV2HTTPEvent, APIGatewayV2HTTPResponse}
// import com.amazonaws.services.lambda.runtime.{Context, LambdaLogger, RequestHandler}
// import software.amazon.awssdk.core.SdkBytes

use aws_sdk_dynamodb::{types::AttributeValue, Client};
// use aws_lambda_events::event::    //kinesis::KinesisEvent;
// use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use tracing::{debug, info, trace};
use aws_sdk_dynamodb::{primitives::Blob, types::AttributeValue, Client};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
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

	run(service_fn(|event: Request| async {
		handle_request(&db, event).await
	}))
	.await
}

////////////////////////////////////////////////////////////////////////////////
///                                Endpoints.                                ///
////////////////////////////////////////////////////////////////////////////////

async fn handle_request(db: &Client, event: Request) -> Result<Response<Body>, Error> {
	debug!("Received request: {:?}", event);
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

	// Extract the query parameters from the request.
	let chars = param_or_default(&event, LENGTH_PARAM, 1024usize);
	let hashes = param_or_default(&event, HASHES_PARAM, 100u16);
	let messages = param_or_default(&event, MESSAGES_PARAM, 64usize);
	debug!("chars={}, hashes={}, messages={}", chars, hashes, messages);

	// Produce the requested number of random messages.
	let mut batch = Vec::with_capacity(messages);
	for _ in 0..messages {
		let datum = Datum::random(chars, hashes);
		trace!("Generated datum: {:?}", &datum);
		batch.push(datum);
	}
	trace!("Generated messages: {}", batch.len());

	// Respond with a simple affirmation.
	let resp = Response::builder()
		.status(200)
		.header("content-type", "text/html")
		.body(format!("{} messages posted to Kinesis.", succeeded).into())
		.map_err(Box::new)?;
	trace!("Responded with: {:?}", &resp);
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

// val tableName = "demo_stock_prices"
//
// val dynamoDbClient = DynamoDbClient.builder
//   .credentialsProvider(DefaultCredentialsProvider.create)
//   .region(Region.US_EAST_1)
//   .build
//
// class Handler extends RequestHandler[APIGatewayV2HTTPEvent, APIGatewayV2HTTPResponse] :
//
//   override def handleRequest(event: APIGatewayV2HTTPEvent, context: Context): APIGatewayV2HTTPResponse =
//     given lambdaLogger: LambdaLogger = context.getLogger
//
//     lambdaLogger.log("Start")
//     lambdaLogger.log(event.getBody)
//
//     val body = Option(event.getBody).withFilter(!_.isBlank)
//       .map(Success(_))
//       .getOrElse(Failure(ParseException("empty body", "''")))
//
//     body
//       .flatMap(parseStockPriceItems)
//       .flatMap(putIntoDynamoDB)
//       .fold(errorToResult, count =>
//         APIGatewayV2HTTPResponse.builder
//           .withStatusCode(200)
//           .withBody(s"""{ "added": $count }""")
//           .build
//       )
//
// def parseStockPriceItems(json: String)(using lambdaLogger: LambdaLogger): Try[Iterable[StockPriceItem]] =
//   lambdaLogger.log(json)
//   Try(JsonNodeParser.create.parse(json).asArray.asScala.map(StockPriceItem.apply))
//
// def putIntoDynamoDB(stockPriceItems: Iterable[StockPriceItem])(using lambdaLogger: LambdaLogger): Try[Long] = Try {
//   val writeRequests = stockPriceItems.map {
//     stockPriceItem =>
//       val request = PutRequest.builder.item(stockPriceItem.dynamoDBAttributeMap).build
//       WriteRequest.builder.putRequest(request).build
//   }
//   val requestItems = Map(tableName -> writeRequests.toList.asJava).asJava
//   val batchWriteItemRequest = BatchWriteItemRequest.builder.requestItems(requestItems).build
//   val batchWriteItemResponse = dynamoDbClient.batchWriteItem(batchWriteItemRequest)
//   if (batchWriteItemResponse.hasUnprocessedItems && batchWriteItemResponse.unprocessedItems.size > 0) {
//     val message = s"Wrote ${writeRequests.size - batchWriteItemResponse.unprocessedItems.size} of ${writeRequests.size}"
//     throw new Exception(message)
//   } else {
//     lambdaLogger.log("Success")
//     writeRequests.size
//   }
// }
//
// def errorToResult(ex: Throwable)(using lambdaLogger: LambdaLogger): APIGatewayV2HTTPResponse =
//   ex match
//     case ParseException(error, content) =>
//       val message = s"Error parsing request $error in $content"
//       lambdaLogger.log(message)
//       APIGatewayV2HTTPResponse.builder.withStatusCode(400).withBody(message).build
//     case _ =>
//       throw ex
