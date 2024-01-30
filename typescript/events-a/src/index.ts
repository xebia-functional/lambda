import
{
	KinesisClient,
	PutRecordsCommand,
	PutRecordsCommandOutput,
	PutRecordsInput,
	PutRecordsRequestEntry
} from "@aws-sdk/client-kinesis";
import { Handler, KinesisStreamEvent } from "aws-lambda";
import winston from "winston";
import { Datum } from "../../data/src/data.js";

/** Set up the logger. */
const logger = winston.createLogger({
	level: "warn",
	format: winston.format.combine(
		winston.format.timestamp(),
		winston.format.prettyPrint()
	),
	defaultMeta: { service: "events-a" },
	transports: [
		new winston.transports.Console()
	]
});

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
 *   The {@link PutRecordsCommandOutput Kinesis response}.
 */
export const handler: Handler = async (
	event: KinesisStreamEvent
): Promise<void> =>
{
	logger.debug("Received event: ", event);
	const writeStream = WRITE_STREAM;
	logger.debug("Posting messages to Kinesis stream: ", writeStream);
	const entries = [];

	for (const record of event.Records)
	{
		logger.debug("Incoming record: ", record);
		const base64 = record.kinesis.data.toString();
		logger.debug("Base64: ", base64);
		const data =
			Buffer.from(record.kinesis.data, 'base64').toString('ascii');
		logger.debug("ASCII: ", data);
		const datum = Datum.fromJSON(data);
		if (datum === undefined)
		{
			logger.error("Failed to deserialize datum: ", data);
			continue;
		}
		logger.debug("Deserialized datum: ", datum.toString());
		datum.hash();
		logger.debug("Outgoing datum: ", datum.toString());
		const json = JSON.stringify(datum);
		logger.debug("Outgoing JSON: ", json);
		if (json === "{}")
		{
			logger.error("JSON object is empty!");
			continue;
		}
		const entry: PutRecordsRequestEntry = {
			Data: Buffer.from(json),
			PartitionKey: record.kinesis.partitionKey
		};
		entries.push(entry);
	}

	logger.debug("Posting messages: ", entries.length);
	const putRecordsInput: PutRecordsInput = {
		Records: entries,
		StreamARN: writeStream
	};
	const command = new PutRecordsCommand(putRecordsInput);
	const response: PutRecordsCommandOutput = await kinesis.send(command);
	logger.debug("Posted messages: ", response);
};
