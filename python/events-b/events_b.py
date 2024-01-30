"""
Herein is the implementation of the `events-a` service, which consumes
documents from Kinesis, computes an iterative SHA-512 hash of the document,
and re-injects the document into Kinesis.
"""

import base64
import logging
import os

from boto3 import client
from boto3.exceptions import Boto3Error
from botocore.exceptions import ClientError

from aws_lambda_powertools.utilities.typing import LambdaContext
from mypy_boto3_dynamodb.client import DynamoDBClient
from mypy_boto3_dynamodb.type_defs import PutItemInputRequestTypeDef

from datum import Datum

# Set the logging level.
logging.basicConfig(level = logging.WARNING, force = True)

db: DynamoDBClient = client('dynamodb')
"""The DynamoDB client."""

def handler(event: dict, context: LambdaContext) -> None:
	# pylint: disable=unused-argument
	"""
	Process an incoming Kinetic event by storing it into DynamoDB. Incoming
	messages are JSON serializations of `Datum`.
	:param event: The incoming Kinetic event.
	:param context: The Lambda context.
	"""
	logging.debug('Received event: %s', event)
	write_table: str = os.environ['DYNAMODB_WRITE_TABLE']
	logging.debug('Writing messages to DynamoDB table: %s', write_table)

	count: int = 0
	for record in event['Records']:
		json: str = base64.b64decode(record['kinesis']['data']).decode('utf-8')
		logging.debug('JSON: %s', json)
		data: Datum = Datum.from_json(json)
		logging.debug('Deserialized datum: %s', data)
		item: PutItemInputRequestTypeDef = {
			'Item': {
				'uuid': {'S': data.uuid},
				'doc': {'S': data.doc},
				'hashes': {'N': str(data.hashes)},
				'hash': {'S': data.iterated_hash}
			},
			'TableName': write_table
		}
		logging.debug('Storing datum: %s', data.iterated_hash)
		try:
			response = db.put_item(**item)
			logging.debug('Stored datum: %s', response)
			count += 1
		except (Boto3Error, ClientError) as error:
			logging.error(error)

	logging.debug('Stored items: %d', count)
