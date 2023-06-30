import { Graph } from './graph';
import { Config, ConnectionType, DsnpVersion, PrivacyType, SchemaConfig } from './models';
import { DevEnvironment, EnvironmentType } from './models/environment';


function getTestConfig(): Config {
    const config: Config = {} as Config;
    config.sdkMaxStaleFriendshipDays = 100;
    config.maxPageId = 100;
    config.dsnpVersions = [DsnpVersion.Version1_0];
    config.maxGraphPageSizeBytes = 100;
    config.maxKeyPageSizeBytes = 100;
    const schemaConfig = {} as SchemaConfig;
    schemaConfig.dsnpVersion = DsnpVersion.Version1_0;
    schemaConfig.connectionType = ConnectionType.Follow;
    schemaConfig.privacyType = PrivacyType.Public;
    config.schemaMap = { 1: schemaConfig };
    return config;
}

test('printHelloGraph should print "Hello, Graph!"', async () => {
    // Mock the console.log function
    const consoleLogMock = jest.spyOn(console, 'log').mockImplementation();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: getTestConfig()};
    const graph = new Graph(environment);
    await graph.printHelloGraph();
    expect(consoleLogMock).toHaveBeenCalledWith('Hello, Graph!');
    await graph.freeGraphState();
});
