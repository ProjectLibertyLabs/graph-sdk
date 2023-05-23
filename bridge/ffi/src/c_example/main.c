#include <stdio.h>
#include <stdbool.h>
#include "dsnp_graph_sdk_ffi.h"

#define ASSERT(condition, message) \
    do { \
        if (!(condition)) { \
            printf("Assertion failed: %s\n", message); \
            return 1; \
        } \
    } while (0)

int test_graph_sdk_ffi() {
    Environment environment;
    
    // ... setting up environment ...

    DsnpGraphFFIResult_GraphState__DsnpGraphError result = initialize_graph_state(&environment);
    ASSERT(result.error == NULL, "Graph state initialization failed");
    GraphState* graph_state = result.result;

    DsnpUserId user_id;
    // Set the value of the user_id
    // ...

    DsnpGraphFFIResult_bool__DsnpGraphError contains_result = graph_contains_user(graph_state, &user_id);
    ASSERT(contains_result.error == NULL, "Failed to check if graph contains user");
    bool contains_user = *(contains_result.result);
    ASSERT(!contains_user, "Graph should not contain user before import");

    DsnpGraphFFIResult_usize__DsnpGraphError count_result = graph_users_count(graph_state);
    ASSERT(count_result.error == NULL, "Failed to count users in graph");
    size_t users_count = *(count_result.result);
    ASSERT(users_count == 0, "Number of users in the graph should be zero");

    ImportBundle import_bundle;
    // Set the values of the import_bundle struct
    // ...

    DsnpGraphFFIResult_bool__DsnpGraphError import_result = graph_import_users_data(graph_state, &import_bundle, 1);
    ASSERT(import_result.error == NULL, "Graph data import failed");
    bool imported = *(import_result.result);
    ASSERT(!imported, "Graph data import should have failed");

    DsnpGraphFFIResult_GraphUpdates__DsnpGraphError export_result = graph_export_updates(graph_state);
    ASSERT(export_result.error == NULL, "Failed to export updates");
    GraphUpdates graph_updates = *(export_result.result);
    ASSERT(graph_updates.updates_len == 0, "Graph export updates failed");

    DsnpGraphFFIResult_GraphConnections__DsnpGraphError connections_result = graph_get_connections_for_user(graph_state, &user_id, NULL, true);
    ASSERT(connections_result.error == NULL, "Failed to get connections for user");
    GraphConnections connections = *(connections_result.result);
    ASSERT(connections.connections_len == 0, "Failed to get connections for user");

    DsnpGraphFFIResult_GraphConnectionsWithoutKeys__DsnpGraphError connections_without_keys_result = graph_get_connections_without_keys(graph_state);
    ASSERT(connections_without_keys_result.error == NULL, "Failed to get connections without keys");
    GraphConnectionsWithoutKeys connections_without_keys = *(connections_without_keys_result.result);
    ASSERT(connections_without_keys.connections_len == 0, "Failed to get connections without keys");

    DsnpGraphFFIResult_GraphConnections__DsnpGraphError one_sided_connections_result = graph_get_one_sided_private_friendship_connections(graph_state, &user_id);
    ASSERT(one_sided_connections_result.error == NULL, "Failed to get one-sided private friendship connections");
    GraphConnections one_sided_connections = *(one_sided_connections_result.result);
    ASSERT(one_sided_connections.connections_len == 0, "Failed to get one-sided private friendship connections");

    DsnpGraphFFIResult_DsnpPublicKeys__DsnpGraphError public_keys_result = graph_get_public_keys(graph_state, &user_id);
    ASSERT(public_keys_result.error == NULL, "Failed to get dsnp public keys");
    DsnpPublicKeys public_keys = *(public_keys_result.result);
    ASSERT(public_keys.keys_len == 0, "Failed to get dsnp public keys");

    free_graph_state(graph_state);

    return 0;
}

int main() {
    int result = test_graph_sdk_ffi();
    if (result == 0) {
        printf("All tests passed!\n");
    } else {
        printf("Some tests failed!\n");
    }

    return result;
}
