import path from "path";
import { EnvironmentInterface } from "./models/environment";
import { Config } from "./models/config";

// Load the native neon graphsdk module
function loadNativeModule(): any {
    try {
        return require(path.join(__dirname, "/dsnp_graph_sdk.node"));
    } catch (error) {
        throw new Error("Unable to load the native module dsnp_graph_sdk.node");
    }
}

const graphsdk =  loadNativeModule();

console.log("Loaded graphsdk.node bindings");

// Define the Native interface
export interface Native {
    printHelloGraph(): void;
    getGraphConfig(): Config;
    initializeGraphState(environment: EnvironmentInterface): number;
    initializeGraphStateWithCapacity(environment: EnvironmentInterface, capacity: number): number;
    containsUserGraph(handle: number, dsnpUserId: number): boolean;
    getGraphUsersLength(handle: number): number;
    removeUserGraph(handle: number, dsnpUserId: number): void;
    importUserData(handle: number, data: any): void;
    exportUpdates(handle: number): any; // Replace `any` with the appropriate type
    applyActions(handle: number, actions: any): void; // Replace `any` with the appropriate type
    forceCalculateGraphs(handle: number, dsnpUserId: number): any; // Replace `any` with the appropriate type
    getConnectionsForUserGraph(handle: number, dsnpUserId: number, schemaId: string, includePending: boolean): any; // Replace `any` with the appropriate type
    getUsersWithoutKeys(handle: number): any; // Replace `any` with the appropriate type
    getOneSidedPrivateFriendshipConnections(handle: number, dsnpUserId: number): any; // Replace `any` with the appropriate type
    getPublicKeys(handle: number, dsnpUserId: number): any; // Replace `any` with the appropriate type
    deserializeDsnpKeys(keys: any): any; // Replace `any` with the appropriate type
    freeGraphState(handle: number): void;
}

// Export the graphsdk module
export const graphsdkModule: Native = graphsdk;
