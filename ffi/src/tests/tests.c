#include "graph_sdk_ffi.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

Environment* create_environment() {
    // Allocate memory for the environment
    Environment* env = malloc(sizeof(Environment));

    // Set the environment type to "dev"
    env->environment_type = Dev;

    // Allocate memory for the config
    env->config.sdk_max_users_graph_size = 1000;
    env->config.max_graph_page_size_bytes = 100000;
    env->config.max_page_id = 1000000;
    env->config.max_key_page_size_bytes = 10000;
    env->config.schema_map_len = 1;
    env->config.schema_map = malloc(sizeof(SchemaConfigTuple) * env->config.schema_map_len);
    env->config.schema_map[0].schema_id = 1;
    env->config.schema_map[0].schema_config.dsnp_version = Version1_0;
    env->config.schema_map[0].schema_config.connection_type.tag = Follow;
    env->config.schema_map[0].schema_config.connection_type.follow = Public;
    env->config.dsnp_versions_len = 1;
    env->config.dsnp_versions = malloc(sizeof(DsnpVersion) * env->config.dsnp_versions_len);
    env->config.dsnp_versions[0] = Version1_0;

    return env;
}


int main() {
    // Create a new environment
    Environment* environment = create_environment();

    // Create a new graph state
    GraphState* graph_state = graph_state_new(environment);

    // Test that the graph state was created successfully
    if (graph_state == NULL) {
        printf("Error: graph_state_new returned NULL\n");
        return EXIT_FAILURE;
    }

    // Free the graph state
    graph_state_free(graph_state);

    printf("Tests passed successfully!\n");
    return EXIT_SUCCESS;
}
