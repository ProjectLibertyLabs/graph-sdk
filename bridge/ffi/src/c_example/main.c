#include <sodium.h>
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

int test_initialize_and_clear_states() {
    Environment environment;
    environment.tag = Mainnet;

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
    ASSERT(connectionswithoutkeys.connections != NULL, "Expected zero length");

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

    DsnpKeys keys;
    // Set the values of the keys struct
    // ...
    KeyData keydata;
    // Set the values of the keydata struct
    // ...
    keydata.index = 0;
    keydata.content = NULL;
    keydata.content_len = 0;

    keys.dsnp_user_id = userid;
    keys.keys = &keydata;
    keys.keys_len = 1;
    keys.keys_hash = 10;
    DsnpGraphPublicKeysResult_Error deserializepublickeysresult = graph_deserialize_dsnp_keys(&keys);
    ASSERT(deserializepublickeysresult.error != NULL, "Expected error to deserialize public keys");
    errormessage = dsnp_graph_error_message(deserializepublickeysresult.error);
    ASSERT(errormessage != NULL, "Failed to get error message");
    errorcode = dsnp_graph_error_code(deserializepublickeysresult.error);
    free_dsnp_graph_error(deserializepublickeysresult.error);
    free_dsnp_graph_error_message(errormessage);

    free_graph_state(graphstate);

    return 0;
}


int test_import_user_data_for_public_follow() {
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

    uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = 13;
    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

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
    ImportBundle importbundle_2 = {
        .dsnp_user_id = dsnp_user_id_2,
        .schema_id = 1,
        .key_pairs = NULL,
        .key_pairs_len = 0,
        .dsnp_keys = {dsnp_user_id_2, 0, NULL, 0},
        .pages = &pages[0],
        .pages_len = pages_len
    };
    ImportBundle importbundles[] = {importbundle_1, importbundle_2};
    size_t importbundles_len = 2;

    // Initialize graph state

    DsnpGraphStateResult_Error state_result = initialize_graph_state(&env);
    ASSERT(state_result.error == NULL, "Graph state initialization failed");
    state = state_result.result;

    // Import user data

    DsnpGraphBooleanResult_Error importresult = graph_import_users_data(state, &importbundles[0], importbundles_len);
    ASSERT(importresult.error == NULL, "Failed to import users data");

    // Perform the necessary assertions

    DsnpGraphCountResult_Error count_result = graph_users_count(state);
    ASSERT(count_result.error == NULL, "Failed to count users in graph");
    size_t userscount = *(count_result.result);
    ASSERT(userscount == 2, "Number of users in the graph is incorrect");

    DsnpGraphBooleanResult_Error contains_result_1 = graph_contains_user(state, &dsnp_user_id_1);
    ASSERT(contains_result_1.error == NULL, "Failed to check if graph contains user 1");
    bool contains_user_1 = *(contains_result_1.result);
    ASSERT(contains_user_1, "Graph should contain user 1");

    DsnpGraphBooleanResult_Error contains_result_2 = graph_contains_user(state, &dsnp_user_id_2);
    ASSERT(contains_result_2.error == NULL, "Failed to check if graph contains user 2");
    bool contains_user_2 = *(contains_result_2.result);
    ASSERT(contains_user_2, "Graph should contain user 2");

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

    return 0;
}

