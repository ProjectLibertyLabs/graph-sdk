import exp from 'constants';
import { Graph } from './graph';
import { PageData, GraphKeyPair, DsnpKeys, ImportBundle, Action, ConnectAction, Connection} from './models';
import { Config, ConnectionType, DsnpVersion, PrivacyType, SchemaConfig } from './models/config';
import { DevEnvironment, EnvironmentInterface, EnvironmentType } from './models/environment';


function getTestConfig(): Config {
    const config: Config = {} as Config;
    config.sdkMaxUsersGraphSize = 100;
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
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    await graph.printHelloGraph();
    expect(consoleLogMock).toHaveBeenCalledWith('Hello, Graph!');
    await graph.freeGraphState();
});

test('getGraphConfig should return the graph config', async () => {
    const config_input = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config: config_input};
    const graph = new Graph(environment);
    const config = await graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.sdkMaxUsersGraphSize).toEqual(100);
    await graph.freeGraphState();
});

test('getGraphConfig with Mainnet environment should return the graph config', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    const config = await graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.sdkMaxUsersGraphSize).toEqual(1000);
    const schema_id = await graph.getSchemaIdFromConfig(environment, ConnectionType.Follow, PrivacyType.Public);
    expect(schema_id).toEqual(1);
    await graph.freeGraphState();
});

test('getGraphConfig with Rococo environment should return the graph config', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Rococo };
    const graph = new Graph(environment);
    const config = await graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.sdkMaxUsersGraphSize).toEqual(1000);
    await graph.freeGraphState();
});

test('initialize graph with low capacity of 100 should return the same capacity', async () => {
    const config = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config};
    const graph = new Graph(environment, 100);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const capacity = await graph.getGraphCapacity();
    expect(capacity).toEqual(100);
    await graph.freeGraphState();
});

test('getGraphStatesCount should be zero after previous graph is freed', async () => {
    const config = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config};
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    await graph.freeGraphState();
    const count = await graph.getGraphStatesCount();
    expect(count).toEqual(0);
});

test('getGraphStatesCount should be one after graph is initialized', async () => {
    const config = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config};
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const count = await graph.getGraphStatesCount();
    expect(count).toEqual(1);
    await graph.freeGraphState();
});

test('getGraphUsersCount should be zero on initialized graph', async () => {
    const config = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config};
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const count = await graph.getGraphUsersCount();
    expect(count).toEqual(0);
    await graph.freeGraphState();
    await expect(async () => {
      await graph.getGraphUsersCount();
    }).rejects.toThrow('Graph state not found');
});

test('containsUserGraph should return false on initialized graph', async () => {
    const config = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config};
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const contains = await graph.containsUserGraph(1);
    expect(contains).toEqual(false);
    await graph.freeGraphState();
});

test('removeUserGraph should pass through on initialized graph', async () => {
    const config = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config};
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const removed = await graph.removeUserGraph(1);
    expect(removed).toEqual(true);
    await graph.freeGraphState();
});

test('importUserData should pass through on initialized graph', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    // Set up import data
    const dsnpUserId1 = 1;
    const dsnpUserId2 = 2;

    const pageData1: PageData = {
      pageId: 1,
      content: new Uint8Array([24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0]),
      contentHash: 100,
    };

    const keyPairs1: GraphKeyPair[] = [];
    const keyPairs2: GraphKeyPair[] = [];

    const dsnpKeys1: DsnpKeys = {
      dsnpUserId: dsnpUserId1,
      keysHash: 100,
      keys: [],
    };

    const dsnpKeys2: DsnpKeys = {
      dsnpUserId: dsnpUserId2,
      keysHash: 100,
      keys: [],
    };

    const importBundle1: ImportBundle = {
      dsnpUserId: dsnpUserId1,
      schemaId: 1,
      keyPairs: keyPairs1,
      dsnpKeys: dsnpKeys1,
      pages: [pageData1],
    };

    const importBundle2: ImportBundle = {
      dsnpUserId: dsnpUserId2,
      schemaId: 1,
      keyPairs: keyPairs2,
      dsnpKeys: dsnpKeys2,
      pages: [pageData1],
    };

    // Import user data for each ImportBundle
    const imported = await graph.importUserData([importBundle1, importBundle2]);
    expect(imported).toEqual(true);
    await graph.freeGraphState();
});

test('applyActions with empty actions should pass through on initialized graph', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    // Set up actions
    const actions = [] as Action[];
    const applied = await graph.applyActions(actions);
    expect(applied).toEqual(true);
    await graph.freeGraphState();
});

test('applyActions with few actions should pass through on initialized graph', async () => {
    const config = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config};
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    // Set up actions
    const actions = [] as Action[];
    const action_1 = {
        type: "Connect",
        ownerDsnpUserId: 1,
        connection: {
            dsnpUserId: 2,
            schemaId: 1,
        } as Connection,
        dsnpKeys: {
          dsnpUserId: 2,
          keysHash: 100,
          keys: [],
        } as DsnpKeys,
    } as ConnectAction;

    actions.push(action_1);
    const applied = await graph.applyActions(actions);
    expect(applied).toEqual(true);

    const exported = await graph.exportUpdates();
    expect(exported).toBeDefined();
    expect(exported.length).toEqual(1);
    
    await graph.freeGraphState();
});

test('getConnectionsForUserGraph with empty connections should return empty array', async () => {
    const config = getTestConfig();
    const environment: DevEnvironment = { environmentType: EnvironmentType.Dev, config};
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    // Set up actions
    const actions = [] as Action[];
    const action_1 = {
        type: "Connect",
        ownerDsnpUserId: 1,
        connection: {
            dsnpUserId: 2,
            schemaId: 1,
        } as Connection,
        dsnpKeys: {
          dsnpUserId: 2,
          keysHash: 100,
          keys: [],
        } as DsnpKeys,
    } as ConnectAction;

    actions.push(action_1);
    const applied = await graph.applyActions(actions);
    expect(applied).toEqual(true);
    const connections = await graph.getConnectionsForUserGraph(1, 1, true);
    expect(connections).toBeDefined();
    expect(connections.length).toEqual(1);

    const forceCalculateGraphs = await graph.forceCalculateGraphs(1);
    expect(forceCalculateGraphs).toBeDefined();
    expect(forceCalculateGraphs.length).toEqual(0);
    await graph.freeGraphState();
});

test('getConnectionsWithoutKeys with empty connections should return empty array', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const connections = await graph.getConnectionsWithoutKeys();
    expect(connections).toBeDefined();
    expect(connections.length).toEqual(0);

    expect(async () => {
        await graph.getOneSidedPrivateFriendshipConnections(1);
    }).rejects.toThrow('User graph for 1 is not imported');

    await graph.freeGraphState();
});

test('getPublicKeys with empty connections should return empty array', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const connections = await graph.getPublicKeys(1);
    expect(connections).toBeDefined();
    expect(connections.length).toEqual(0);
    await graph.freeGraphState();
});

test('deserializeDsnpKeys with empty keys should return empty array', async () => {
    const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };
    const graph = new Graph(environment);
    const handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
    const keys = {          
        dsnpUserId: 2,
        keysHash: 100,
        keys: [],
    } as DsnpKeys;
    const connections = await graph.deserializeDsnpKeys(keys);
    expect(connections).toBeDefined();
    expect(connections.length).toEqual(0);
    await graph.freeGraphState();
});
