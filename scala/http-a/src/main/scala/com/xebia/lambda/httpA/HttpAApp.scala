package com.xebia.lambda.httpA

import cats.effect.{ExitCode, IO, IOApp}
import cats.effect.std.Random
import cats.implicits.*
import com.amazonaws.Request
import com.amazonaws.services.kinesis.model.{PutRecordsRequest, PutRecordsRequestEntry, PutRecordsResult}
import com.amazonaws.services.kinesis.{AmazonKinesisAsync, AmazonKinesisAsyncClientBuilder}
import com.amazonaws.services.lambda.runtime.RequestHandler
import com.amazonaws.services.lambda.runtime.Context
import com.amazonaws.services.lambda.runtime.events.APIGatewayV2HTTPResponse
import com.xebia.lambda.{Datum, Kinesis}

import scala.util.Try
import scala.jdk.CollectionConverters.*

object HttpAApp extends RequestHandler[Request[_], APIGatewayV2HTTPResponse] {

  /// The name of the query parameter that specifies the number of random
  /// characters to generate.
  val LENGTH_PARAM  = "chars"

  /// The name of the query parameter that specifies the number of hash iterations
  /// to perform.
  val HASHES_PARAM = "hashes"

  /// The name of the query parameter that specifies the number of messages to
  /// post to Kinesis.
  val MESSAGES_PARAM = "msgs"

  /// The name of the environment variable that specifies the name of the Kinesis
  /// stream to which messages should be posted. This environment exists in the
  /// Lambda execution environment, not in the local development environment.
  val WRITE_STREAM = "KINESIS_EVENT_A"

  

  private def getParameterOrDefault(event: Request[_], key: String, default: Int): Int =
    (for {
      jList <- Option(event.getParameters.get(key))
      head  <- if jList.size() > 0 then Some(jList.get(0)) else None
      value <- Try(head.toInt).toOption
    } yield value).getOrElse(default)

  def handleRequestIO(kinesis: AmazonKinesisAsync, event: Request[_]) : IO[APIGatewayV2HTTPResponse] =
    val chars = getParameterOrDefault(event, LENGTH_PARAM,1024)
    val hashes = getParameterOrDefault(event, HASHES_PARAM, 100)
    val messages = getParameterOrDefault(event, MESSAGES_PARAM, 64)
    val data:IO[List[Datum]] = List.fill(messages)((chars, hashes)).traverse(Datum.random[IO].tupled)
    data.map(Kinesis.postData(kinesis, WRITE_STREAM, _)).map(_ => makeResponse(messages))

  def makeResponse(nbMessages: Int): APIGatewayV2HTTPResponse =
    APIGatewayV2HTTPResponse.builder()
      .withStatusCode(200)
      .withHeaders(Map("content-type" -> "text/html").asJava)
      .withBody(s"Posted $nbMessages messages to Kinesis")
      .build()

  def handleRequest(event: Request[_], ctx: Context): APIGatewayV2HTTPResponse =
    import cats.effect.unsafe.implicits.global
    val prog = for {
      kinesis <- Kinesis.kinesisClient
      response <- handleRequestIO(kinesis, event)
    } yield response
    prog.unsafeRunSync()


}
