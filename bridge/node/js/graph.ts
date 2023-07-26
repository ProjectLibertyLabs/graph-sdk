import { graphsdkModule } from "./index";
import { ImportBundle, Update, DsnpGraphEdge, Action, DsnpPublicKey, DsnpKeys, Config, ConnectionType, PrivacyType, GraphKeyPair, ActionOptions } from "./models";
import { EnvironmentInterface } from "./models/environment";

export class Graph {
    /// The handle to the native graph state
    private handle: number;

    constructor(environment: EnvironmentInterface) {
        this.handle = graphsdkModule.initializeGraphState(environment);
    }

    getGraphHandle(): number {
        return this.handle;
    }

    // getGraphConfig(environment: EnvironmentInterface): Promise<Config> {
    getGraphConfig(environment: EnvironmentInterface): Config {
        return graphsdkModule.getGraphConfig(environment);
    }

    // getSchemaIdFromConfig(environment: EnvironmentInterface, connectionType: ConnectionType, privacyType: PrivacyType): Promise<number> {
    getSchemaIdFromConfig(environment: EnvironmentInterface, connectionType: ConnectionType, privacyType: PrivacyType): number {
        return graphsdkModule.getSchemaIdFromConfig(environment, connectionType, privacyType);
    }

    // getGraphStatesCount(): Promise<number> {
    getGraphStatesCount(): number {
        return graphsdkModule.getGraphStatesCount();
    }

    // containsUserGraph(dsnpUserId: string): Promise<boolean> {
    containsUserGraph(dsnpUserId: string): boolean {
        return graphsdkModule.containsUserGraph(this.handle, dsnpUserId);
    }

    // getGraphUsersCount(): Promise<number> {
    getGraphUsersCount(): number {
        return graphsdkModule.getGraphUsersCount(this.handle);
    }

    // removeUserGraph(dsnpUserId: string): Promise<boolean> {
    removeUserGraph(dsnpUserId: string): boolean {
        return graphsdkModule.removeUserGraph(this.handle, dsnpUserId);
    }

    // importUserData(payload: ImportBundle[]): Promise<boolean> {
    importUserData(payload: ImportBundle[]): boolean {
        return graphsdkModule.importUserData(this.handle, payload);
    }

    // exportUpdates(): Promise<Update[]> {
    exportUpdates(): Update[] {
        return graphsdkModule.exportUpdates(this.handle);
    }

    // exportUserGraphUpdates(dsnpUserId: string): Promise<Update[]> {
    exportUserGraphUpdates(dsnpUserId: string): Update[] {
        return graphsdkModule.exportUserGraphUpdates(this.handle, dsnpUserId);
    }

    // getConnectionsForUserGraph(dsnpUserId: string, schemaId: number, includePending: boolean): Promise<DsnpGraphEdge[]> {
    getConnectionsForUserGraph(dsnpUserId: string, schemaId: number, includePending: boolean): DsnpGraphEdge[] {
        return graphsdkModule.getConnectionsForUserGraph(this.handle, dsnpUserId, schemaId, includePending);
    }

    applyActions(actions: Action[], options?: ActionOptions): boolean {
        if (options) {
            return graphsdkModule.applyActions(this.handle, actions, options);
        }
        return graphsdkModule.applyActions(this.handle, actions);
    }

    // forceCalculateGraphs(dsnpUserId: string): Promise<Update[]> {
    forceCalculateGraphs(dsnpUserId: string): Update[] {
        return graphsdkModule.forceCalculateGraphs(this.handle, dsnpUserId);
    }

    // getConnectionsWithoutKeys(): Promise<string[]> {
    getConnectionsWithoutKeys(): string[] {
        return graphsdkModule.getConnectionsWithoutKeys(this.handle);
    }

    // getOneSidedPrivateFriendshipConnections(dsnpUserId: string): Promise<DsnpGraphEdge[]> {
    getOneSidedPrivateFriendshipConnections(dsnpUserId: string): DsnpGraphEdge[] {
        return graphsdkModule.getOneSidedPrivateFriendshipConnections(this.handle, dsnpUserId);
    }

    // getPublicKeys(dsnpUserId: string): Promise<DsnpPublicKey[]> {
    getPublicKeys(dsnpUserId: string): DsnpPublicKey[] {
        return graphsdkModule.getPublicKeys(this.handle, dsnpUserId);
    }

    // static deserializeDsnpKeys(keys: DsnpKeys): Promise<DsnpPublicKey[]> {
    static deserializeDsnpKeys(keys: DsnpKeys): DsnpPublicKey[] {
        return graphsdkModule.deserializeDsnpKeys(keys);
    }

    // static generateKeyPair(keyType: number): Promise<GraphKeyPair> {
    static generateKeyPair(keyType: number): GraphKeyPair {
        return graphsdkModule.generateKeyPair(keyType);
    }

    // freeGraphState(): Promise<boolean> {
    freeGraphState(): boolean {
        return graphsdkModule.freeGraphState(this.handle);
    }

    printHelloGraph(): void {
        console.log(graphsdkModule.printHelloGraph());
    }
}
