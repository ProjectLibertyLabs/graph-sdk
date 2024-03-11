import path from "path";
import os from "os";
import fs from "fs";
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
    const targetPath = path.join(
      __dirname,
      `/${getTarget()}_dsnp_graph_sdk_node.node`,
    );
    if (fs.existsSync(targetPath)) {
      return require(targetPath);
    }else {
      // using the default library name (useful for local testing)
      return require(path.join(__dirname, "/dsnp_graph_sdk_node.node"));
    }
  } catch (error) {
    let message = 'Unknown Error'
    if (error instanceof Error) message = error.message
    throw new Error(
      `Unable to load the native module dsnp_graph_sdk_node.node (${message})`,
    );
  }
}

function getTarget(): String {
  const platform = os.platform().toLowerCase();
  const arch = os.arch().toLowerCase();

  // Windows
  if (platform.includes("win") && arch.includes("x64")) {
    return "x86_64-pc-windows-msvc";

    // MacOS
  } else if (platform.includes("darwin") && arch.includes("x64")) {
    return "x86_64-apple-darwin";
  } else if (platform.includes("darwin") && arch.includes("arm64")) {
    return "aarch64-apple-darwin";

    // Linux
  } else if (platform.includes("linux") && arch.includes("x64")) {
    return "x86_64-unknown-linux-gnu";
  } else if (platform.includes("linux") && arch.includes("arm64")) {
    return "aarch64-unknown-linux-gnu";
  }

  throw new Error(
    `Operating System: ${platform} Architecture: ${arch} is not supported!`,
  );
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
