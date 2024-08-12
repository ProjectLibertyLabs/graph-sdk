# DSNP Graph SDK

The DSNP Graph SDK is a TypeScript library that provides a high-level interface for interacting with the DSNP Graph. It allows you to perform various operations such as importing data, applying actions, and retrieving graph updates.

## Installation

You can install the DSNP Graph SDK using npm:

```bash
npm install @dsnp/graph-sdk@latest
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
const imported = graph.importUserData([importBundle1, importBundle2]);

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

graph.applyActions(actions.push(action_1));

// Get graph updates
const updates: Update[] = graph.exportUpdates();

// One can retrieve the graph configuration and respective schema mappings
const graph_config = graph.getGraphConfig(environment);

// Get connections for a user graph
const dsnpUserId: number = 1;
const schemaId: number = 1;
const includePending: boolean = true;

const connections: DsnpGraphEdge[] = graph.getConnectionsForUserGraph(dsnpUserId, schemaId, includePending);

// It is important to free the graph state from memory
graph.freeGraphState();

```

## API Reference

### Class: Graph

#### Constructor: new Graph(environment: EnvironmentInterface)

Creates a new instance of the Graph class.

- `environment`: An object that represents the environment details.

#### Methods

- `getGraphHandle(): number`: Returns the handle to the native graph state.
- `getGraphConfig(environment: EnvironmentInterface): Config`: Retrieves the graph configuration.
- `getSchemaIdFromConfig(environment: EnvironmentInterface, connectionType: ConnectionType, privacyType: PrivacyType): number`: Retrieves the schema ID from the graph configuration.
- `getGraphStatesCount(): number`: Number of different graph states in memory. Each instance of the Graph class creates a new graph state.
- `containsUserGraph(dsnpUserId: number): boolean`: Checks if the graph contains the user graph for the specified DSNP user ID.
- `getGraphUsersCount(): number`: Retrieves the count of user graphs in the graph.
- `removeUserGraph(dsnpUserId: number): boolean`: Removes the user graph for the specified DSNP user ID from the graph.
- `importUserData(payload: ImportBundle[]): boolean`: Imports user data into the graph.
- `exportUpdates(): Update[]`: Retrieves the graph updates.
- `exportUserGraphUpdates(dsnpUserId: string): Update[]`: Retrieves the graph updates for a specific user.
- `getConnectionsForUserGraph(dsnpUserId: string, schemaId: number, includePending: boolean): DsnpGraphEdge[]`: Retrieves the connections for a user graph.
- `applyActions(actions: Action[], options?: ActionOptions): boolean`: Applies actions to the graph.
- `commit(): void`: Any changes applied to the graph lives inside a transaction that can get rollbacked until committed.
- `rollback(): void`: Any changes applied to the graph lives inside a transaction will get rollbacked.
- `forceCalculateGraphs(dsnpUserId: string): Update[]`: Forces the calculation of graphs for the specified DSNP user ID.
- `getConnectionsWithoutKeys(): string[]`: Retrieves the connections without keys in the graph.
- `getOneSidedPrivateFriendshipConnections(dsnpUserId: string): DsnpGraphEdge[]`: Retrieves the one-sided private friendship connections for the specified DSNP user ID.
- `getPublicKeys(dsnpUserId: string): DsnpPublicKey[]`: Retrieves the public keys for the specified DSNP user ID.
- `deserializeDsnpKeys(keys: DsnpKeys): DsnpPublicKey[]`: Deserializes DSNP keys.
- `generateKeyPair(keyType: number): GraphKeyPair`: Generates a new key pair for the requested keyType.
- `freeGraphState(): boolean`: Frees the graph state from memory.

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
- `GraphKeyPair`: Represents a key pair for a DSNP user.
- `PageData`: Represents the page data to be retrieved from the chain.
- `Connection`: Represents a connection between two DSNP users.- `

## Examples

### Create and export a new graph

  ```typescript
  import { Graph, EnvironmentInterface, EnvironmentType } from "@dsnp/graph-sdk";

  const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };

  const graph = new Graph(environment);

  const public_follow_schema_id = graph.getSchemaIdFromConfig(environment, ConnectionType.Follow, PrivacyType.Public);

  const connect_action = {
    type: "Connect",
    ownerDsnpUserId: 1,
    connection: {
      dsnpUserId: 2,
      schemaId: public_follow_schema_id,
    },
    dsnpKeys: {
      dsnpUserId: 2,
      keysHash: 100,
      keys: [],
    },
  };

  graph.applyActions([connect_action]);

  const updates = graph.exportUpdates();

  // It is important to free the graph state from memory
  graph.freeGraphState();

  ```

### Add a new graph key

  ```typescript
  import { Graph, EnvironmentInterface, EnvironmentType } from "@dsnp/graph-sdk";

  const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };

  const graph = new Graph(environment);

  const ownerDsnpUserId = 1;
  const x25519_public_key = [ 15, 234, 44, 175, 171, 220, 131, 117, 43, 227, 111, 165, 52, 150, 64, 218, 44, 130, 138, 221, 10, 41, 13, 241, 60, 210, 216, 23, 62, 178, 73, 111,];

  const addGraphKeyAction = {
      type: "AddGraphKey",
      ownerDsnpUserId: dsnpOwnerId,
      newPublicKey: new Uint8Array(x25519_public_key),
  } as AddGraphKeyAction;

  graph.applyActions([addGraphKeyAction]);

  const updates = graph.exportUpdates();

  // It is important to free the graph state from memory
  graph.freeGraphState();

  ```

### Read and deserialize published graph keys

  ```typescript

  import { Graph, EnvironmentInterface, EnvironmentType, DsnpPublicKey } from "@dsnp/graph-sdk";

  const environment: EnvironmentInterface = { environmentType: EnvironmentType.Mainnet };

  const graph = new Graph(environment);

  const dsnpUserId = 1000;
  // published keys blobs fetched from blockchain
  const published_keys_blob = [ 64, 15, 234, 44, 175, 171, 220, 131, 117, 43, 227, 111, 165, 52, 150, 64, 218, 44, 130, 138, 221, 10, 41, 13, 241, 60, 210, 216, 23, 62, 178, 73, 111,];

  let dsnp_keys ={
        dsnpUserId: dsnp_key_owner,
        keysHash: 100,
        keys: [
            {
                index: 0,
                content: new Uint8Array(published_keys_blob),
            }

         ] as KeyData[],
    } as DsnpKeys;

  const deserialized_keys = Graph.deserializeDsnpKeys(dsnp_keys);

  // It is important to free the graph state from memory
  graph.freeGraphState();

  ```

### Update a Private Follow graph

  ```typescript
  const environment: EnvironmentInterface = { environmentType: EnvironmentType Mainnet };
  const graph = new Graph(environment);
  const dsnpOwnerId = 1;
  const private_follow_graph_schema_id = graph.getSchemaIdFromConfig(environment, ConnectionType.Follow, PrivacyType.Private);
  const import_bundle = {
      dsnpUserId: dsnpOwnerId,
      schemaId: private_follow_graph_schema_id,
      keyPairs: [/* get key-pairs associated with the my_dsnp_user_id user from wallet */],
      dsnpKeys: {
          dsnpUserId: dsnpOwnerId,
          keysHash: 100, // get from blockchain
          keys: [/* published keys got from blockchain */],
      } as DsnpKeys,
      pages: [/* published graph pages got from blockchain */],
  } as ImportBundle;

  const imported = graph.importUserData([import_bundle]);

  const connect_action: ConnectAction = {
      type: "Connect",
      ownerDsnpUserId: dsnpOwnerId,
      connection: {
          dsnpUserId: 2,
          schemaId: private_follow_graph_schema_id,
      } as Connection,
  } as ConnectAction;
  const actions = [] as Action[];
  actions.push(connect_action);
  const applied = graph.applyActions(actions);

  const exported_updates = graph.exportUpdates();

  // It is important to free the graph state from memory
  graph.freeGraphState();

  ```

### Update a Private Friendship graph

  ```typescript
  const environment: EnvironmentInterface = { environmentType: EnvironmentType Mainnet };

  const graph = new Graph(environment);

  const dsnpOwnerId = 1;

  const private_friendship_graph_schema_id = graph.getSchemaIdFromConfig(environment, ConnectionType.Friendship, PrivacyType.Private);

  const import_bundle = {
      dsnpUserId: dsnpOwnerId,
      schemaId: private_friendship_graph_schema_id,
      keyPairs: [/* get key-pairs associated with the my_dsnp_user_id user from wallet */],
      dsnpKeys: {
          dsnpUserId: dsnpOwnerId,
          keysHash: 100, // get from blockchain
          keys: [/* published keys got from blockchain */],
      } as DsnpKeys,
      pages: [/* published graph pages got from blockchain */],
  } as ImportBundle;

  const imported = graph.importUserData([import_bundle]);

  // get all associated user without keys so we can fetch and import keys for them
  const user_without_keys = graph.getConnectionsWithoutKeys();
  let users_import_bundles = [] as ImportBundle[];
  for (const user of user_without_keys) {
    let user_dsnp_keys = DsnpKeys {..}  // fetch published DsnpKeys for user
    let user_pages = .. // fetch published private friendship pages for the user
    let user_import_bundle = ImportBundle {
      dsnpUserId: user,
      schemaId: private_friendship_graph_schema_id,
      keyPairs: []. //  empty key pairs for user since we don't know and need their secret keys
      dsnpKeys: user_dsnp_keys,
      pages: user_pages,
    } as ImportBundle;
  }

  const imported = graph.importUserData(users_import_bundles);

  const connect_action: ConnectAction = {
      type: "Connect",
      ownerDsnpUserId: dsnpOwnerId,
      connection: {
          dsnpUserId: 2,
          schemaId: private_friendship_graph_schema_id,
      } as Connection,
    dsnKeys: {
      dsnpUserId: 2,
      keysHash: 100,
      keys: [/* get keys from chain for user 2 */],
    } as DsnpKeys,
  } as ConnectAction;

  const actions = [] as Action[];
  actions.push(connect_action);

  const applied = graph.applyActions(actions);

  // It is important to free the graph state from memory
  graph.freeGraphState();
  
  ```
### Memory management
Creating any `Graph` instance allocates some memory which should be released manually by calling `graph.freeGraphState()` at the end of
graph operations.

Even though we have implemented `FinalizationRegistry`, based on the [documentations](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/FinalizationRegistry#avoid_where_possible) it is not a reliable
way of releasing memory and the best practice is still manual release.