int test_add_bad_page_get_bad_response() {
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

    uint8_t page_data_1_content[] = {1, 2, 4, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = 1;
    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

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
    ImportBundle importbundle_2 = {
        .dsnp_user_id = dsnp_user_id_2,
        .schema_id = 1,
        .key_pairs = NULL,
        .key_pairs_len = 0,
        .dsnp_keys = {dsnp_user_id_2, 0, NULL, 0},
        .pages = &pages[0],
        .pages_len = pages_len
    };
    ImportBundle importbundles[] = {importbundle_1, importbundle_2};
    size_t importbundles_len = 2;

    // Initialize graph state

    DsnpGraphStateResult_Error state_result = initialize_graph_state(&env);
    ASSERT(state_result.error == NULL, "Graph state initialization failed");
    state = state_result.result;

    // Import user data

    DsnpGraphBooleanResult_Error importresult = graph_import_users_data(state, &importbundles[0], importbundles_len);
    ASSERT(importresult.error != NULL, "Failed to import users data");

    // Clean up

    free_graph_state(state);
    free_dsnp_graph_error(state_result.error);
    free_dsnp_graph_error(importresult.error);

    return 0;
}

int test_bad_schema_id_should_fail() {
    Environment env;
    env.tag = Mainnet;
    GraphState* state = NULL;

    // Set up import data

    DsnpUserId dsnp_user_id_1 = 1;

    uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = 13;
    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

    PageData pages[] = {page_data_1};
    size_t pages_len = 1;

    ImportBundle importbundle_1 = {
        .dsnp_user_id = dsnp_user_id_1,
        .schema_id = 1000,
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

    // Import user data

    DsnpGraphBooleanResult_Error importresult = graph_import_users_data(state_result.result, &importbundles[0], importbundles_len);
    ASSERT(importresult.error != NULL, "Expected import to fail due to bad schema id");

    // Clean up

    free_graph_state(state_result.result);
    free_dsnp_graph_error(state_result.error);
    free_dsnp_graph_error(importresult.error);

    return 0;
}

int test_import_user_data_with_invalid_serialized_public_key_should_fail() {
    Environment env;
    env.tag = Mainnet;
    GraphState* state = NULL;

    // Set up import data

    DsnpUserId dsnp_user_id = 1;

    uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = 13;

    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

    PageData pages[] = {page_data_1};
    size_t pages_len = 1;

    GraphKeyPair graph_key_pair = {
        .key_type = X25519,
        .secret_key = NULL,
        .secret_key_len = 0,
        .public_key = (const uint8_t[]){0, 1}, // invalid serialized public key
        .public_key_len = 2
    };

    ImportBundle import_bundle = {
        .dsnp_user_id = dsnp_user_id,
        .schema_id = 1, // Set the correct schema ID here
        .key_pairs = &graph_key_pair,
        .key_pairs_len = 1,
        .dsnp_keys = {dsnp_user_id, 0, NULL, 0},
        .pages = &pages[0],
        .pages_len = pages_len
    };

    // Initialize graph state

    DsnpGraphStateResult_Error state_result = initialize_graph_state(&env);
    ASSERT(state_result.error == NULL, "Graph state initialization failed");
    state = state_result.result;

    // Import user data

    DsnpGraphBooleanResult_Error import_result = graph_import_users_data(state, &import_bundle, 1);
    ASSERT(import_result.error != NULL, "Expected import to fail with invalid serialized public key");

    // Clean up

    free_graph_state(state);
    free_dsnp_graph_error(state_result.error);
    free_dsnp_graph_error(import_result.error);

    return 0;
}

int test_import_user_data_with_invalid_secret_fails(){
    Environment env;
    env.tag = Mainnet;
    GraphState* state = NULL;

    // Set up import data

    DsnpUserId dsnp_user_id = 1;

    uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = 13;

    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

    PageData pages[] = {page_data_1};
    size_t pages_len = 1;
    if (sodium_init() < 0) {
        printf("Failed to initialize libsodium\n");
        return 1;
    }
    unsigned char public_key[crypto_box_PUBLICKEYBYTES];
    unsigned char secret_key[crypto_box_SECRETKEYBYTES];

    if (crypto_box_keypair(public_key, secret_key) != 0) {
        printf("Failed to generate X25519 key pair\n");
        return 1;
    }

    GraphKeyPair graph_key_pair = {
        .key_type = X25519,
        .secret_key = (const uint8_t[]){0, 1}, // invalid serialized secret key
        .secret_key_len = 2,
        .public_key = public_key,
        .public_key_len = crypto_box_PUBLICKEYBYTES
    };

    ImportBundle import_bundle = {
        .dsnp_user_id = dsnp_user_id,
        .schema_id = 1, // Set the correct schema ID here
        .key_pairs = &graph_key_pair,
        .key_pairs_len = 1,
        .dsnp_keys = {dsnp_user_id, 0, NULL, 0},
        .pages = &pages[0],
        .pages_len = pages_len
    };

    // Initialize graph state

    DsnpGraphStateResult_Error state_result = initialize_graph_state(&env);
    ASSERT(state_result.error == NULL, "Graph state initialization failed");
    state = state_result.result;

    // Import user data

    DsnpGraphBooleanResult_Error import_result = graph_import_users_data(state, &import_bundle, 1);
    ASSERT(import_result.error != NULL, "Expected import to fail with invalid serialized public key");

    // Clean up

    free_graph_state(state);
    free_dsnp_graph_error(state_result.error);
    free_dsnp_graph_error(import_result.error);

    return 0;
}

int api_import_user_data_should_import_graph_for_private_follow_successfully() {
    // Arrange
    Environment env;
    env.tag = Mainnet;
    GraphState* state = initialize_graph_state(&env).result;

    unsigned char secret_key[crypto_box_SECRETKEYBYTES];
    unsigned char public_key[crypto_box_PUBLICKEYBYTES];
    crypto_box_keypair(public_key, secret_key);

    DsnpUserId dsnp_user_id = 1;
    uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = 13;

    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

    PageData pages[] = {page_data_1};
    size_t pages_len = 1;

    GraphKeyPair graph_key_pair = {
        .key_type = X25519,
        .secret_key = secret_key,
        .secret_key_len = sizeof(secret_key),
        .public_key = public_key,
        .public_key_len = sizeof(public_key)
    };
    ImportBundle import_bundle = {
        .dsnp_user_id = dsnp_user_id,
        .schema_id = 1, // Set the correct schema ID here
        .key_pairs = &graph_key_pair,
        .key_pairs_len = 1,
        .dsnp_keys = {dsnp_user_id, 0, NULL, 0},
        .pages = &pages[0],
        .pages_len = pages_len
    };

    // Act
    DsnpGraphBooleanResult_Error importresult = graph_import_users_data(state, &import_bundle, 1);

    // Assert
    ASSERT(importresult.error == NULL, "Failed to import users data");

    DsnpGraphCountResult_Error count_result = graph_users_count(state);
    ASSERT(count_result.error == NULL, "Failed to count users in graph");
    size_t userscount = *(count_result.result);
    ASSERT(userscount == 1, "Number of users in the graph is incorrect");

    DsnpGraphBooleanResult_Error contains_result = graph_contains_user(state, &dsnp_user_id);
    ASSERT(contains_result.error == NULL, "Failed to check if graph contains user");
    bool contains_user = *(contains_result.result);
    ASSERT(contains_user, "Graph should contain user");

    DsnpUserId invalid_user_id = dsnp_user_id + 1;
    DsnpGraphBooleanResult_Error contains_result_invalid = graph_contains_user(state, &invalid_user_id);
    ASSERT(contains_result_invalid.error == NULL, "Failed to check if graph contains invalid user");
    bool contains_invalid_user = *(contains_result_invalid.result);
    ASSERT(!contains_invalid_user, "Graph should not contain invalid user");

    // Clean up
    free_graph_state(state);
    free_dsnp_graph_error(importresult.error);
    free_dsnp_graph_error(contains_result.error);
    free_dsnp_graph_error(contains_result_invalid.error);
    free_dsnp_graph_error(count_result.error);


    return 0;
}

int api_import_user_data_with_wrong_encryption_keys_should_fail() {
    // Arrange
    Environment env;
    env.tag = Mainnet;
    GraphState* state = initialize_graph_state(&env).result;

    unsigned char resolved_key[crypto_secretbox_KEYBYTES];
    unsigned char secret_key[crypto_box_SECRETKEYBYTES];
    unsigned char public_key[crypto_box_PUBLICKEYBYTES];
    crypto_box_keypair(public_key, secret_key);

    DsnpUserId dsnp_user_id = 123;
    uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = sizeof(page_data_1_content);

    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

    PageData pages[] = {page_data_1};
    size_t pages_len = sizeof(pages) / sizeof(PageData);

    GraphKeyPair graph_key_pair = {
        .key_type = X25519,
        .secret_key = resolved_key,
        .secret_key_len = sizeof(resolved_key),
        .public_key = public_key,
        .public_key_len = sizeof(public_key)
    };
    ImportBundle import_bundle = {
        .dsnp_user_id = dsnp_user_id,
        .schema_id = 1, // Set the correct schema ID here
        .key_pairs = &graph_key_pair,
        .key_pairs_len = 1,
        .dsnp_keys = {dsnp_user_id, 0, NULL, 0},
        .pages = pages,
        .pages_len = pages_len
    };

    // Act
    DsnpGraphBooleanResult_Error import_result = graph_import_users_data(state, &import_bundle, 1);

    // Assert
    ASSERT(import_result.error != NULL, "Import should fail");

    DsnpGraphBooleanResult_Error contains_result = graph_contains_user(state, &dsnp_user_id);
    ASSERT(contains_result.error == NULL, "Failed to check if graph contains user");
    bool contains_user = *(contains_result.result);
    ASSERT(!contains_user, "Graph should not contain user");

    // Clean up
    free_graph_state(state);
    free_dsnp_graph_error(import_result.error);
    free_dsnp_graph_error(contains_result.error);

    return 0;
}

int api_remove_user_graph_should_remove_user_successfully() {
    // Arrange
    Environment env;
    env.tag = Mainnet;
    GraphState* state = initialize_graph_state(&env).result;

    DsnpUserId dsnp_user_id_1 = 1;
    uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = sizeof(page_data_1_content);

    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

    PageData pages[] = {page_data_1};
    size_t pages_len = sizeof(pages) / sizeof(PageData);

    ImportBundle import_bundle_1 = {
        .dsnp_user_id = dsnp_user_id_1,
        .schema_id = 1, // Set the correct schema ID here
        .key_pairs = NULL,
        .key_pairs_len = 0,
        .dsnp_keys = {dsnp_user_id_1, 0, NULL, 0},
        .pages = pages,
        .pages_len = pages_len
    };

    // Act
    DsnpGraphBooleanResult_Error import_result_1 = graph_import_users_data(state, &import_bundle_1, 1);
    ASSERT(import_result_1.error == NULL, "Failed to import user data");

    graph_remove_user(state, &dsnp_user_id_1);

    // Assert
    DsnpGraphCountResult_Error count_result = graph_users_count(state);
    ASSERT(count_result.error == NULL, "Failed to count users in graph");
    size_t users_count = *(count_result.result);
    ASSERT(users_count == 0, "Number of users in the graph is incorrect");

    DsnpGraphBooleanResult_Error contains_result = graph_contains_user(state, &dsnp_user_id_1);
    ASSERT(contains_result.error == NULL, "Failed to check if graph contains user");
    bool contains_user = *(contains_result.result);
    ASSERT(!contains_user, "Graph should not contain user");

    // Clean up
    free_graph_state(state);
    free_dsnp_graph_error(import_result_1.error);
    free_dsnp_graph_error(count_result.error);
    free_dsnp_graph_error(contains_result.error);

    return 0;
}

int api_apply_actions_should_work_as_expected_and_include_changes_in_pending() {
    // Arrange
    Environment env;
    env.tag = Mainnet;
    GraphState* state = initialize_graph_state(&env).result;

    DsnpUserId dsnp_user_id_1 = 1;
    uint8_t page_data_1_content[] = {24, 227, 96, 97, 96, 99, 224, 96, 224, 98, 96, 0, 0};
    size_t page_data_1_content_len = sizeof(page_data_1_content);

    PageData page_data_1 = {1, page_data_1_content, page_data_1_content_len, 10};

    PageData pages[] = {page_data_1};
    size_t pages_len = sizeof(pages) / sizeof(PageData);

    ImportBundle import_bundle_1 = {
        .dsnp_user_id = dsnp_user_id_1,
        .schema_id = 1, // Set the correct schema ID here
        .key_pairs = NULL,
        .key_pairs_len = 0,
        .dsnp_keys = {dsnp_user_id_1, 0, NULL, 0},
        .pages = pages,
        .pages_len = pages_len
    };

    DsnpGraphBooleanResult_Error import_result_1 = graph_import_users_data(state, &import_bundle_1, 1);
    ASSERT(import_result_1.error == NULL, "Failed to import user data");

    DsnpUserId dsnp_user_id_2 = 10;
    Connection connection_1 = {dsnp_user_id_2, 1};

    // Generate new public key using libsodium
    unsigned char new_public_key[crypto_box_PUBLICKEYBYTES];
    unsigned char new_secret_key[crypto_box_SECRETKEYBYTES];
    crypto_box_keypair(new_public_key, new_secret_key);

    Action add_graph_key_action = {
        .tag = AddGraphKey,
        .add_graph_key = {
            .owner_dsnp_user_id = dsnp_user_id_1,
            .new_public_key = new_public_key,
            .new_public_key_len = crypto_box_PUBLICKEYBYTES
        }
    };
    Action connect_action = {
        .tag = Connect,
        .connect = {
            .owner_dsnp_user_id = dsnp_user_id_1,
            .connection = connection_1,
            .dsnp_keys = NULL
        }
    };
    Action disconnect_action = {
        .tag = Disconnect,
        .disconnect = {
            .owner_dsnp_user_id = dsnp_user_id_1,
            .connection = {3, 1}
        }
    };
    Action actions[] = {add_graph_key_action, connect_action, disconnect_action};
    size_t actions_len = sizeof(actions) / sizeof(Action);

    DsnpGraphBooleanResult_Error apply_result = graph_apply_actions(state, actions, actions_len);
    ASSERT(apply_result.error == NULL, "Failed to apply actions");

    DsnpGraphConnectionsResult_Error connections_result = graph_get_connections_for_user(state, &dsnp_user_id_1, &connection_1.schema_id, true);
    ASSERT(connections_result.error == NULL, "Failed to get connections");
    GraphConnections* connections = connections_result.result;
    size_t expected_connections_len = 4;

    DsnpGraphEdge expected_connections[] = {
        {2, 1},
        {dsnp_user_id_2, 0},
        {4, 3},
        {5, 4}
    };
    ASSERT(connections->connections_len == expected_connections_len, "Number of connections is incorrect");

    // Clean up
    free_graph_state(state);
    free_dsnp_graph_error(import_result_1.error);
    free_dsnp_graph_error(apply_result.error);
    free_graph_connections(connections);

    return 0;
}

int main() {
    int result = 0;
int testno = 1;
    result += test_initialize_and_clear_states();
    result += test_import_user_data_for_public_follow();
    result += test_add_bad_page_get_bad_response();
    result += test_bad_schema_id_should_fail();
    result += test_import_user_data_with_invalid_serialized_public_key_should_fail();
    result += test_import_user_data_with_invalid_secret_fails();
    result += api_import_user_data_should_import_graph_for_private_follow_successfully();
    result += api_import_user_data_with_wrong_encryption_keys_should_fail();
    result += api_remove_user_graph_should_remove_user_successfully();
    result += api_apply_actions_should_work_as_expected_and_include_changes_in_pending();

    if (result == 0) {
        printf("All tests passed!\n");
    } else {
        printf("Some tests failed!\n");
    }

    return result;
}
