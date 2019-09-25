package io.casperlabs.node.configuration

import cats._
import cats.syntax._
import cats.implicits._
import cats.data.{Validated, ValidatedNel}
import com.google.protobuf.ByteString
import com.google.common.io.Resources
import eu.timepit.refined._
import eu.timepit.refined.api.Refined
import eu.timepit.refined.numeric._
import io.casperlabs.casper.consensus.state
import io.casperlabs.configuration.SubConfig
import io.casperlabs.crypto.Keys.PublicKey
import io.casperlabs.crypto.codec.Base64
import io.casperlabs.ipc
import java.io.{ByteArrayOutputStream, File}
import java.nio.file.{Files, Path, Paths}
import java.util.stream.Collectors
import org.apache.commons.io.IOUtils
import scala.io.Source
import scala.util.Try
import scala.collection.JavaConverters._

/**
  * ChainSpec is the definition of the chain which we use for the Genesis process.
  * https://casperlabs.atlassian.net/wiki/spaces/EN/pages/133529693/Genesis+process+design+doc
  */
object ChainSpec extends ParserImplicits {

  class ConfCompanion[T](confParser: ConfParser[T]) {
    def parseManifest(manifest: => Source): ValidatedNel[String, T] =
      Utils.readFile(manifest).toValidatedNel[String].andThen { raw =>
        confParser
          .parse(
            cliByName = _ => None,
            envVars = Map.empty,
            configFile = None,
            defaultConfigFile = Utils.parseToml(raw),
            pathToField = Nil
          )
      }
  }

  /** The first set of changes should define the Genesis section and the costs. */
  final case class GenesisConf(
      genesis: Genesis,
      wasmCosts: WasmCosts
  )
  object GenesisConf extends ConfCompanion[GenesisConf](ConfParser.gen[GenesisConf])

  /** Subsequent changes describe upgrades. */
  final case class UpgradeConf(
      upgrade: Upgrade,
      wasmCosts: Option[WasmCosts]
  )
  object UpgradeConf extends ConfCompanion[UpgradeConf](ConfParser.gen[UpgradeConf])

  final case class Genesis(
      name: String,
      timestamp: Long,
      mintCodePath: Path,
      posCodePath: Path,
      initialAccountsPath: Path,
      // TODO: Change this later to semver.
      protocolVersion: Long
  ) extends SubConfig

  final case class Upgrade(
      activationPointRank: Long,
      installerCodePath: Option[Path],
      // TODO: Change this later to semver.
      protocolVersion: Long
  ) extends SubConfig

  final case class WasmCosts(
      regular: Int Refined NonNegative,
      divMultiplier: Int Refined NonNegative,
      mulMultiplier: Int Refined NonNegative,
      memMultiplier: Int Refined NonNegative,
      memInitialPages: Int Refined NonNegative,
      memGrowPerPage: Int Refined NonNegative,
      memCopyPerByte: Int Refined NonNegative,
      maxStackHeight: Int Refined NonNegative,
      opcodesMultiplier: Int Refined NonNegative,
      opcodesDivisor: Int Refined Positive
  ) extends SubConfig

  final case class Account(
      publicKey: PublicKey,
      initialBalance: BigInt,
      initialBondedAmount: BigInt
  )

  object Accounts {
    def parseCsv(raw: String, skipHeader: Boolean = false): Either[String, List[Account]] =
      raw
        .split('\n')
        .drop(if (skipHeader) 1 else 0)
        .filterNot(_.isEmpty)
        .map { line =>
          line.split(',') match {
            case Array(publicKeyBase64, balanceStr, bondedAmountStr) =>
              for {
                publicKey    <- parsePublicKey(publicKeyBase64)
                balance      <- parseBigInt(balanceStr)
                bondedAmount <- parseBigInt(bondedAmountStr)
              } yield Account(publicKey, balance, bondedAmount)

            case _ =>
              s"Could not parse line into an Account: $line".asLeft[Account]
          }
        }
        .toList
        .sequence

    private def parsePublicKey(publicKeyBase64: String) =
      Base64.tryDecode(publicKeyBase64) match {
        case None        => s"Could not decode public key as Base64: $publicKeyBase64".asLeft[PublicKey]
        case Some(bytes) => PublicKey(bytes).asRight[String]
      }

    private def parseBigInt(amount: String) =
      Try(BigInt(amount)).fold(
        _ => s"Could not parse amount: $amount".asLeft[BigInt],
        i => i.asRight[String]
      )
  }

