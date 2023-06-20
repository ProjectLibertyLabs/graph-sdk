# DSNP Graph SDK

The DSNP Graph SDK is a TypeScript library that provides a high-level interface for interacting with the DSNP Graph. It allows you to perform various operations such as importing data, applying actions, and retrieving graph updates.

## Installation

You can install the DSNP Graph SDK using npm:

```bash
npm install @dsnp/graph-sdk
```

## Usage

Here's an example of how to use the DSNP Graph SDK:

```typescript
import { Graph, EnvironmentInterface, EnvironmentType, ImportBundle, Action, Update, DsnpGraphEdge, DsnpPublicKey, DsnpKeys } from "@dsnp/graph-sdk";

// Create a new instance of the Graph class
const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };

const graph = new Graph(environment);

// Import data into the graph
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

// Apply actions to the graph
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

await graph.applyActions(actions.push(action_1));

// Get graph updates
const updates: Update[] = await graph.exportUpdates();

// One can retrieve the graph configuration and respective schema mappings
const graph_config = await graph.getGraphConfig(environment);

// Get connections for a user graph
const dsnpUserId: number = 1;
const schemaId: number = 1;
const includePending: boolean = true;

const connections: DsnpGraphEdge[] = await graph.getConnectionsForUserGraph(dsnpUserId, schemaId, includePending);

// Free the graph state
graph.freeGraphState();

```

## API Reference

### Class: Graph

#### Constructor: new Graph(environment: EnvironmentInterface, capacity?: number)

Creates a new instance of the Graph class.

- `environment`: An object that represents the environment details.
- `capacity` (optional): The initial capacity of the graph.

#### Methods

- `getGraphHandle(): number`: Returns the handle to the native graph state.
- `getGraphCapacity(): Promise<number>`: Retrieves the capacity of the graph.
- `getGraphSize(): Promise<number>`: Retrieves the current size of the graph.
- `containsUserGraph(dsnpUserId: number): Promise<boolean>`: Checks if the graph contains the user graph for the specified DSNP user ID.
- `getGraphUsersCount(): Promise<number>`: Retrieves the count of user graphs in the graph.
- `removeUserGraph(dsnpUserId: number): Promise<void>`: Removes the user graph for the specified DSNP user ID from the graph.
- `importUserData(payload: ImportBundle[]): Promise<void>`: Imports user data into the graph.
- `exportUpdates(): Promise<Update>`: Retrieves the graph updates.
- `getConnectionsForUserGraphUpdates(dsnpUserId: number, schemaId: string, includePending: boolean): Promise<DsnpGraphEdge[]>`: Retrieves the connections for a user graph.
- `applyActions(actions: Action[]): Promise<void>`: Applies actions to the graph.
- `forceCalculateGraphs(dsnpUserId: number): Promise<Update>`: Forces the calculation of graphs for the specified DSNP user ID.
- `getConnectionsWithoutKeys(): Promise<number[]>`: Retrieves the connections without keys in the graph.
- `getOneSidedPrivateFriendshipConnections(dsnpUserId: number): Promise<DsnpGraphEdge[]>`: Retrieves the one-sided private friendship connections for the specified DSNP user ID.
- `getPublicKeys(dsnpUserId: number): Promise<DsnpPublicKey[]>`: Retrieves the public keys for the specified DSNP user ID.
- `deserializeDsnpKeys(keys: DsnpKeys): Promise<DsnpPublicKey[]>`: Deserializes DSNP keys.
- `getGraphConfig(environment: EnvironmentInterface): Promise<Config>`: Retrieves the graph configuration.
- `freeGraphState(): void`: Frees the graph state.

### Type Definitions

The SDK provides various type definitions that can be used with the Graph class and other functions.

- `Config`: Represents the graph configuration.
- `EnvironmentInterface`: Represents the environment details.
- `ImportBundle`: Encapsulates the decryption keys and page data to be retrieved from the chain.
- `Update`: Represents the different updates to be applied to the graph.
- `Action`: Represents the different kinds of actions that can be applied to the graph.
- `DsnpGraphEdge`: Represents a connection in the graph.
- `DsnpPublicKey`: Represents a published graph key for a DSNP user.
- `DsnpKeys`: Encapsulates a DSNP user and their associated graph public keys.
  
