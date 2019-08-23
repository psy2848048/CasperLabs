package io.casperlabs.casper.helper

import java.nio.ByteBuffer
import java.nio.file.{Files, Path}
import java.util.zip.CRC32

import cats.effect._
import cats.implicits._
import com.google.protobuf.ByteString
import doobie.util.ExecutionContexts
import doobie.util.transactor.Transactor
import io.casperlabs.casper.consensus.Block
import io.casperlabs.catscontrib.TaskContrib.TaskOps
import io.casperlabs.metrics.Metrics
import io.casperlabs.metrics.Metrics.MetricsNOP
import io.casperlabs.shared.Log
import io.casperlabs.shared.PathOps.RichPath
import io.casperlabs.storage._
import io.casperlabs.storage.block._
import io.casperlabs.storage.dag.DagRepresentation.Validator
import io.casperlabs.storage.dag._
import monix.eval.Task
import monix.execution.Scheduler
import org.flywaydb.core.Flyway
import org.flywaydb.core.api.Location
import org.lmdbjava.{Env, EnvFlags}
import org.scalatest.{BeforeAndAfter, Suite}

trait DagStorageFixture extends BeforeAndAfter { self: Suite =>
  val scheduler = Scheduler.fixedPool("dag-storage-fixture-scheduler", 4)

  def withStorage[R](f: BlockStorage[Task] => IndexedDagStorage[Task] => Task[R]): R = {
    val testProgram = Sync[Task].bracket {
      Sync[Task].delay {
        (DagStorageTestFixture.dagStorageDir, DagStorageTestFixture.blockStorageDir)
      }
    } {
      case (dagStorageDir, blockStorageDir) =>
        implicit val metrics = new MetricsNOP[Task]()
        implicit val log     = new Log.NOPLog[Task]()
        for {
          blockStorage      <- DagStorageTestFixture.createBlockStorage[Task](blockStorageDir)
          dagStorage        <- DagStorageTestFixture.createDagStorage[Task](dagStorageDir)
          indexedDagStorage <- IndexedDagStorage.create(dagStorage)
          result            <- f(blockStorage)(indexedDagStorage)
        } yield result
    } {
      case (dagStorageDir, blockStorageDir) =>
        Sync[Task].delay {
          dagStorageDir.recursivelyDelete()
          blockStorageDir.recursivelyDelete()
        }
    }
    testProgram.unsafeRunSync(scheduler)
  }
}

object DagStorageTestFixture {
  def dagStorageDir: Path   = Files.createTempDirectory("casper-dag-storage-test-")
  def blockStorageDir: Path = Files.createTempDirectory("casper-block-storage-test-")

  def writeInitialLatestMessages(
      latestMessagesData: Path,
      latestMessagesCrc: Path,
      latestMessages: Map[Validator, Block]
  ): Unit = {
    val data = latestMessages
      .foldLeft(ByteString.EMPTY) {
        case (byteString, (validator, block)) =>
          byteString.concat(validator).concat(block.blockHash)
      }
      .toByteArray
    val crc = new CRC32()
    latestMessages.foreach {
      case (validator, block) =>
        crc.update(validator.concat(block.blockHash).toByteArray)
    }
    val crcByteBuffer = ByteBuffer.allocate(8)
    crcByteBuffer.putLong(crc.getValue)
    Files.write(latestMessagesData, data)
    Files.write(latestMessagesCrc, crcByteBuffer.array())
  }

  def env(
      path: Path,
      mapSize: Long,
      flags: List[EnvFlags] = List(EnvFlags.MDB_NOTLS)
  ): Env[ByteBuffer] =
    Env
      .create()
      .setMapSize(mapSize)
      .setMaxDbs(8)
      .setMaxReaders(126)
      .open(path.toFile, flags: _*)

  val mapSize: Long = 1024L * 1024L * 100L

  def createBlockStorage[F[_]: Concurrent: Metrics: Log](
      blockStorageDir: Path
  ): F[BlockStorage[F]] = {
    val env = Context.env(blockStorageDir, mapSize)
    FileLMDBIndexBlockStorage.create[F](env, blockStorageDir).map(_.right.get)
  }

  //TODO: Use single database for all storages when we have SQLiteBlockStorage
  //https://casperlabs.atlassian.net/browse/NODE-663
  def createDagStorage[F[_]: Metrics: Async: ContextShift](
      dagStorageDir: Path,
      maybeGenesis: Option[Block] = None
  ): F[DagStorage[F]] = {
    val jdbcUrl = s"jdbc:sqlite:${dagStorageDir.resolve("dag_storage.db")}"
    val flyway = {
      val conf =
        Flyway
          .configure()
          .dataSource(jdbcUrl, "", "")
          .locations(new Location("classpath:/db/migration"))
      conf.load()
    }
    implicit val xa = Transactor
      .fromDriverManager[F](
        "org.sqlite.JDBC",
        jdbcUrl,
        "",
        "",
        ExecutionContexts.synchronous
      )

    SQLiteDagStorage[F].use(
      dagStorage =>
        for {
          _ <- Sync[F].delay(flyway.migrate)
          _ <- maybeGenesis.fold(().pure[F])(genesis => dagStorage.insert(genesis).void)
        } yield dagStorage
    )
  }
}
