package com.xebia.lambda

import java.nio.ByteBuffer
import java.nio.charset.Charset
import java.util.UUID

import cats.effect.Sync, cats.effect.std.Random
import cats.implicits.*

import io.circe.*, io.circe.syntax.*,  io.circe.generic.auto.*
case class Datum (uuid: UUID, doc: String, hashes: Int, hash: Option[String])

object Datum {
  def random[F[_]: Sync: Logger](size: Int, hashes: Int): F[Datum] =
    for
      rng <- Random.scalaUtilRandom[F]
      doc <- (0 until size).toList.traverse(_ => rng.nextPrintableChar).map(_.mkString)
      datum = Datum(UUID.randomUUID(), s"$doc", hashes, None)
      _ <- Logger[F].trace(s"Generated datum: $datum")
    yield datum

  def serialize(datum: Datum): ByteBuffer =
    ByteBuffer.wrap(datum.asJson.noSpaces.getBytes(Charset.forName("UTF-8")))
}