  /** Normally we expect files to be relative to the directory where the update is,
    * but it's possible someone would locally want to re-point it to an absolute path.
    */
  def resolvePath(dir: Path, file: Path): Path =
    if (file.startsWith(Paths.get("~/")))
      Paths.get(sys.props("user.home")).resolve(file.toString.drop(2))
    else dir.resolve(file)

  def withManifest[A, B](dir: Path, parseManifest: (=> Source) => ValidatedNel[String, A])(
      read: A => Either[String, B]
  ): ValidatedNel[String, B] = {
    val manifest = new File(dir.toFile, "manifest.toml")
    if (!manifest.exists)
      Validated.invalidNel(s"Manifest file '$manifest' is missing!")
    else {
      parseManifest(Source.fromFile(manifest)) andThen { conf =>
        read(conf)
          .leftMap(err => s"Could not read chainspec sub-directory $dir: $err")
          .toValidatedNel
      }
    }
  }

  /** List the files contained in the resources directory packaged in the JAR. */
  def listFilesInResources(dir: Path): List[Path] = {
    val chainspecRoot = Paths.get(Resources.getResource(dir.toString).getPath)
    Files
      .list(chainspecRoot)
      .map[Path](chainspecRoot.getParent.relativize(_))
      .collect(Collectors.toList[Path]())
      .asScala
      .sorted
      .toList
  }
}

/** Resolve a path to its contents. */
trait Resolver {
  def asBytes(path: Path): Either[String, Array[Byte]]
  def asSource(path: Path): Either[String, Source]
  def asString(path: Path): Either[String, String]
}

/** Resolve to normal files. */
object FileResolver extends Resolver {
  override def asBytes(path: Path) =
    Utils.readBytes(path)

  override def asSource(path: Path) =
    if (!path.toFile.exists)
      s"File '$path' is missing!".asLeft[Source]
    else
      Source.fromFile(path.toFile).asRight[String]

  override def asString(path: Path) =
    Utils.readFile(path)
}

/** Resolve paths in resources, unless an override in the data directory exists. */
class ResourceResolver(dataDir: Path) extends Resolver {
  override def asBytes(path: Path) =
    read(path, readResourceBytes, FileResolver.asBytes)

  override def asSource(path: Path) =
    read(path, x => Source.fromResource(x.toString), FileResolver.asSource)

  override def asString(path: Path) =
    asSource(path).flatMap(src => Utils.readFile(src))

  private def read[T](
      path: Path,
      fromResource: Path => T,
      fromFile: Path => Either[String, T]
  ): Either[String, T] =
    if (path.isAbsolute) {
      fromFile(path)
    } else {
      val over = dataDir.resolve(path).toFile
      if (over.exists) fromFile(over.toPath)
      else
        Try(fromResource(path)).fold(
          ex => s"Cannot read resource $path: ${ex.getMessage}".asLeft[T],
          x => x.asRight[String]
        )
    }

  private def readResourceBytes(path: Path) = {
    val in  = this.getClass.getResourceAsStream(path.toString)
    val out = new ByteArrayOutputStream()
    try {
      IOUtils.copy(in, out)
      out.toByteArray
    } finally {
      in.close()
      out.close()
    }
  }
}

/** Adds extension methods to ipc.ChainSpec so it can be read from various sources. */
trait ChainSpecReader {
  import ChainSpec._

  implicit class ChainSpecOps(typ: ipc.ChainSpec.type) {

    /** If there's no explicit ChainSpec location defined we can use the default one
      * packaged with the node. Every file can be overridden by placing one with the
      * same path under the ~/.casperlabs data directory.
      */
    def fromResources(resourcePath: Path, dataDir: Path): ValidatedNel[String, ipc.ChainSpec] = {
      val changesets = listFilesInResources(resourcePath)
      fromChangesets(resourcePath, changesets)(new ResourceResolver(dataDir))
    }

    /** Parse and read the contents of a chainspec directory into the IPC DTOs. */
    def fromDirectory(path: Path): ValidatedNel[String, ipc.ChainSpec] = {
      val dir = path.toFile
      if (!dir.exists)
        Validated.invalidNel(s"Chain spec directory '$path' does not exist!")
      else if (!dir.isDirectory)
        Validated.invalidNel(s"Chain spec path '$path' is not a directory!")
      else {
        // Consider each subdirectory an upgrade, starting with Genesis.
        val changesets = dir.listFiles.sortBy(_.getName).map(_.toPath).toList
        fromChangesets(path, changesets)(FileResolver)
      }
    }

    private def fromChangesets(
        path: Path,
        changesets: List[Path]
    )(implicit resolver: Resolver): ValidatedNel[String, ipc.ChainSpec] =
      changesets match {
        case Nil =>
          Validated.invalidNel(s"Chain spec directory '$path' is empty!")

        case genesisDir :: upgradeDirs =>
          val genesis = ipc.ChainSpec.GenesisConfig.fromChangeset(genesisDir)
          val upgrades = upgradeDirs.map { dir =>
            ipc.ChainSpec.UpgradePoint.fromChangeset(dir)
          }

          genesis andThen { g =>
            upgrades.sequence map { us =>
              ipc.ChainSpec().withGenesis(g).withUpgrades(us)
            }
          }
      }
  }

