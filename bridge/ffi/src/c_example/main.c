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

    // Test 1: Initialize and clear states should work
    {
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
        ASSERT(connectionswithoutkeys.connections_len == 0, "Failed to get connections without keys");

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
    }

    // Test 2: State Capacity Test
    {
        size_t capacity = 10;
        Environment env;
        env.tag = Dev;
        env.dev = (Config) {
            .sdk_max_users_graph_size = (uint32_t) capacity
        };
        DsnpGraphStateResult_Error state_result = initialize_graph_state(&env);
        
        ASSERT(state_result.error == NULL, "Graph state initialization failed");
        GraphState* graphstate = state_result.result;

        DsnpGraphCountResult_Error capacity_result = get_graph_capacity(graphstate);
        ASSERT(capacity_result.error == NULL, "Failed to retrieve graph capacity");

        uintptr_t state_capacity = *(capacity_result.result);
        ASSERT(state_capacity == capacity, "State capacity is incorrect");

        free_graph_state(graphstate);
        free_dsnp_graph_error(capacity_result.error);
        free_dsnp_graph_error(state_result.error);
    }

    // Test 3: Initialize state with larger capacity should revert to smaller
    // ...
    {
          // Arrange
        size_t capacity = 10000;
        Environment env;
        env.tag = Mainnet;
        GraphState* state = NULL;

        // Act
        DsnpGraphStateResult_Error result = initialize_graph_state_with_capacity(&env, capacity);
        if (result.error != NULL) {
            // Handle initialization error
            printf("Error initializing graph state: %s\n", dsnp_graph_error_message(result.error));
            free_dsnp_graph_error(result.error);
            return 1;
        }
        state = result.result;

        // Assert
        DsnpGraphCountResult_Error count_result = get_graph_capacity(state);
        if (count_result.error != NULL) {
            // Handle capacity retrieval error
            printf("Error retrieving graph capacity: %s\n", dsnp_graph_error_message(count_result.error));
            free_dsnp_graph_error(count_result.error);
            free_graph_state(state);
            return 1;
        }
        size_t state_capacity = *(count_result.result);
        ASSERT(state_capacity < capacity, "State capacity is incorrect");

        // Clean up
        free_dsnp_graph_error(count_result.error);
        free_graph_state(state);
    }
    // Test 4: Import user data for public follow should import the graph successfully
    {
        Environment env;
        env.tag = Mainnet;
        GraphState* state = NULL;

        // Set up import data
        DsnpUserId dsnp_user_id_1 = 1;
        DsnpUserId dsnp_user_id_2 = 2;

        Connection connections_1[] = {{2, 0}, {3, 0}, {4, 0}, {5, 0}};
        size_t connections_1_len = sizeof(connections_1) / sizeof(Connection);

        Connection connections_2[] = {{10, 0}, {11, 0}, {12, 0}, {13, 0}};
        size_t connections_2_len = sizeof(connections_2) / sizeof(Connection);

        Connection* connections_ptr_1 = connections_1;
        Connection* connections_ptr_2 = connections_2;

        // Modify the page data for importbundle_1
        uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
        size_t page_data_1_content_len = 13;
        PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 0};

        PageData pages[] = {page_data_1};
        size_t pages_len = 1;

        ImportBundle importbundle_1 = {
            .dsnp_user_id = dsnp_user_id_1,
            .schema_id = 1,
            .key_pairs = NULL,
            .key_pairs_len = 0,
            .dsnp_keys = {dsnp_user_id_1, 0, NULL, 0},
            .pages = &pages[0],
            .pages_len = pages_len
        };
        ImportBundle importbundles[] = {importbundle_1};

        size_t importbundles_len = 1;

        // Initialize graph state
        DsnpGraphStateResult_Error state_result = initialize_graph_state(&env);
        ASSERT(state_result.error == NULL, "Graph state initialization failed");
        state = state_result.result;


        // Import user data
        DsnpGraphBooleanResult_Error importresult = graph_import_users_data(
            state, &importbundles[0], importbundles_len);
        ASSERT(importresult.error == NULL, "Failed to import users data");

        // Perform test assertions
        DsnpGraphCountResult_Error count_result = graph_users_count(state);
        ASSERT(count_result.error == NULL, "Failed to count users in graph");
        size_t userscount = *(count_result.result);
        ASSERT(userscount == 1, "Number of users in the graph is incorrect");

        DsnpGraphBooleanResult_Error contains_result_1 = graph_contains_user(state, &dsnp_user_id_1);
        ASSERT(contains_result_1.error == NULL, "Failed to check if graph contains user 1");
        bool contains_user_1 = *(contains_result_1.result);
        ASSERT(contains_user_1, "Graph should contain user 1");

        DsnpUserId invalid_user_id = dsnp_user_id_2 + 1;
        DsnpGraphBooleanResult_Error contains_result_invalid = graph_contains_user(state, &invalid_user_id);
        ASSERT(contains_result_invalid.error == NULL, "Failed to check if graph contains invalid user");
        bool contains_invalid_user = *(contains_result_invalid.result);
        ASSERT(!contains_invalid_user, "Graph should not contain invalid user");

        // Clean up
        free_dsnp_graph_error(importresult.error);
        free_dsnp_graph_error(contains_result_1.error);
        free_dsnp_graph_error(contains_result_2.error);
        free_dsnp_graph_error(contains_result_invalid.error);
        free_dsnp_graph_error(count_result.error);
        free_graph_state(state);
        free_dsnp_graph_error(state_result.error);
    }

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
