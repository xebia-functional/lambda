package com.xebia.lambda

import cats.effect.IO
import com.amazonaws.services.kinesis.model.{PutRecordsRequest, PutRecordsResult, PutRecordsRequestEntry}
import com.amazonaws.services.kinesis.{AmazonKinesisAsync, AmazonKinesisAsyncClientBuilder}

object Kinesis {
  def kinesisClient: IO[AmazonKinesisAsync] = IO(
    AmazonKinesisAsyncClientBuilder.defaultClient()
  )

  def postData(kinesis: AmazonKinesisAsync, stream: String, data: List[Datum]): IO[Unit] =
    val putReq: PutRecordsRequest = PutRecordsRequest().withStreamName(stream)
    putReq.withRecords(data.map(datum =>
      PutRecordsRequestEntry()
        .withPartitionKey(datum.uuid.toString)
        .withData(Datum.serialize(datum))): _*
    )
    val putRes: IO[PutRecordsResult] = {
      val fut = kinesis.putRecordsAsync(putReq)
      IO.blocking(fut.get())
    }
    putRes.void
}
