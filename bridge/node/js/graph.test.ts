import { Graph } from "./graph";
import { ImportBundleBuilder } from "./import-bundle-builder";
import {
  PageData,
  GraphKeyPair,
  DsnpKeys,
  ImportBundle,
  Action,
  ConnectAction,
  Connection,
  AddGraphKeyAction,
  KeyData,
  GraphKeyType,
  PersistPageUpdate,
  Update,
} from "./models";
import {
  Config,
  ConnectionType,
  DsnpVersion,
  PrivacyType,
} from "./models/config";
import {
  DevEnvironment,
  EnvironmentInterface,
  EnvironmentType,
} from "./models/environment";

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
    2: {
      dsnpVersion: DsnpVersion.Version1_0,
      connectionType: ConnectionType.Follow,
      privacyType: PrivacyType.Private,
    },
    3: {
      dsnpVersion: DsnpVersion.Version1_0,
      connectionType: ConnectionType.Friendship,
      privacyType: PrivacyType.Private,
    },
  },
  graphPublicKeySchemaId: 11,
};
const environment: DevEnvironment = {
  environmentType: EnvironmentType.Dev,
  config,
};

describe("Graph tests", () => {
  let graph: Graph;
  let handle: number;

  beforeEach(() => {
    graph = new Graph(environment);
    handle = graph.getGraphHandle();
    expect(handle).toBeDefined();
  });

  afterEach(() => {
    graph.freeGraphState();
  });

  test('printHelloGraph should print "Hello, Graph!"', async () => {
    // Mock the console.log function
    const consoleLogMock = jest.spyOn(console, "log").mockImplementation();
    graph.printHelloGraph();
    expect(consoleLogMock).toHaveBeenCalledWith("Hello, Graph!");
  });

  test("getGraphConfig should return the graph config", async () => {
    const config_ret = graph.getGraphConfig(environment);
    expect(config_ret).toBeDefined();
    expect(config_ret.graphPublicKeySchemaId).toEqual(11);
  });

  test("getGraphConfig with Mainnet environment should return the graph config", async () => {
    const environment: EnvironmentInterface = {
      environmentType: EnvironmentType.Mainnet,
    };
    const graph = new Graph(environment);
    const config = graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    expect(config.graphPublicKeySchemaId).toEqual(5);
    const schema_id = graph.getSchemaIdFromConfig(
      environment,
      ConnectionType.Follow,
      PrivacyType.Public,
    );
    expect(schema_id).toEqual(1);
    graph.freeGraphState();
  });

  test("getGraphConfig with Rococo environment should return the graph config", async () => {
    const environment: EnvironmentInterface = {
      environmentType: EnvironmentType.Rococo,
    };
    const graph = new Graph(environment);
    const config = graph.getGraphConfig(environment);
    expect(config).toBeDefined();
    graph.freeGraphState();
  });

  test("getGraphStatesCount should be unchanged after create/free new graph", async () => {
    const originalCount = graph.getGraphStatesCount();
    const secondGraph = new Graph(environment);
    secondGraph.freeGraphState();
    expect(secondGraph.getGraphStatesCount()).toEqual(originalCount);
  });

  test("getGraphStatesCount should be one after graph is initialized", async () => {
    const count = graph.getGraphStatesCount();
    expect(count).toEqual(1);
  });

  test("getGraphUsersCount should be zero on initialized graph", async () => {
    const count = graph.getGraphUsersCount();
    expect(count).toEqual(0);
  });

  test("containsUserGraph should return false on initialized graph", async () => {
    const contains = graph.containsUserGraph("1");
    expect(contains).toEqual(false);
  });

  test("removeUserGraph should pass through on initialized graph", async () => {
    const removed = graph.removeUserGraph("1");
    expect(removed).toEqual(true);
  });

  test("importUserData should pass through on initialized graph", async () => {
    // Set up import data
    const dsnpUserId1 = 1;
    const dsnpUserId2 = 2;

    const pageData1: PageData = {
      pageId: 1,
      content: new Uint8Array([
        24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0,
      ]),
      contentHash: 100,
    };

    const keyPairs1: GraphKeyPair[] = [];
    const keyPairs2: GraphKeyPair[] = [];

    const dsnpKeys1: DsnpKeys = {
      dsnpUserId: dsnpUserId1.toString(),
      keysHash: 100,
      keys: [],
    };

    const dsnpKeys2: DsnpKeys = {
      dsnpUserId: dsnpUserId2.toString(),
      keysHash: 100,
      keys: [],
    };

    const importBundle1: ImportBundle = {
      dsnpUserId: dsnpUserId1.toString(),
      schemaId: 1,
      keyPairs: keyPairs1,
      dsnpKeys: dsnpKeys1,
      pages: [pageData1],
    };

    const importBundle2: ImportBundle = {
      dsnpUserId: dsnpUserId2.toString(),
      schemaId: 1,
      keyPairs: keyPairs2,
      dsnpKeys: dsnpKeys2,
      pages: [pageData1],
    };

    // Import user data for each ImportBundle
    const imported = graph.importUserData([importBundle1, importBundle2]);
    expect(imported).toEqual(true);
  });

  test("applyActions with empty actions should pass through on initialized graph", async () => {
    // Set up actions
    const actions = [] as Action[];
    const applied = graph.applyActions(actions);
    expect(applied).toEqual(true);
  });

  test("applyActions with some actions should pass through on initialized graph", async () => {
    // Set up actions
    const actions = [] as Action[];
    const action_1 = {
      type: "Connect",
      ownerDsnpUserId: "1",
      connection: {
        dsnpUserId: "2",
        schemaId: 1,
      } as Connection,
    } as ConnectAction;

    actions.push(action_1);
    const applied = graph.applyActions(actions);
    expect(applied).toEqual(true);

    const exported = graph.exportUpdates();
    expect(exported).toBeDefined();
    expect(exported.length).toEqual(1);
  });

  test("applyActions with options should honor options", async () => {
    // Add some connections to 2 empty graphs
    const dsnpId_1 = "1";
    const dsnpId_2 = "2";
    const schemaId = 1;

    let actions: Action[] = [
      {
        type: "Connect",
        ownerDsnpUserId: dsnpId_1,
        connection: {
          dsnpUserId: dsnpId_2,
          schemaId,
        },
      },
      {
        type: "Connect",
        ownerDsnpUserId: dsnpId_2,
        connection: {
          dsnpUserId: dsnpId_1,
          schemaId,
        },
      },
    ];

    graph.applyActions(actions);
    const exports: Update[] = graph.exportUpdates();
    const imports = exports
      .filter((update) => update.type === "PersistPage")
      .map((update) => {
        const persist = update as PersistPageUpdate;
        const builder = new ImportBundleBuilder()
          .withDsnpUserId(persist.ownerDsnpUserId)
          .withSchemaId(persist.schemaId)
          .withPageData(persist.pageId, persist.payload, 1000);
        return builder.build();
      });

    graph.importUserData(imports);

    // Now we have graphs with connections, attempt to add redundant connection
    actions = [
      {
        type: "Connect",
        ownerDsnpUserId: dsnpId_1,
        connection: {
          dsnpUserId: dsnpId_2,
          schemaId,
        },
      },
    ];
    expect(() => graph.applyActions(actions)).toThrow();
    expect(() =>
      graph.applyActions(actions, { ignoreExistingConnections: true }),
    ).not.toThrow();

    // Now try to remove a non-existent connection
    actions = [
      {
        type: "Disconnect",
        ownerDsnpUserId: dsnpId_1,
        connection: {
          dsnpUserId: "999",
          schemaId,
        },
      },
    ];
    expect(() => graph.applyActions(actions)).toThrow();
    expect(() =>
      graph.applyActions(actions, { ignoreMissingConnections: true }),
    ).not.toThrow();
  });

  test("getConnectionsForUserGraph with empty connections should return empty array", async () => {
    // Set up actions
    const actions = [] as Action[];
    const action_1 = {
      type: "Connect",
      ownerDsnpUserId: "1",
      connection: {
        dsnpUserId: "2",
        schemaId: 1,
      } as Connection,
      dsnpKeys: {
        dsnpUserId: "2",
        keysHash: 100,
        keys: [],
      } as DsnpKeys,
    } as ConnectAction;

    actions.push(action_1);
    const applied = graph.applyActions(actions);
    expect(applied).toEqual(true);
    const connections = graph.getConnectionsForUserGraph("1", 1, true);
    expect(connections).toBeDefined();
    expect(connections.length).toEqual(1);

    const forceCalculateGraphs = graph.forceCalculateGraphs("1");
    expect(forceCalculateGraphs).toBeDefined();
    expect(forceCalculateGraphs.length).toEqual(0);
  });

  test("getConnectionsWithoutKeys with empty connections should return empty array", async () => {
    const connections = graph.getConnectionsWithoutKeys();
    expect(connections).toBeDefined();
    expect(connections.length).toEqual(0);

    expect(() => graph.getOneSidedPrivateFriendshipConnections("1")).toThrow(
      "User graph for 1 is not imported",
    );
  });

  test("getPublicKeys with empty connections should return empty array", async () => {
    const keys = graph.getPublicKeys("1");
    expect(keys).toBeDefined();
    expect(keys.length).toEqual(0);
  });

  test("deserializeDsnpKeys with empty keys should return empty array", async () => {
    const keys = {
      dsnpUserId: "2",
      keysHash: 100,
      keys: [],
    } as DsnpKeys;
    const des_keys = Graph.deserializeDsnpKeys(keys);
    expect(des_keys).toBeDefined();
    expect(des_keys.length).toEqual(0);
  });

  test("Create and export a new graph", async () => {
    const public_follow_graph_schema_id = graph.getSchemaIdFromConfig(
      environment,
      ConnectionType.Follow,
      PrivacyType.Public,
    );

    const connect_action: ConnectAction = {
      type: "Connect",
      ownerDsnpUserId: "1",
      connection: {
        dsnpUserId: "2",
        schemaId: public_follow_graph_schema_id,
      } as Connection,
      dsnpKeys: {
        dsnpUserId: "2",
        keysHash: 100,
        keys: [],
      } as DsnpKeys,
    } as ConnectAction;

    const actions = [] as Action[];
    actions.push(connect_action);
    const applied = graph.applyActions(actions);
    expect(applied).toEqual(true);

    const connections_including_pending = graph.getConnectionsForUserGraph(
      "1",
      public_follow_graph_schema_id,
      true,
    );

    expect(connections_including_pending).toBeDefined();
    expect(connections_including_pending.length).toEqual(1);

    const exported = graph.exportUpdates();
    expect(exported).toBeDefined();
    expect(exported.length).toEqual(1);
  });

  test("Test exportGraph all-user and single-user variants", async () => {
    const publicFollowGraphSchemaId = graph.getSchemaIdFromConfig(
      environment,
      ConnectionType.Follow,
      PrivacyType.Public,
    );

    const actions: Action[] = [
      {
        type: "Connect",
        ownerDsnpUserId: "1",
        connection: {
          dsnpUserId: "2",
          schemaId: publicFollowGraphSchemaId,
        },
      },
      {
        type: "Connect",
        ownerDsnpUserId: "2",
        connection: {
          dsnpUserId: "1",
          schemaId: publicFollowGraphSchemaId,
        },
      },
    ];
    const applied = graph.applyActions(actions);
    expect(applied).toEqual(true);

    // Check that exportUpdates exports all users
    let exported = graph.exportUpdates();
    expect(exported).toBeDefined();
    expect(exported.length).toEqual(2);
    expect(
      exported.every((bundle) =>
        ["1", "2"].some((user) => user === bundle.ownerDsnpUserId),
      ),
    );

    // Check that single-user export contains only that user
    exported = graph.exportUserGraphUpdates("1");
    expect(exported).toBeDefined();
    expect(exported.length).toEqual(1);
    expect(exported.every((bundle) => bundle.ownerDsnpUserId === "1"));
  });

  test("Add a new graph key", async () => {
    const dsnpOwnerId = 1;
    const x25519_public_key = [
      15, 234, 44, 175, 171, 220, 131, 117, 43, 227, 111, 165, 52, 150, 64, 218,
      44, 130, 138, 221, 10, 41, 13, 241, 60, 210, 216, 23, 62, 178, 73, 111,
    ];

    const addGraphKeyAction = {
      type: "AddGraphKey",
      ownerDsnpUserId: dsnpOwnerId.toString(),
      newPublicKey: new Uint8Array(x25519_public_key),
    } as AddGraphKeyAction;

    const actions = [] as Action[];
    actions.push(addGraphKeyAction);

    const applied = graph.applyActions(actions);
    expect(applied).toEqual(true);

    const exported = graph.exportUpdates();
    expect(exported).toBeDefined();
    expect(exported.length).toEqual(1);
  });

  test("Read and deserialize published graph keys", async () => {
    const dsnp_key_owner = 1000;

    // published keys blobs fetched from blockchain
    const published_keys_blob = [
      64, 15, 234, 44, 175, 171, 220, 131, 117, 43, 227, 111, 165, 52, 150, 64,
      218, 44, 130, 138, 221, 10, 41, 13, 241, 60, 210, 216, 23, 62, 178, 73,
      111,
    ];
    const dsnp_keys = {
      dsnpUserId: dsnp_key_owner.toString(),
      keysHash: 100,
      keys: [
        {
          index: 0,
          content: new Uint8Array(published_keys_blob),
        },
      ] as KeyData[],
    } as DsnpKeys;

    const deserialized_keys = Graph.deserializeDsnpKeys(dsnp_keys);
    expect(deserialized_keys).toBeDefined();
  });

  test("generateKeyPair should return a key pair", async () => {
    const keyPair = Graph.generateKeyPair(GraphKeyType.X25519);
    expect(keyPair).toBeDefined();
    expect(keyPair.publicKey).toBeDefined();
    expect(keyPair.secretKey).toBeDefined();
    expect(keyPair.keyType).toEqual(GraphKeyType.X25519);
  });
});
