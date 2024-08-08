import { graphsdkModule } from "./index";
import {
  ImportBundle,
  Update,
  DsnpGraphEdge,
  Action,
  DsnpPublicKey,
  DsnpKeys,
  Config,
  ConnectionType,
  PrivacyType,
  GraphKeyPair,
  ActionOptions,
} from "./models";
import { EnvironmentInterface } from "./models/environment";

export class Graph {
  /// while there is no guarantee that this will free the allocated resources, and we should always manually
  /// call freeGraphState for cleanup, having FinalizationRegistry might help in some cases that we forgot.
  static #finalizerInstance: FinalizationRegistry<number>;

  /// The handle to the native graph state
  private readonly handle: number;

  constructor(environment: EnvironmentInterface) {
    if (!Graph.#finalizerInstance) {
      Graph.#finalizerInstance = new FinalizationRegistry<number>(
        (handle: number) => {
          try {
            graphsdkModule.freeGraphState(handle);
          } catch {
            // nothing to do here
          }
        },
      );
    }
    const my_handle = graphsdkModule.initializeGraphState(environment);
    Graph.#finalizerInstance.register(this, my_handle);
    this.handle = my_handle;
  }

  getGraphHandle(): number {
    return this.handle;
  }

  getGraphConfig(environment: EnvironmentInterface): Config {
    return graphsdkModule.getGraphConfig(environment);
  }

  getSchemaIdFromConfig(
    environment: EnvironmentInterface,
    connectionType: ConnectionType,
    privacyType: PrivacyType,
  ): number {
    return graphsdkModule.getSchemaIdFromConfig(
      environment,
      connectionType,
      privacyType,
    );
  }

  getGraphStatesCount(): number {
    return graphsdkModule.getGraphStatesCount();
  }

  containsUserGraph(dsnpUserId: string): boolean {
    return graphsdkModule.containsUserGraph(this.handle, dsnpUserId);
  }

  getGraphUsersCount(): number {
    return graphsdkModule.getGraphUsersCount(this.handle);
  }

  removeUserGraph(dsnpUserId: string): boolean {
    return graphsdkModule.removeUserGraph(this.handle, dsnpUserId);
  }

  importUserData(payload: ImportBundle[]): boolean {
    return graphsdkModule.importUserData(this.handle, payload);
  }

  exportUpdates(): Update[] {
    return graphsdkModule.exportUpdates(this.handle);
  }

  exportUserGraphUpdates(dsnpUserId: string): Update[] {
    return graphsdkModule.exportUserGraphUpdates(this.handle, dsnpUserId);
  }

  getConnectionsForUserGraph(
    dsnpUserId: string,
    schemaId: number,
    includePending: boolean,
  ): DsnpGraphEdge[] {
    return graphsdkModule.getConnectionsForUserGraph(
      this.handle,
      dsnpUserId,
      schemaId,
      includePending,
    );
  }

  applyActions(actions: Action[], options?: ActionOptions): boolean {
    if (options) {
      return graphsdkModule.applyActions(this.handle, actions, options);
    }
    return graphsdkModule.applyActions(this.handle, actions);
  }

  commit(): void {
    return graphsdkModule.commit(this.handle);
  }

  rollback(): void {
    return graphsdkModule.rollback(this.handle);
  }

  forceCalculateGraphs(dsnpUserId: string): Update[] {
    return graphsdkModule.forceCalculateGraphs(this.handle, dsnpUserId);
  }

  getConnectionsWithoutKeys(): string[] {
    return graphsdkModule.getConnectionsWithoutKeys(this.handle);
  }

  getOneSidedPrivateFriendshipConnections(dsnpUserId: string): DsnpGraphEdge[] {
    return graphsdkModule.getOneSidedPrivateFriendshipConnections(
      this.handle,
      dsnpUserId,
    );
  }

  getPublicKeys(dsnpUserId: string): DsnpPublicKey[] {
    return graphsdkModule.getPublicKeys(this.handle, dsnpUserId);
  }

  static deserializeDsnpKeys(keys: DsnpKeys): DsnpPublicKey[] {
    return graphsdkModule.deserializeDsnpKeys(keys);
  }

  static generateKeyPair(keyType: number): GraphKeyPair {
    return graphsdkModule.generateKeyPair(keyType);
  }

  freeGraphState(): boolean {
    return graphsdkModule.freeGraphState(this.handle);
  }

  printHelloGraph(): void {
    console.log(graphsdkModule.printHelloGraph());
  }
}
