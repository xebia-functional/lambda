"""
Herein is the implementation of `Datum`, which is the data currency produced by
the `https-a` service and used by the `events-a` and `events-b` services.
"""

from hashlib import sha3_512
from json import dumps, loads
from typing import Optional

class Datum:
	"""
	Datum represents an arbitrary document. Instances are generated
	pseudo-randomly by the `httpd-a` service and placed into Kinesis for
	consumption by the `events-a` service. `events-a` compute an iterative
	SHA-512 hash of the document prior to re-injecting it into Kinesis. Finally,
	`events-b` consumes the document and stores it in DynamoDB.
	"""

	def __init__(
		self,
		uuid: str,
		doc: str,
		hashes: int,
		iterated_hash: Optional[str] = None
	):
		"""
		Construct a new Datum.

		:param str uuid: The unique identifier for this document.
		:param str doc: The pseudo-randomly generated document.
		:param int hashes: The target iteration count for the SHA-512 hash.
		:param hash: The hash of the document.
		:type hash: str or None
		"""
		self._uuid: str = uuid
		self._doc: str = doc
		self._hashes: int = hashes
		self._hash: Optional[str] = iterated_hash
		"""
		The hash of the document, computed by `events-a`.
		:type: str or None
		"""

	@property
	def uuid(self) -> str:
		"""
		The unique identifier for this document, used as a partition key in
		DynamoDB.
		:rtype: str
		"""
		return self._uuid

	@property
	def doc(self) -> str:
		"""
		The pseudo-randomly generated document, produced by `httpd-a`. The
		generator is only defined in the Rust project.
		:rtype: str
		"""
		return self._uuid

	@property
	def hashes(self) -> int:
		"""
		The target iteration count for the SHA-512 hash.
		:rtype: int
		"""
		return self._hashes

	@property
	def iterated_hash(self) -> str:
		"""
		The hash of the document, computed by `events-a`.
		:rtype: str or None
		"""
		if self._hash is None:
			partial = self.doc.encode('utf-8')
			for _ in range(self.hashes):
				partial = sha3_512(partial).hexdigest().upper().encode('utf-8')
			self._hash = partial.decode('utf-8')
		return self._hash

	def to_json(self) -> str:
		"""
		Serialize this Datum to JSON. Do not compute the iterated hash.
		:rtype: str
		"""
		return dumps({
			'uuid': self.uuid,
			'doc': self.doc,
			'hashes': self.hashes,
			'hash': self._hash
		})

	@staticmethod
	def from_json(json: str) -> 'Datum':
		"""
		Deserialize a Datum from JSON.
		:param str json: The JSON representation of a Datum.
		:rtype: Datum
		"""
		data = loads(json)
		return Datum(
			data['uuid'],
			data['doc'],
			data['hashes'],
			data['hash']
		)

	def __str__(self) -> str:
		"""
		Serialize this Datum to JSON.
		:rtype: str
		"""
		return f'doc: {self.doc}' \
			if self._hash is None \
			else f'hash: {self._hash}'
