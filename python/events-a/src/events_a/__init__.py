"""
Herein is the implementation of the `events-a` service, which consumes
documents from Kinesis, computes an iterative SHA-512 hash of the document,
and re-injects the document into Kinesis.
"""

from data import Datum
