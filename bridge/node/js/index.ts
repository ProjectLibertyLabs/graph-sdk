import path from "path";
import {
  Action,
  ActionOptions,
  Config,
  ConnectionType,
  DsnpGraphEdge,
  DsnpKeys,
  DsnpPublicKey,
  EnvironmentInterface,
  GraphKeyPair,
  GraphKeyType,
  ImportBundle,
  PrivacyType,
  Update,
} from "./models";

// Load the native neon graphsdk module
function loadNativeModule(): Native {
  try {
    return require(path.join(__dirname, "/dsnp_graph_sdk_node.node"));
  } catch (error) {
    throw new Error(
      "Unable to load the native module dsnp_graph_sdk_node.node",
    );
  }
}

// Define the Native interface
export interface Native {
  printHelloGraph(): void;
  initializeGraphState(environment: EnvironmentInterface): number;
  getGraphConfig(environment: EnvironmentInterface): Config;
  getSchemaIdFromConfig(
    environment: EnvironmentInterface,
    connectionType: ConnectionType,
    privacyType: PrivacyType,
  ): number;
  getGraphStatesCount(): number;
  getGraphUsersCount(handle: number): number;
  containsUserGraph(handle: number, dsnpUserId: string): boolean;
  removeUserGraph(handle: number, dsnpUserId: string): boolean;
  importUserData(handle: number, payload: ImportBundle[]): boolean;
  applyActions(
    handle: number,
    actions: Action[],
    options?: ActionOptions,
  ): boolean;
  exportUpdates(handle: number): Update[];
  exportUserGraphUpdates(handle: number, dsnpUserId: string): Update[];
  getConnectionsForUserGraph(
    handle: number,
    dsnpUserId: string,
    schemaId: number,
    includePending: boolean,
  ): DsnpGraphEdge[];
  forceCalculateGraphs(handle: number, dsnpUserId: string): Update[];
  getConnectionsWithoutKeys(handle: number): string[];
  getOneSidedPrivateFriendshipConnections(
    handle: number,
    dsnpUserId: string,
  ): DsnpGraphEdge[];
  getPublicKeys(handle: number, dsnpUserId: string): DsnpPublicKey[];
  deserializeDsnpKeys(keys: DsnpKeys): DsnpPublicKey[];
  generateKeyPair(keyType: GraphKeyType): GraphKeyPair;
  freeGraphState(handle: number): boolean;
}

// Export the graphsdk module
export const graphsdkModule: Native = loadNativeModule();

// Export the models
export * from "./models";
export * from "./graph";
export * from "./import-bundle-builder";
