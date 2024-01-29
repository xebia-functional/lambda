import
	{
		DynamoDBClient,
		PutItemCommand,
		PutItemCommandOutput,
		PutItemInput,
	} from "@aws-sdk/client-dynamodb";
import { Handler, KinesisStreamEvent } from "aws-lambda";
import { Datum } from "../../data/src/data.js";

/** The {@link DynamoDBClient DynamoDB client}. */
const db = new DynamoDBClient({});

/**
 * The name of the environment variable that specifies the name of the DynamoDB
 * table to which messages should be posted. This environment exists in the
 * Lambda execution environment, not in the local development environment.
 */
const WRITE_TABLE = process.env["DYNAMODB_WRITE_TABLE"];

/**
 * Process an incoming Kinetic {@link KinesisStreamEvent Kinetic event} by
 * storing it into DynamoDB. Incoming messages are JSON serializations of
 * {@link Datum}.
 *
 * @param event
 *   The {@link KinesisStreamEvent Kinesis event}.
 * @returns
 *   The {@link PutItemCommandOutput Kinesis response}.
 */
export const handler: Handler = async (
	event: KinesisStreamEvent,
): Promise<void> =>
{
	console.debug("Received event: ", event);
	const writeTable = process.env["DYNAMODB_WRITE_TABLE"];
	console.debug("Writing messages to DynamoDB table: ", writeTable);
	const promises: Array<Promise<PutItemCommandOutput | undefined>> = [];

	for (const record of event.Records)
	{
		console.trace("Incoming record: ", record);
		const base64 = record.kinesis.data.toString();
		console.trace("Base64: ", base64);
		const data = Buffer.from(record.kinesis.data, "base64").toString("ascii");
		console.trace("ASCII: ", data);
		const datum = Datum.fromJSON(data);
		if (datum === undefined)
		{
			console.error("Failed to deserialize datum: ", data);
			continue;
		}
		console.trace("Deserialized datum: ", datum.toString());
		const item: PutItemInput = {
			Item: {
				uuid: { S: datum.uuid() },
				doc: { S: datum.doc() },
				hashes: { N: datum.hashes().toString() },
				hash: { S: datum.hash() },
			},
			TableName: writeTable,
		};
		console.debug("Storing datum: ", datum.hash());
		const command: PutItemCommand = new PutItemCommand(item);
		promises.push(db.send(command));
	}

	await Promise.all(promises);
	console.debug("Stored items: ", promises.length);
};
