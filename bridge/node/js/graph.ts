
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

    getGraphCapacity(handle: number): Promise<number> {
        return graphsdkModule.getGraphCapacity(handle);
    }

    getGraphSize(handle: number): Promise<number> {
        throw new Error("Method not implemented.");
    }

    containsUserGraph(handle: number, dsnpUserId: number): Promise<boolean> {
        throw new Error("Method not implemented.");
    }

    getGraphUsersCount(handle: number): Promise<number> {
        throw new Error("Method not implemented.");
    }

    removeUserGraph(handle: number, dsnpUserId: number): Promise<void> {
        throw new Error("Method not implemented.");
    }

    importUserData(handle: number, payload: [ImportBundle]): Promise<void> {
        throw new Error("Method not implemented.");
    }

    exportUpdates(handle: number): Promise<Update> {
        throw new Error("Method not implemented.");
    }

    getConnectionsForUserGraphUpdates(handle: number, dsnpUserId: number, schemaId: string, includePending: boolean): Promise<[DsnpGraphEdge]> {
        throw new Error("Method not implemented.");
    }

    applyActions(handle: number, actions: [Action]): Promise<void> {
        throw new Error("Method not implemented.");
    }

    forceCalculateGraphs(handle: number, dsnpUserId: number): Promise<Update> {
        throw new Error("Method not implemented.");
    }

    getConnectionsWithoutKeys(handle: number): Promise<[number]> {
        throw new Error("Method not implemented.");
    }

    getOneSidedPrivateFriendshipConnections(handle: number, dsnpUserId: number): Promise<[DsnpGraphEdge]> {
        throw new Error("Method not implemented.");
    }

    getPublicKeys(handle: number, dsnpUserId: number): Promise<[DsnpPublicKey]> {
        throw new Error("Method not implemented.");
    }

    deserializeDsnpKeys(keys: DsnpKeys): Promise<[DsnpPublicKey]> {
        throw new Error("Method not implemented.");
    }

    freeGraphState(): Promise<void> {
        throw new Error("Method not implemented.");
    }

    public getGraphConfig(environment: EnvironmentInterface): Promise<Config> {
        return graphsdkModule.getGraphConfig(environment);
    }
    
    public printHelloGraph(): void {
        console.log( graphsdkModule.printHelloGraph() );
    }
}
