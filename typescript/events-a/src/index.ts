import
	{
		KinesisClient,
		PutRecordsCommand,
		PutRecordsInput,
		PutRecordsRequestEntry
	} from "@aws-sdk/client-kinesis";
import { Handler, KinesisStreamEvent } from "aws-lambda";
import { Datum } from "../../data/src/data.js";

/** The {@link KinesisClient Kinesis client}. */
const kinesis = new KinesisClient({});

/**
 * The name of the environment variable that specifies the name of the Kinesis
 * stream to which messages should be posted. This environment exists in the
 * Lambda execution environment, not in the local development environment.
 */
const WRITE_STREAM = process.env["KINESIS_EVENT_B"];

/**
 * Process an incoming {@link KinesisStreamEvent Kinetic event} by computing the
 * hash for each message and then re-posting the augmented messages to another
 * Kinesis stream. Incoming messages are JSON serializations of {@link Datum}.
 *
 * @param event
 *   The {@link KinesisStreamEvent Kinesis event}.
 * @returns
 *   The {@link PutRecordCommandOutput Kinesis response}.
 */
export const handler: Handler = async (
	event: KinesisStreamEvent
): Promise<void> =>
{
	console.debug("Received event: ", event);
	const writeStream = WRITE_STREAM;
	console.debug("Posting messages to Kinesis stream: ", writeStream);
	const entries = [];

	for (const record of event.Records)
	{
		console.trace("Incoming record: ", record);
		const base64 = record.kinesis.data.toString();
		console.trace("Base64: ", base64);
		const data =
			Buffer.from(record.kinesis.data, 'base64').toString('ascii');
		console.trace("ASCII: ", data);
		const datum = Datum.fromJSON(data);
		if (datum === undefined)
		{
			console.error("Failed to deserialize datum: ", data);
			continue;
		}
		console.trace("Deserialized datum: ", datum.toString());
		datum.hash();
		console.trace("Outgoing datum: ", datum.toString());
		const json = JSON.stringify(datum);
		console.trace("Outgoing JSON: ", json);
		if (json === "{}")
		{
			console.error("JSON object is empty!");
			continue;
		}
		const entry: PutRecordsRequestEntry = {
			Data: Buffer.from(json),
			PartitionKey: record.kinesis.partitionKey
		};
		entries.push(entry);
	}

	console.debug("Posting messages: ", entries.length);
	const putRecordsInput: PutRecordsInput = {
		Records: entries,
		StreamARN: writeStream
	};
	const putRecordsCommand = new PutRecordsCommand(putRecordsInput);
	const response = await kinesis.send(putRecordsCommand);
	console.debug("Posted messages: ", response);
};
