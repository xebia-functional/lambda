package com.xebia.lambda

import cats.effect.IO
import com.amazonaws.services.lambda.runtime.LambdaLogger
import com.amazonaws.services.lambda.runtime.logging.LogLevel

object Logger {

  def ioLogger(lambdaLogger: LambdaLogger): Logger[IO] = new Logger[IO]{
    override def info(msg: String): IO[Unit] = IO.delay(lambdaLogger.log(msg, LogLevel.INFO))

    override def debug(msg: String): IO[Unit] = IO.delay(lambdaLogger.log(msg, LogLevel.DEBUG))

    override def trace(msg: String): IO[Unit] = IO.delay(lambdaLogger.log(msg, LogLevel.TRACE))

  }

  def apply[F[_]](using F: Logger[F]): Logger[F] = F
}


trait Logger[F[_]] {
  def info(msg: String): F[Unit]
  def debug(msg: String): F[Unit]
  def trace(msg: String): F[Unit]
}

