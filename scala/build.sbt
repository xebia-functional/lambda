val scala3Version = "3.3.1"

lazy val commonSettings: Seq[Setting[_]] = Seq(
  libraryDependencies ++= Seq(
    "com.amazonaws"  % "aws-lambda-java-core"   % "1.2.3",
    "com.amazonaws"  % "aws-lambda-java-events" % "3.11.4",
    "com.amazonaws"  % "amazon-kinesis-client"  % "1.15.0",
    "org.typelevel" %% "cats-effect"            % "3.5.2",
    "org.scalameta" %% "munit"                  % "0.7.29" % Test,
    "org.scalameta" %% "munit-scalacheck"       % "0.7.29" % Test
  )
)

inThisBuild(
  assemblyMergeStrategy := {
    case PathList("META-INF", _)               => MergeStrategy.discard
    case PathList("module-info.class")         => MergeStrategy.discard
    case x if x.endsWith("/module-info.class") => MergeStrategy.discard
    case x                                     => MergeStrategy.first
  }
)
lazy val circeSettings = Seq(
  libraryDependencies ++= Seq(
    "io.circe" %% "circe-core",
    "io.circe" %% "circe-generic",
    "io.circe" %% "circe-parser"
  ).map(_ % "0.15.0-M1")
)

inThisBuild(
  Seq(
    version      := "0.1.0-SNAPSHOT",
    scalaVersion := scala3Version
  )
)

lazy val root = project
  .in(file("."))
  .settings(
    name := "lambda"
  )
  .aggregate(common, httpA, eventsA, eventsB)

lazy val common = project
  .settings(commonSettings ++ circeSettings)
  .in(file("common"))

lazy val httpA = project
  .in(file("http-a"))
  .dependsOn(common)

lazy val eventsA = project
  .in(file("events-a"))
  .dependsOn(common)

lazy val eventsB = project
  .in(file("events-b"))
  .dependsOn(common)
