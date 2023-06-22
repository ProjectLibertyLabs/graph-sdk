import path from "path";
import { Action, Config, ConnectAction, ConnectionType, DsnpGraphEdge, DsnpKeys, DsnpPublicKey, EnvironmentInterface, GraphKeyPair, GraphKeyType, ImportBundle, PrivacyType, Update } from "./models";


// Load the native neon graphsdk module
function loadNativeModule(): Native {
    try {
        return require(path.join(__dirname, "/dsnp_graph_sdk_node.node"));
    } catch (error) {
        throw new Error("Unable to load the native module dsnp_graph_sdk_node.node");
    }
}

// Define the Native interface
export interface Native {
    printHelloGraph(): void;
    initializeGraphState(environment: EnvironmentInterface): number;
    initializeGraphStateWithCapacity(environment: EnvironmentInterface, capacity: number):number
    getGraphConfig(environment: EnvironmentInterface): Promise<Config>;
    getSchemaIdFromConfig(environment: EnvironmentInterface, connectionType: ConnectionType, privacyType: PrivacyType): Promise<number>;
    getGraphCapacity(handle: number): Promise<number>;
    getGraphStatesCount(): Promise<number>;
    getGraphUsersCount(handle: number): Promise<number>;
    containsUserGraph(handle: number, dsnpUserId: number): Promise<boolean>;
    removeUserGraph(handle: number, dsnpUserId: number): Promise<boolean>;
    importUserData(handle: number, payload: ImportBundle[]): Promise<boolean>;
    applyActions(handle: number, actions: Action[]): Promise<boolean>;
    exportUpdates(handle: number): Promise<Update[]>;
    getConnectionsForUserGraph(handle: number, dsnpUserId: number, schemaId: number, includePending: boolean):Promise<DsnpGraphEdge[]>;
    forceCalculateGraphs(handle: number, dsnpUserId: number): Promise<Update[]>;
    getConnectionsWithoutKeys(handle: number): Promise<number[]>;
    getOneSidedPrivateFriendshipConnections(handle: number, dsnpUserId: number): Promise<DsnpGraphEdge[]>;
    getPublicKeys(handle: number, dsnpUserId: number): Promise<DsnpPublicKey[]>;
    deserializeDsnpKeys(keys: DsnpKeys): Promise<DsnpPublicKey[]>;
    generateKeyPair(keyType: GraphKeyType): Promise<GraphKeyPair>;
    freeGraphState(handle: number): Promise<boolean>;
}

// Export the graphsdk module
export const graphsdkModule: Native = loadNativeModule();

// Export the models
export * from "./models";
export * from "./graph";
