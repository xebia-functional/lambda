package com.xebia.lambda
package httpA

import cats.effect.{ExitCode, IO, IOApp}
import cats.effect.std.{Env, Random}
import cats.implicits.*
import com.amazonaws.Request
import com.amazonaws.services.kinesis.model.{
  PutRecordsRequest,
  PutRecordsRequestEntry,
  PutRecordsResult
}
import com.amazonaws.services.kinesis.{
  AmazonKinesisAsync,
  AmazonKinesisAsyncClientBuilder
}
import com.amazonaws.services.lambda.runtime.{
  Context,
  LambdaLogger,
  RequestHandler
}
import com.amazonaws.services.lambda.runtime.events.{
  APIGatewayV2HTTPEvent,
  APIGatewayV2HTTPResponse
}

import scala.util.Try
import scala.jdk.CollectionConverters.*

object HttpAApp
    extends RequestHandler[APIGatewayV2HTTPEvent, APIGatewayV2HTTPResponse] {

  /// The name of the query parameter that specifies the number of random
  /// characters to generate.
  val LENGTH_PARAM = "chars"

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

  private def getParameterOrDefault(
      event: APIGatewayV2HTTPEvent,
      key: String,
      default: Int
  )(using logger: Logger[IO]): IO[Int] =
    for
      queryParams <- IO.pure(Option(event.getQueryStringParameters.asScala))
      _           <- logger.trace(s"Query parameters: $queryParams")
      param       <- IO.pure(queryParams.flatMap(_.get(key)))
      value       <- IO.pure(param.flatMap(p => Try(p.toInt).toOption))
    yield value.getOrElse(default)

  def handleRequestIO(
      kinesis: AmazonKinesisAsync,
      event: APIGatewayV2HTTPEvent
  )(using log: Logger[IO]): IO[APIGatewayV2HTTPResponse] =
    for
      chars        <- getParameterOrDefault(event, LENGTH_PARAM, 1024)
      hashes       <- getParameterOrDefault(event, HASHES_PARAM, 100)
      messages     <- getParameterOrDefault(event, MESSAGES_PARAM, 64)
      _            <- log.debug(s"chars=$chars, hashes=$hashes, messages=$messages")
      stream       <- Env[IO].get(WRITE_STREAM)
      data         <- List
                        .fill(messages)((chars, hashes))
                        .traverse(Datum.random[IO].tupled)
      _            <- log.trace(s"Generated messages: ${data.size}")
      nbPutRecords <- Kinesis.postData(kinesis, stream.get, data)
      _            <- log.debug(s"Posted messages: $nbPutRecords")
    yield makeResponse(nbPutRecords)

  def makeResponse(nbMessages: Int): APIGatewayV2HTTPResponse =
    APIGatewayV2HTTPResponse
      .builder()
      .withStatusCode(200)
      .withHeaders(Map("content-type" -> "text/html").asJava)
      .withBody(s"Posted $nbMessages messages to Kinesis")
      .build()

  def handleRequest(
      event: APIGatewayV2HTTPEvent,
      ctx: Context
  ): APIGatewayV2HTTPResponse =
    import cats.effect.unsafe.implicits.global
    given log: Logger[IO] = Logger.ioLogger(ctx.getLogger)
    val prog              =
      for
        _        <- log.debug(s"Received request $event")
        kinesis  <- Kinesis.kinesisClient
        response <- handleRequestIO(kinesis, event)
        _        <- log.trace(s"Responded with $response")
      yield response
    prog.unsafeRunSync()

}
