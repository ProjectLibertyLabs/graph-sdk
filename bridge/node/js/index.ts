/*
    @dsnp/graphsdk-node
    Entry point for graphsdk.node bindings. This file exposes types and functions
    from the native module to the JS layer.
*/

import path from "path";

// load the native neon graphsdk module
const graphsdk = require(path.join(__dirname, "/dsnp_graph_sdk.node"));

console.log("Loaded graphsdk.node bindings");
// Export the printHelloGraph function
export const printHelloGraph = graphsdk.printHelloGraph;
