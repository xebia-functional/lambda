val scala3Version = "3.3.1"




lazy val commonSettings: Seq[Setting[_]] = Seq(
  libraryDependencies ++= Seq(
    "com.amazonaws" % "aws-lambda-java-core" % "1.2.3",
    "com.amazonaws" % "aws-lambda-java-events" % "3.11.4",
    "com.amazonaws" % "amazon-kinesis-client" % "1.15.0",
    "org.typelevel" %% "cats-effect" % "3.5.2",
    "org.scalameta" %% "munit" % "0.7.29" % Test,
  ),
  assemblyMergeStrategy :=
    {
      case PathList("META-INF", xs @ _*) => MergeStrategy.discard
      case x => MergeStrategy.first
    }
)

lazy val circeSettings = Seq(
  libraryDependencies ++= Seq("io.circe" %% "circe-core",
  "io.circe" %% "circe-generic",
  "io.circe" %% "circe-parser"
).map(_ % "0.15.0-M1"))


inThisBuild(Seq(

  version := "0.1.0-SNAPSHOT",

  scalaVersion := scala3Version,
))

lazy val root = project
  .in(file("."))
  .settings(
    name := "lambda"
  ).aggregate(httpA, eventsA, eventsB)

lazy val data = project
  .settings(commonSettings ++ circeSettings)
  .in(file("data"))

lazy val httpA = project
  .in(file("http-a"))
  .dependsOn(data)

lazy val eventsA = project
  .in(file("events-a"))
  .settings(
    libraryDependencies += "pt.kcry" %% "sha" % "2.0.1"
  )
  .dependsOn(data)

lazy val eventsB = project
  .in(file("events-b"))
  .dependsOn(data)