  implicit class GenesisConfigOps(typ: ipc.ChainSpec.GenesisConfig.type) {
    def fromChangeset(
        path: Path
    )(implicit resolver: Resolver): ValidatedNel[String, ipc.ChainSpec.GenesisConfig] =
      withManifest[GenesisConf, ipc.ChainSpec.GenesisConfig](path, GenesisConf.parseManifest) {
        case GenesisConf(genesis, wasmCosts) =>
          for {
            mintCodeBytes <- resolver.asBytes(resolvePath(path, genesis.mintCodePath))
            posCodeBytes  <- resolver.asBytes(resolvePath(path, genesis.posCodePath))
            accountsCsv   <- resolver.asString(resolvePath(path, genesis.initialAccountsPath))
            accounts      <- Accounts.parseCsv(accountsCsv, skipHeader = false)
          } yield {
            ipc.ChainSpec
              .GenesisConfig()
              .withName(genesis.name)
              .withTimestamp(genesis.timestamp)
              .withProtocolVersion(state.ProtocolVersion(genesis.protocolVersion))
              .withMintInstaller(ByteString.copyFrom(mintCodeBytes))
              .withPosInstaller(ByteString.copyFrom(posCodeBytes))
              .withAccounts(accounts.map { account =>
                ipc.ChainSpec
                  .GenesisAccount()
                  .withPublicKey(ByteString.copyFrom(account.publicKey))
                  .withBalance(state.BigInt(account.initialBalance.toString, bitWidth = 512))
                  .withBondedAmount(
                    state.BigInt(account.initialBondedAmount.toString, bitWidth = 512)
                  )
              })
              .withCosts(ipc.ChainSpec.CostTable.fromConfig(wasmCosts))
          }
      }
  }

  implicit class UpgradePointOps(typ: ipc.ChainSpec.UpgradePoint.type) {
    def fromChangeset(
        path: Path
    )(implicit resolver: Resolver): ValidatedNel[String, ipc.ChainSpec.UpgradePoint] =
      withManifest[UpgradeConf, ipc.ChainSpec.UpgradePoint](path, UpgradeConf.parseManifest) {
        case UpgradeConf(upgrade, maybeWasmCosts) =>
          upgrade.installerCodePath.fold(
            none[Array[Byte]].asRight[String]
          ) { file =>
            resolver.asBytes(resolvePath(path, file)).map(_.some)
          } map { maybeInstallerCodeBytes =>
            ipc.ChainSpec
              .UpgradePoint(
                upgradeInstaller = maybeInstallerCodeBytes.map { bytes =>
                  ipc.DeployCode(
                    code = ByteString.copyFrom(bytes)
                  )
                },
                newCosts = maybeWasmCosts.map { wasmCosts =>
                  ipc.ChainSpec.CostTable.fromConfig(wasmCosts)
                }
              )
              .withActivationPoint(
                ipc.ChainSpec.ActivationPoint(upgrade.activationPointRank)
              )
              .withProtocolVersion(state.ProtocolVersion(upgrade.protocolVersion))
          }
      }
  }

  implicit class CostTableOps(typ: ipc.ChainSpec.CostTable.type) {
    def fromConfig(wasmCosts: WasmCosts): ipc.ChainSpec.CostTable =
      ipc.ChainSpec
        .CostTable()
        .withWasm(
          ipc.ChainSpec.CostTable
            .WasmCosts()
            .withRegular(wasmCosts.regular.value)
            .withDiv(wasmCosts.divMultiplier.value)
            .withMul(wasmCosts.mulMultiplier.value)
            .withMem(wasmCosts.memMultiplier.value)
            .withInitialMem(wasmCosts.memInitialPages.value)
            .withGrowMem(wasmCosts.memGrowPerPage.value)
            .withMemcpy(wasmCosts.memCopyPerByte.value)
            .withMaxStackHeight(wasmCosts.maxStackHeight.value)
            .withOpcodesMul(wasmCosts.opcodesMultiplier.value)
            .withOpcodesDiv(wasmCosts.opcodesDivisor.value)
        )
  }
}

object ChainSpecReader extends ChainSpecReader
