package com.xebia.lambda

import cats.effect.IO
import com.amazonaws.services.kinesis.model.{PutRecordsRequest, PutRecordsResult, PutRecordsRequestEntry}
import com.amazonaws.services.kinesis.{AmazonKinesisAsync, AmazonKinesisAsyncClientBuilder}

object Kinesis {
  def kinesisClient: IO[AmazonKinesisAsync] = IO(
    AmazonKinesisAsyncClientBuilder.defaultClient()
  )

  def postData(kinesis: AmazonKinesisAsync, stream: String, data: List[Datum])(using log: Logger[IO]): IO[Int] =
    val putReq: PutRecordsRequest = PutRecordsRequest().withStreamARN(stream)
    putReq.withRecords(data.map(datum =>
      PutRecordsRequestEntry()
        .withPartitionKey(datum.uuid.toString)
        .withData(Datum.serialize(datum))): _*
    )
    val putRes: IO[PutRecordsResult] = {
      IO.delay(kinesis.putRecordsAsync(putReq)).flatMap(fut => IO.blocking(fut.get()))

    }

    log.debug(s"Posting ${data.size} messages to Kinesis stream: $stream") *> putRes.flatMap(res =>
      val successfullyPutRecords = res.getRecords.size() - res.getFailedRecordCount
      (if res.getFailedRecordCount > 0 then
        log.debug(s"Failed to post ${res.getFailedRecordCount} records") *> IO.pure(successfullyPutRecords)
      else IO.pure(successfullyPutRecords)) <* log.debug(s"Posted $successfullyPutRecords messages")

    )
}
