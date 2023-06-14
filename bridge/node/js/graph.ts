import { graphsdkModule } from "./index";
import { ImportBundle, Update, DsnpGraphEdge, Action, DsnpPublicKey, DsnpKeys, Config } from "./models";
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

        // Register the finalizer
        this.registerFinalizer();
    }

    getGraphHandle(): number {
        return this.handle;
    }

    getGraphConfig(environment: EnvironmentInterface): Promise<Config> {
        return graphsdkModule.getGraphConfig(environment);
    }

    getGraphCapacity(): Promise<number> {
        return graphsdkModule.getGraphCapacity(this.handle);
    }

    getGraphStatesCount(): Promise<number> {
        return graphsdkModule.getGraphStatesCount();
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

    forceCalculateGraphs(dsnpUserId: number): Promise<Update> {
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

    // Finalizer to free the graph state
    private registerFinalizer(): void {
        const finalizer = () => {
            this.freeGraphState();
        };

        // Register the finalizer
        if (typeof FinalizationRegistry !== "undefined") {
            const registry = new FinalizationRegistry<Graph>(() => {
                finalizer();
            });
            registry.register({}, this);
        } else if (typeof process !== "undefined" && typeof process.on === "function") {
            process.on("exit", finalizer);
        } else {
            console.warn("Unable to register finalizer. Memory may not be freed correctly.");
        }
    }

    freeGraphState(): Promise<void> {
        return graphsdkModule.freeGraphState(this.handle);
    }
    
    printHelloGraph(): void {
        console.log(graphsdkModule.printHelloGraph());
    }
}
