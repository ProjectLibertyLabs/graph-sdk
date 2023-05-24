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

   DsnpGraphStateResult_Error result = initialize_graph_state(&environment);
    ASSERT(result.error == NULL, "Graph state initialization failed");
    GraphState* graphstate = result.result;

    DsnpUserId userid;
    // Set the value of the userid
    // ...

   DsnpGraphBooleanResult_Error containsresult = graph_contains_user(graphstate, &userid);
    ASSERT(containsresult.error == NULL, "Failed to check if graph contains user");
    bool containsuser = *(containsresult.result);
    ASSERT(!containsuser, "Graph should not contain user before import");

   DsnpGraphCountResult_Error countresult = graph_users_count(graphstate);
    ASSERT(countresult.error == NULL, "Failed to count users in graph");
    size_t userscount = *(countresult.result);
    ASSERT(userscount == 0, "Number of users in the graph should be zero");

    ImportBundle importbundle;
    // Set the values of the importbundle struct
    // ...

   DsnpGraphBooleanResult_Error importresult = graph_import_users_data(graphstate, &importbundle, 1);
    ASSERT(importresult.error != NULL, "Expected error to import users data");
    // get the error message
    const char* errormessage = dsnp_graph_error_message(importresult.error);
    ASSERT(errormessage != NULL, "Failed to get error message");
    size_t errorcode = dsnp_graph_error_code(importresult.error);
    ASSERT(errorcode  < 1000, "Error code should be less than 1000");
    free_dsnp_graph_error(importresult.error);
    free_dsnp_graph_error_message(errormessage);

   DsnpGraphUpdatesResult_Error exportresult = graph_export_updates(graphstate);
    ASSERT(exportresult.error == NULL, "Failed to export updates");

   DsnpGraphConnectionsResult_Error connectionsresult = graph_get_connections_for_user(graphstate, &userid, NULL, true);
    ASSERT(connectionsresult.error != NULL, "Expected error to get connections for user");
    errormessage = dsnp_graph_error_message(connectionsresult.error);
    ASSERT(errormessage != NULL, "Failed to get error message");
    errorcode = dsnp_graph_error_code(connectionsresult.error);
    ASSERT(errorcode  < 1000, "Error code should be less than 1000");
    free_dsnp_graph_error(connectionsresult.error);
    free_dsnp_graph_error_message(errormessage);

   DsnpGraphConnectionsWithoutKeysResult_Error connectionswithoutkeysresult = graph_get_connections_without_keys(graphstate);
    ASSERT(connectionswithoutkeysresult.error == NULL, "Failed to get connections without keys");
    GraphConnectionsWithoutKeys connectionswithoutkeys = *(connectionswithoutkeysresult.result);
    // print 
    printf("Connections without keys: %zu\n", connectionswithoutkeys.connections_len);
    ASSERT(connectionswithoutkeys.connections_len == 0, "Failed to get connections without keys len");

   DsnpGraphConnectionsResult_Error onesidedconnectionsresult = graph_get_one_sided_private_friendship_connections(graphstate, &userid);
    ASSERT(onesidedconnectionsresult.error != NULL, "Expected error to get one sided private friendship connections");
    errormessage = dsnp_graph_error_message(onesidedconnectionsresult.error);
    ASSERT(errormessage != NULL, "Failed to get error message");
    errorcode = dsnp_graph_error_code(onesidedconnectionsresult.error);
    free_dsnp_graph_error(onesidedconnectionsresult.error);
    free_dsnp_graph_error_message(errormessage);

   DsnpGraphPublicKeysResult_Error publickeysresult = graph_get_public_keys(graphstate, &userid);
    ASSERT(publickeysresult.error == NULL, "Failed to get dsnp public keys");
    DsnpPublicKeys publickeys = *(publickeysresult.result);
    ASSERT(publickeys.keys_len == 0, "Failed to get dsnp public keys");

    free_graph_state(graphstate);
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
