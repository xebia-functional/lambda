"""
Herein is the implementation of the `events-a` service, which consumes
documents from Kinesis, computes an iterative SHA-512 hash of the document,
and re-injects the document into Kinesis.
"""

import base64
from boto3 import BotoCoreError, ClientError
import logging
from mypy_boto3_kinesis.client import KinesisClient
import os

from data import Datum

# Set the logging level.
logging.basicConfig(level = logging.DEBUG)

# Set up the Kinesis client.
kinesis: KinesisClient = boto3.client('kinesis')

def handler(event, context):
	logging.debug(f'Received event: {event}')
	write_stream: str = os.environ['KINESIS_EVENT_B_STREAM']
	logging.debug(f'Posting messages to Kinesis stream: {write_stream}')
	entries = []

	for record in event['Records']:
		data: str = base64.b64decode(record['kinesis']['data']).decode('utf-8')
		logging.debug(f'JSON: {data}')
		data: Datum = Datum.from_json(data)
		logging.debug(f'Deserialized datum: {data}')
		data.iterated_hash()
		logging.debug(f'Outgoing datum: {data}')
		data = data.to_json().encode('utf-8')
		entry = {
			'Data': data,
			'PartitionKey': record['kinesis']['partitionKey']
		}
		entries.append(entry)

	logging.debug("Posting messages: %d", len(entries))
	try:
		response = kinesis.put_records(
			Records=entries,
			StreamName=write_stream
		)
		logging.debug("Posted messages: %s", response)
	except (BotoCoreError, ClientError) as error:
		logging.error(error)
		return {
			'statusCode': 500,
			'body': json.dumps(str(error))
		}

	return {
		'statusCode': 200,
		'body': json.dumps('Success')
	}
