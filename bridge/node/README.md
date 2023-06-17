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
import { Graph, EnvironmentInterface, ImportBundle, Action, Update, DsnpGraphEdge, DsnpPublicKey, DsnpKeys } from "@dsnp/graph-sdk";

// Create a new instance of the Graph class
const environment: EnvironmentInterface = /* provide environment details */;
const graph = new Graph(environment);

// Import data into the graph
const importBundle: ImportBundle = /* provide import bundle */;
await graph.importUserData([importBundle]);

// Apply actions to the graph
const actions: Action[] = /* provide actions */;
await graph.applyActions(actions);

// Get graph updates
const updates: Update = await graph.exportUpdates();

// Get connections for a user graph
const dsnpUserId: number = /* provide dsnp user id */;
const schemaId: string = /* provide schema id */;
const includePending: boolean = /* specify if pending connections should be included */;
const connections: DsnpGraphEdge[] = await graph.getConnectionsForUserGraphUpdates(dsnpUserId, schemaId, includePending);

// Access other graph-related functions
// ...

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
  