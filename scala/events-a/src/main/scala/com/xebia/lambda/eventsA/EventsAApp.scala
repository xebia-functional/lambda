package com.xebia.lambda
package eventsA

import cats.implicits.*
import cats.effect.IO
import cats.effect.std.Env
import com.amazonaws.services.lambda.runtime.{Context, RequestHandler}
import com.amazonaws.services.lambda.runtime.events.KinesisEvent
import com.amazonaws.services.lambda.runtime.events.KinesisEvent.KinesisEventRecord
import io.circe.*
import io.circe.generic.auto.*
import pt.kcry.sha.*

import scala.jdk.CollectionConverters.*
object EventsAApp extends RequestHandler[KinesisEvent, Unit] {

  val WRITE_STREAM = "KINESIS_EVENT_B"

  def computeHash(d: Datum): Datum =
    if d.hash.isEmpty then
      val hashedBytes = (0 to d.hashes).foldLeft(d.doc.getBytes("UTF-8")) {
        case (h, _) =>
          Sha3_512.hash(h)
      }

      d.copy(hash =
        Some(
          hashedBytes
            .foldLeft(new StringBuilder())((sb, b) =>
              sb ++= String.format("%02x", Byte.box(b))
            )
            .toString
        )
      )
    else d

  override def handleRequest(event: KinesisEvent, context: Context): Unit =
    import cats.effect.unsafe.implicits.global
    given log: Logger[IO] = Logger.ioLogger(context.getLogger)
    val prg               =
      for
        _                  <- log.debug(s"Received event: $event")
        streamOpt          <- Env[IO].get(WRITE_STREAM)
        stream             <-
          IO.fromOption(streamOpt)(
            new RuntimeException(s"Missing environment variable $WRITE_STREAM")
          )
        hashedRecords      <- event.getRecords.asScala.toList.traverse(processRecord)
        kinesis            <- Kinesis.kinesisClient
        successfullyPosted <- Kinesis.postData(
                                kinesis,
                                stream,
                                hashedRecords.flatten
                              )
        _                  <- log.debug(s"Posted $successfullyPosted messages")
      yield ()
    prg.unsafeRunSync()

  def processRecord(
      record: KinesisEventRecord
  )(using log: Logger[IO]): IO[Option[Datum]] =
    for
      _     <- log.trace(s"Incoming record: $record")
      json  <- IO.pure(
                 jawn.parseByteBuffer(record.getKinesis.getData).toOption
               )
      _     <- log.trace(s"JSON: $json")
      datum <- IO.pure(json.flatMap(_.as[Datum].toOption))
      _     <- log.trace(s"Deserialized datum: $datum")
      hashed = datum.map(computeHash)
      _     <- log.trace(s"Outgoing datum: $hashed")
    yield hashed

}
