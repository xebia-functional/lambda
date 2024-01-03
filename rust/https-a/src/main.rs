// use aws_sdk_dynamodb::Client; //, Error};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // let shared_config = aws_config::load_from_env().await;
    // let client = Client::new(&shared_config);
    // let req = client.list_tables().limit(10);
    // let resp = req.send().await?;
    // println!("Current DynamoDB tables: {:?}", resp.table_names);
    // let message = format!("Current DynamoDB tables: {:?}", resp.table_names);

    // Extract some useful information from the request
    let who = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or("world");

    use sha3::{Digest, Sha3_512};

    let mut hasher = Sha3_512::new();
    let data = b"Hello world!";
    hasher.update(data);
    // `update` can be called repeatedly and is generic over `AsRef<[u8]>`
    hasher.update("String data");
    // Note that calling `finalize()` consumes hasher
    let hash = hasher.finalize();

    let message = format!("Hello {who}, AWS Lambda HTTP {:?}", hash);

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
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
