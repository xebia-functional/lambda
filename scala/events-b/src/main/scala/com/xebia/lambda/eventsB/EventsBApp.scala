package com.xebia.lambda.eventsB

import cats.effect.IO
import cats.implicits.*
import com.amazonaws.services.dynamodbv2.{AmazonDynamoDBAsync, AmazonDynamoDBAsyncClientBuilder}
import com.amazonaws.services.lambda.runtime.{Context, RequestHandler}
import com.amazonaws.services.lambda.runtime.events.KinesisEvent
import com.amazonaws.services.lambda.runtime.events.KinesisEvent.KinesisEventRecord
import com.amazonaws.services.dynamodbv2.model.AttributeValue
import com.xebia.lambda.Datum
import io.circe.*
import io.circe.generic.auto.*
import io.circe.jawn.JawnParser

import scala.jdk.CollectionConverters.*

object EventsBApp extends RequestHandler[KinesisEvent, Unit] {

  val WRITE_TABLE = "DYNAMODB_WRITE_TABLE"

  val dynamoDbClient: IO[AmazonDynamoDBAsync] = IO(AmazonDynamoDBAsyncClientBuilder.defaultClient())
  override def handleRequest(event: KinesisEvent, context: Context): Unit =
    import cats.effect.unsafe.implicits.global
    val prg =
      for
        client <- dynamoDbClient
        parser = new JawnParser()
        _ <- event.getRecords.asScala.toList.traverse(processRecord(parser, client))
      yield ()
    prg.unsafeRunSync()

  def processRecord(parser: JawnParser, db: AmazonDynamoDBAsync)(record: KinesisEventRecord): IO[Unit] =
    for
      json <- IO.fromEither(parser.parseByteBuffer(record.getKinesis.getData))
      datum <- IO.fromEither(json.as[Datum])
      fut = db.putItemAsync(WRITE_TABLE, Map(
          "uuid" -> AttributeValue().withS(datum.uuid.toString),
          "doc" -> AttributeValue().withS(datum.doc),
          "hashes" -> AttributeValue().withN(datum.hashes.toString),
          "hash" -> AttributeValue().withS(datum.hash.get)
        ).asJava)
      _ <- IO.blocking(fut)
    yield ()
}
