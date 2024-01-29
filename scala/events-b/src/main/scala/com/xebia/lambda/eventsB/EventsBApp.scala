package com.xebia.lambda
package eventsB

import cats.effect.IO
import cats.implicits.*
import com.amazonaws.services.dynamodbv2.{
  AmazonDynamoDBAsync,
  AmazonDynamoDBAsyncClientBuilder
}
import com.amazonaws.services.lambda.runtime.{Context, RequestHandler}
import com.amazonaws.services.lambda.runtime.events.KinesisEvent
import com.amazonaws.services.lambda.runtime.events.KinesisEvent.KinesisEventRecord
import com.amazonaws.services.dynamodbv2.model.AttributeValue
import io.circe.*
import io.circe.generic.auto.*

import cats.effect.std.Env

import scala.jdk.CollectionConverters.*

object EventsBApp extends RequestHandler[KinesisEvent, Unit] {

  val WRITE_TABLE = "DYNAMODB_WRITE_TABLE"

  val dynamoDbClient: IO[AmazonDynamoDBAsync] = IO(
    AmazonDynamoDBAsyncClientBuilder.defaultClient()
  )

  override def handleRequest(event: KinesisEvent, context: Context): Unit =
    import cats.effect.unsafe.implicits.global
    given logger: Logger[IO] = Logger.ioLogger(context.getLogger)
    val prg                  =
      for
        _        <- logger.debug(s"Received event: $event")
        tableOpt <- Env[IO].get(WRITE_TABLE)
        table    <-
          IO.fromOption(tableOpt)(
            new RuntimeException(s"missing $WRITE_TABLE environment variable")
          )
        _        <- logger.debug(s"Writing messages to DynamoDB table: $table")
        client   <- dynamoDbClient
        records   = event.getRecords.asScala.toList
        _        <- logger.debug(s"Writing records to DynamoDB: ${records.size}")
        _        <- records.parTraverse(processRecord(storeDatum(client, table)))
        _        <- logger.debug(s"Wrote records to DynamoDB: ${records.size}")
      yield ()
    prg.unsafeRunSync()

  def processRecord(
      store: Datum => IO[Unit]
  )(using logger: Logger[IO]): KinesisEventRecord => IO[Unit] =
    record =>
      for
        jsonOpt <- IO.pure(
                     jawn.parseByteBuffer(record.getKinesis.getData).toOption
                   )
        _       <- logger.trace(s"JSON: $jsonOpt")
        datum   <- IO.pure(jsonOpt.flatMap(_.as[Datum].toOption))
        _       <- datum.map(store).getOrElse(IO.unit)
      yield ()

  def storeDatum(db: AmazonDynamoDBAsync, table: String)(
      datum: Datum
  )(using logger: Logger[IO]): IO[Unit] =
    for
      _   <- logger.trace(s"Incoming datum: $datum")
      fut <- IO(
               db.putItemAsync(
                 table,
                 Map(
                   "uuid"   -> AttributeValue().withS(datum.uuid.toString),
                   "doc"    -> AttributeValue().withS(datum.doc),
                   "hashes" -> AttributeValue().withN(datum.hashes.toString),
                   "hash"   -> AttributeValue().withS(datum.hash.get)
                 ).asJava
               )
             )
      _   <- IO.blocking(fut.get)
      _   <- logger.trace(s"Stored datum: $datum")
    yield ()
}
