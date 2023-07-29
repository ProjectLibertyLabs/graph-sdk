import { Graph } from "./graph";
import { Config, ConnectionType, DsnpVersion, PrivacyType } from "./models";
import { DevEnvironment, EnvironmentType } from "./models/environment";

const config: Config = {
  sdkMaxStaleFriendshipDays: 100,
  maxPageId: 100,
  dsnpVersions: [DsnpVersion.Version1_0],
  maxGraphPageSizeBytes: 100,
  maxKeyPageSizeBytes: 100,
  schemaMap: {
    1: {
      dsnpVersion: DsnpVersion.Version1_0,
      connectionType: ConnectionType.Follow,
      privacyType: PrivacyType.Public,
    },
  },
  graphPublicKeySchemaId: 11,
};

test('printHelloGraph should print "Hello, Graph!"', async () => {
  // Mock the console.log function
  const consoleLogMock = jest.spyOn(console, "log").mockImplementation();
  const environment: DevEnvironment = {
    environmentType: EnvironmentType.Dev,
    config,
  };
  const graph = new Graph(environment);
  graph.printHelloGraph();
  expect(consoleLogMock).toHaveBeenCalledWith("Hello, Graph!");
  graph.freeGraphState();
});
