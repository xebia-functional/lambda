package com.xebia.lambda

import cats.effect.IO
import io.circe.jawn.JawnParser
import munit.ScalaCheckSuite
import org.scalacheck.{Arbitrary, Gen}
import org.scalacheck.Prop.*
import io.circe.*
import io.circe.generic.auto.*
import cats.effect.unsafe.implicits.global
class DatumSuite extends ScalaCheckSuite {

  given Logger[IO] = new Logger[IO]{

    override def info(msg: String): IO[Unit] = IO.unit

    override def debug(msg: String): IO[Unit] = IO.unit

    override def trace(msg: String): IO[Unit] = IO.unit
  }

  given Arbitrary[Datum] = Arbitrary(Gen.choose(1, 1024).map(s => Datum.random[IO](s, 0).unsafeRunSync()))

  property("serialization round-trip"){
    val parser = new JawnParser()
    forAll { (datum: Datum) =>
      parser.parseByteBuffer(Datum.serialize(datum)).flatMap(_.as[Datum]) == Right(datum)
    }
  }
}
