import { Config } from "./config";

enum EnvironmentType {
  Mainnet = "Mainnet",
  TestnetPaseo = "TestnetPaseo",
  Rococo = "Rococo",
  Dev = "Dev",
}

interface EnvironmentInterface {
  environmentType: EnvironmentType;
}

interface Environment extends EnvironmentInterface {
  environmentType: EnvironmentType.Mainnet | EnvironmentType.Rococo | EnvironmentType.TestnetPaseo;
}

interface DevEnvironment extends EnvironmentInterface {
  environmentType: EnvironmentType.Dev;
  config: Config;
}

type EnvironmentConfig = DevEnvironment;

export { EnvironmentType, Environment, DevEnvironment, EnvironmentConfig, EnvironmentInterface };
