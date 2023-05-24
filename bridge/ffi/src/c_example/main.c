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

    DsnpGraphFFIResult_GraphState__DsnpGraphErrorHandle result = initialize_graph_state(&environment);
    ASSERT(result.error == NULL, "Graph state initialization failed");
    GraphState* graph_state = result.result;

    DsnpUserId user_id;
    // Set the value of the user_id
    // ...

    DsnpGraphFFIResult_bool__DsnpGraphErrorHandle contains_result = graph_contains_user(graph_state, &user_id);
    ASSERT(contains_result.error == NULL, "Failed to check if graph contains user");
    bool contains_user = *(contains_result.result);
    ASSERT(!contains_user, "Graph should not contain user before import");

    DsnpGraphFFIResult_usize__DsnpGraphErrorHandle count_result = graph_users_count(graph_state);
    ASSERT(count_result.error == NULL, "Failed to count users in graph");
    size_t users_count = *(count_result.result);
    ASSERT(users_count == 0, "Number of users in the graph should be zero");

    ImportBundle import_bundle;
    // Set the values of the import_bundle struct
    // ...

    DsnpGraphFFIResult_bool__DsnpGraphErrorHandle import_result = graph_import_users_data(graph_state, &import_bundle, 1);
    ASSERT(import_result.error != NULL, "Expected error to import users data");
    // get the error message
    const char* error_message = dsnp_graph_error_message(import_result.error);
    ASSERT(error_message != NULL, "Failed to get error message");
    size_t error_code = dsnp_graph_error_code(import_result.error);
    ASSERT(error_code  < 1000, "Error code should be less than 1000");
    free_dsnp_graph_error(import_result.error);

    DsnpGraphFFIResult_GraphUpdates__DsnpGraphErrorHandle export_result = graph_export_updates(graph_state);
    ASSERT(export_result.error == NULL, "Failed to export updates");

    DsnpGraphFFIResult_GraphConnections__DsnpGraphErrorHandle connections_result = graph_get_connections_for_user(graph_state, &user_id, NULL, true);
    ASSERT(connections_result.error != NULL, "Expected error to get connections for user");
    error_message = dsnp_graph_error_message(connections_result.error);
    ASSERT(error_message != NULL, "Failed to get error message");
    error_code = dsnp_graph_error_code(connections_result.error);
    ASSERT(error_code  < 1000, "Error code should be less than 1000");
    free_dsnp_graph_error(connections_result.error);

    DsnpGraphFFIResult_GraphConnectionsWithoutKeys__DsnpGraphErrorHandle connections_without_keys_result = graph_get_connections_without_keys(graph_state);
    ASSERT(connections_without_keys_result.error == NULL, "Failed to get connections without keys");
    GraphConnectionsWithoutKeys connections_without_keys = *(connections_without_keys_result.result);
    ASSERT(connections_without_keys.connections_len == 0, "Failed to get connections without keys");

    DsnpGraphFFIResult_GraphConnections__DsnpGraphErrorHandle one_sided_connections_result = graph_get_one_sided_private_friendship_connections(graph_state, &user_id);
    ASSERT(one_sided_connections_result.error != NULL, "Expected error to get one sided private friendship connections");
    error_message = dsnp_graph_error_message(one_sided_connections_result.error);
    ASSERT(error_message != NULL, "Failed to get error message");
    error_code = dsnp_graph_error_code(one_sided_connections_result.error);
    free_dsnp_graph_error(one_sided_connections_result.error);

    DsnpGraphFFIResult_DsnpPublicKeys__DsnpGraphErrorHandle public_keys_result = graph_get_public_keys(graph_state, &user_id);
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
