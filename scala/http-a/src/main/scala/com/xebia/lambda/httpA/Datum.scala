package com.xebia.lambda.httpA

import cats.Functor
import cats.effect.kernel.Sync
import cats.effect.std.Random
import cats.implicits._

import java.nio.ByteBuffer
import java.nio.charset.Charset
import java.util.UUID

case class Datum (uuid: UUID, doc: String, hashes: Int, hash: Option[String])

object Datum {
  def random[F[_]: Sync](size: Int, hashes: Int): F[Datum] =
    Random.scalaUtilRandom[F].flatMap(_.nextString(size)).map(doc => Datum(UUID.randomUUID(), doc, hashes, None))

  def serialize(datum: Datum): ByteBuffer =
    val jsonString = s"""{"uuid":"${datum.uuid}","doc":"${datum.doc}","hashes":${datum.hashes},"hash":${datum.hash.map("\""+ _ +"\"").getOrElse("null")}}"""
    ByteBuffer.wrap(jsonString.getBytes(Charset.forName("UTF-8")))
}
