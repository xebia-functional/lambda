package com.xebia.lambda
package eventsA

import cats.implicits.*
import cats.effect.IO
import com.amazonaws.services.lambda.runtime.{Context, RequestHandler}
import com.amazonaws.services.lambda.runtime.events.KinesisEvent
import com.amazonaws.services.lambda.runtime.events.KinesisEvent.KinesisEventRecord
import com.xebia.lambda.{Datum, Kinesis}
import io.circe.*
import io.circe.generic.auto.*
import io.circe.jawn.JawnParser
import pt.kcry.sha.*

import scala.jdk.CollectionConverters.*
object EventsAApp extends RequestHandler[KinesisEvent, Unit]{

  val WRITE_STREAM = "KINESIS_EVENT_B"

  extension (d: Datum)
    def computeHash: Datum =
      if d.hash.isEmpty then
        val hashedBytes = (0 to d.hashes).foldLeft(d.doc.getBytes("UTF-8")){case (h, _) =>
          Sha3_512.hash(h)
        }
        val hashSb = new StringBuilder()
        hashedBytes.foreach(b => String.format("%02x", Byte.box(b)))
        d.copy(hash = Some(hashSb.toString))
      else d

  override def handleRequest(event: KinesisEvent, context: Context): Unit =
    import cats.effect.unsafe.implicits.global
    val parser = new JawnParser()
    given log: Logger[IO] = Logger.ioLogger(context.getLogger)
    val prg = 
      for 
        _ <- log.debug(s"Received event: $event")
        hashedRecords <- event.getRecords.asScala.toList.traverse(processRecord(parser)) 
        kinesis <- Kinesis.kinesisClient
        _ <- Kinesis.postData(kinesis, WRITE_STREAM, hashedRecords)
      yield ()
    prg.unsafeRunSync()

  def processRecord(parser: JawnParser)(record: KinesisEventRecord)(using log: Logger[IO]): IO[Datum] =
    for 
      _ <- log.trace(s"Incoming record: $record")
      json <- IO.fromEither(parser.parseByteBuffer(record.getKinesis.getData))
      _ <- log.trace(s"JSON: $json")
      datum <- IO.fromEither(json.as[Datum])
      _ <- log.trace(s"Deserialized datum: $datum")
      hashed = datum.computeHash
      _ <- log.trace(s"Outgoing datum: $hashed")
    yield hashed


}
