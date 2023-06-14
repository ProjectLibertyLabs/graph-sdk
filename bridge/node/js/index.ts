import path from "path";
import { Action, Config, DsnpGraphEdge, DsnpKeys, DsnpPublicKey, EnvironmentInterface, ImportBundle, Update } from "./models";


// Load the native neon graphsdk module
function loadNativeModule(): Native {
    try {
        return require(path.join(__dirname, "/graphsdk.node"));
    } catch (error) {
        throw new Error("Unable to load the native module graphsdk.node");
    }
}

// Define the Native interface
export interface Native {
    printHelloGraph(): void;
    initializeGraphState(environment: EnvironmentInterface): number;
    initializeGraphStateWithCapacity(environment: EnvironmentInterface, capacity: number):number
    getGraphConfig(environment: EnvironmentInterface): Promise<Config>;
    getGraphCapacity(handle: number): Promise<number>;
    getGraphStatesCount(): Promise<number>;
    getGraphUsersCount(handle: number): Promise<number>;
    containsUserGraph(handle: number, dsnpUserId: number): Promise<boolean>;
    removeUserGraph(handle: number, dsnpUserId: number): Promise<void>;
    importUserData(handle: number, payload: [ImportBundle]): Promise<void>;
    exportUpdates(handle: number): Promise<Update>;
    getConnectionsForUserGraphUpdates(handle: number, dsnpUserId: number, schemaId: string, includePending: boolean):Promise<[DsnpGraphEdge]>;
    applyActions(handle: number, actions: [Action]): Promise<void>;
    forceCalculateGraphs(handle: number, dsnpUserId: number): Promise<Update>;
    getConnectionsWithoutKeys(handle: number): Promise<[number]>;
    getOneSidedPrivateFriendshipConnections(handle: number, dsnpUserId: number): Promise<[DsnpGraphEdge]>;
    getPublicKeys(handle: number, dsnpUserId: number): Promise<[DsnpPublicKey]>;
    deserializeDsnpKeys(keys: DsnpKeys): Promise<[DsnpPublicKey]>;
    freeGraphState(handle: number): Promise<void>;
    freeAllGraphStates(): Promise<void>;
}

// Export the graphsdk module
export const graphsdkModule: Native = loadNativeModule();
