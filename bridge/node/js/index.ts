import path from "path";

// Load the native neon graphsdk module
const graphsdk = require(path.join(__dirname, "/dsnp_graph_sdk.node"));

console.log("Loaded graphsdk.node bindings");

// Define the Native interface
export interface Native {
    printHelloGraph(): void;
    initializeGraphState(environment: string): number;
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
