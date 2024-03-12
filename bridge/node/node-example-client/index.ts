import { Graph, EnvironmentInterface, Action, DsnpKeys, EnvironmentType, ConnectAction, Connection, ConnectionType, PrivacyType } from "@dsnp/graph-sdk";

const environment: EnvironmentInterface =  {environmentType: EnvironmentType.Mainnet};
const graph = new Graph(environment);

async function interactWithGraph() {
    let public_follow_graph_schema_id = await graph.getSchemaIdFromConfig(environment, ConnectionType.Follow, PrivacyType.Public);

    let connect_action: ConnectAction = {
        type: "Connect",
        ownerDsnpUserId: "1",
        connection: {
            dsnpUserId: "2",
            schemaId: public_follow_graph_schema_id,
        } as Connection,
        dsnpKeys: {
            dsnpUserId: "2",
            keysHash: 100,
            keys: [],
        } as DsnpKeys,
    } as ConnectAction;
    
    let actions = [] as Action[];
    actions.push(connect_action);

    let applied = await graph.applyActions(actions);
    console.log(applied);

    let connections_including_pending = await graph.getConnectionsForUserGraph("1", public_follow_graph_schema_id, true);
    console.log(connections_including_pending);

    let exported = await graph.exportUpdates();
    console.log(exported);
}
  
interactWithGraph();
