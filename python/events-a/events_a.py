"""
Herein is the implementation of the `events-a` service, which consumes
documents from Kinesis, computes an iterative SHA-512 hash of the document,
and re-injects the document into Kinesis.
"""

import base64
import logging
from typing import MutableSequence
import os

from boto3 import client
from boto3.exceptions import Boto3Error
from botocore.exceptions import ClientError

from aws_lambda_powertools.utilities.typing import LambdaContext
from mypy_boto3_kinesis.type_defs import PutRecordsRequestEntryTypeDef
from mypy_boto3_kinesis.client import KinesisClient

from datum import Datum

# Set the logging level.
logging.basicConfig(level = logging.DEBUG, force = True)

# Set up the Kinesis client.
kinesis: KinesisClient = client('kinesis')
"""The Kinesis client."""

def handler(event: dict, context: LambdaContext) -> None:
	# pylint: disable=unused-argument
	"""
	Process an incoming Kinetic event by computing the hash of each message and
	then re-posting the augmented messages to another Kinesis stream. Incoming
	messages are JSON serializations of `Datum`.
	:param event: The incoming Kinetic event.
	:param context: The Lambda context.
	"""
	print('Got here\n')
	logging.debug('Received event: %s', event)
	write_stream: str = os.environ['KINESIS_EVENT_B']
	logging.debug('Posting messages to Kinesis stream: %s', write_stream)
	entries: MutableSequence[PutRecordsRequestEntryTypeDef] = []

	for record in event['Records']:
		json: str = base64.b64decode(record['kinesis']['data']).decode('utf-8')
		logging.debug('JSON: %s', json)
		data: Datum = Datum.from_json(json)
		logging.debug('Deserialized datum: %s', data)
		data.iterated_hash # pylint: disable=pointless-statement
		logging.debug('Outgoing datum: %s', data)
		output: bytes = data.to_json().encode('utf-8')
		entry: PutRecordsRequestEntryTypeDef = {
			'Data': output,
			'PartitionKey': record['kinesis']['partitionKey']
		}
		entries.append(entry)

	logging.debug('Posting messages: %d', len(entries))
	try:
		response = kinesis.put_records(
			Records=entries,
			StreamName=write_stream
		)
		logging.debug('Posted messages: %s', response)
	except (Boto3Error, ClientError) as error:
		logging.error(error)
