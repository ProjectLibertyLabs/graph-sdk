
import { graphsdkModule, Native } from "./index";
import { ImportBundle, Update, DsnpGraphEdge, Action, DsnpPublicKey, DsnpKeys } from "./models";
import { Config } from "./models/config";
import { EnvironmentInterface } from "./models/environment";

export class Graph {
    /// The handle to the native graph state
    private handle: number;
    
    constructor( environment: EnvironmentInterface , capacity?: number ) {
        if ( capacity ) {
            this.handle = graphsdkModule.initializeGraphStateWithCapacity( environment, capacity );
        } else {
            this.handle = graphsdkModule.initializeGraphState( environment );
        }
    }

    getGraphHandle(): number {
        return this.handle;
    }

    getGraphCapacity(): Promise<number> {
        return graphsdkModule.getGraphCapacity(this.handle);
    }

    getGraphSize(): Promise<number> {
        return graphsdkModule.getGraphSize(this.handle);
    }

    containsUserGraph(dsnpUserId: number): Promise<boolean> {
        return graphsdkModule.containsUserGraph(this.handle, dsnpUserId);
    }

    getGraphUsersCount(): Promise<number> {
        return graphsdkModule.getGraphUsersCount(this.handle);
    }

    removeUserGraph(dsnpUserId: number): Promise<void> {
        return graphsdkModule.removeUserGraph(this.handle, dsnpUserId);
    }

    importUserData(payload: [ImportBundle]): Promise<void> {
        return graphsdkModule.importUserData(this.handle, payload);
    }

    exportUpdates(): Promise<Update> {
        return graphsdkModule.exportUpdates(this.handle);
    }

    getConnectionsForUserGraphUpdates(dsnpUserId: number, schemaId: string, includePending: boolean): Promise<[DsnpGraphEdge]> {
        return graphsdkModule.getConnectionsForUserGraphUpdates(this.handle, dsnpUserId, schemaId, includePending);
    }

    applyActions(actions: [Action]): Promise<void> {
        return graphsdkModule.applyActions(this.handle, actions);
    }

    forceCalculateGraphs( dsnpUserId: number): Promise<Update> {
        return graphsdkModule.forceCalculateGraphs(this.handle, dsnpUserId);
    }

    getConnectionsWithoutKeys(): Promise<[number]> {
        return graphsdkModule.getConnectionsWithoutKeys(this.handle);
    }

    getOneSidedPrivateFriendshipConnections(dsnpUserId: number): Promise<[DsnpGraphEdge]> {
        return graphsdkModule.getOneSidedPrivateFriendshipConnections(this.handle, dsnpUserId);
    }

    getPublicKeys(dsnpUserId: number): Promise<[DsnpPublicKey]> {
        return graphsdkModule.getPublicKeys(this.handle, dsnpUserId);
    }

    deserializeDsnpKeys(keys: DsnpKeys): Promise<[DsnpPublicKey]> {
        return graphsdkModule.deserializeDsnpKeys(keys);
    }

    public getGraphConfig(environment: EnvironmentInterface): Promise<Config> {
        return graphsdkModule.getGraphConfig(environment);
    }
    
    // finalizer: TODO figure out a better way to do this
    public freeGraphState(): void {
        graphsdkModule.freeGraphState(this.handle);
    }

    public printHelloGraph(): void {
        console.log( graphsdkModule.printHelloGraph() );
    }
}
