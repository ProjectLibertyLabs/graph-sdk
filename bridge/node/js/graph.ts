import { graphsdkModule } from "./index";
import { ImportBundle, Update, DsnpGraphEdge, Action, DsnpPublicKey, DsnpKeys, Config, ConnectionType, PrivacyType, GraphKeyPair } from "./models";
import { EnvironmentInterface } from "./models/environment";

export class Graph {
    /// The handle to the native graph state
    private handle: number;

    constructor(environment: EnvironmentInterface, capacity?: number) {
        if (capacity) {
            this.handle = graphsdkModule.initializeGraphStateWithCapacity(environment, capacity);
        } else {
            this.handle = graphsdkModule.initializeGraphState(environment);
        }
    }

    getGraphHandle(): number {
        return this.handle;
    }

    getGraphConfig(environment: EnvironmentInterface): Promise<Config> {
        return graphsdkModule.getGraphConfig(environment);
    }

    getSchemaIdFromConfig(environment: EnvironmentInterface, connectionType: ConnectionType, privacyType: PrivacyType): Promise<number> {
        return graphsdkModule.getSchemaIdFromConfig(environment, connectionType, privacyType);
    }

    getGraphCapacity(): Promise<number> {
        return graphsdkModule.getGraphCapacity(this.handle);
    }

    getGraphStatesCount(): Promise<number> {
        return graphsdkModule.getGraphStatesCount();
    }

    containsUserGraph(dsnpUserId: string): Promise<boolean> {
        return graphsdkModule.containsUserGraph(this.handle, dsnpUserId);
    }

    getGraphUsersCount(): Promise<number> {
        return graphsdkModule.getGraphUsersCount(this.handle);
    }

    removeUserGraph(dsnpUserId: string): Promise<boolean> {
        return graphsdkModule.removeUserGraph(this.handle, dsnpUserId);
    }

    importUserData(payload: ImportBundle[]): Promise<boolean> {
        return graphsdkModule.importUserData(this.handle, payload);
    }

    exportUpdates(): Promise<Update[]> {
        return graphsdkModule.exportUpdates(this.handle);
    }

    getConnectionsForUserGraph(dsnpUserId: string, schemaId: number, includePending: boolean): Promise<DsnpGraphEdge[]> {
        return graphsdkModule.getConnectionsForUserGraph(this.handle, dsnpUserId, schemaId, includePending);
    }

    applyActions(actions: Action[]): Promise<boolean> {
        return graphsdkModule.applyActions(this.handle, actions);
    }

    forceCalculateGraphs(dsnpUserId: string): Promise<Update[]> {
        return graphsdkModule.forceCalculateGraphs(this.handle, dsnpUserId);
    }

    getConnectionsWithoutKeys(): Promise<number[]> {
        return graphsdkModule.getConnectionsWithoutKeys(this.handle);
    }

    getOneSidedPrivateFriendshipConnections(dsnpUserId: string): Promise<DsnpGraphEdge[]> {
        return graphsdkModule.getOneSidedPrivateFriendshipConnections(this.handle, dsnpUserId);
    }

    getPublicKeys(dsnpUserId: string): Promise<DsnpPublicKey[]> {
        return graphsdkModule.getPublicKeys(this.handle, dsnpUserId);
    }

    static deserializeDsnpKeys(keys: DsnpKeys): Promise<DsnpPublicKey[]> {
        return graphsdkModule.deserializeDsnpKeys(keys);
    }

    static generateKeyPair(keyType: number): Promise<GraphKeyPair> {
        return graphsdkModule.generateKeyPair(keyType);
    }

    freeGraphState(): Promise<boolean> {
        return graphsdkModule.freeGraphState(this.handle);
    }
    
    printHelloGraph(): void {
        console.log(graphsdkModule.printHelloGraph());
    }
}
