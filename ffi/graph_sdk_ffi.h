#ifndef GRAPH_SDK_FFI_H
#define GRAPH_SDK_FFI_H

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct GraphState GraphState;
typedef struct GraphAPI GraphAPI;
typedef struct DsnpGraphEdge DsnpGraphEdge;
typedef struct Environment Environment;
typedef struct ImportBundle ImportBundle;
typedef struct Update Update;
typedef struct Action Action;
typedef struct Config Config;
typedef enum ConnectionType ConnectionType;
typedef enum DsnpVersion DsnpVersion;
typedef enum EnvironmentType EnvironmentType;
typedef enum PrivacyType PrivacyType;
typedef uint16_t KeyType;

typedef bool (*ContainsUserGraphFn)(const GraphState*, const uint64_t*);
typedef size_t (*LenFn)(const GraphState*);
typedef void (*RemoveUserGraphFn)(const GraphState*, const uint64_t*);
typedef int32_t (*ImportUsersDataFn)(const GraphState*, const ImportBundle*, const size_t);
typedef int32_t (*ExportUpdatesFn)(const GraphState*, Update**, size_t*);
typedef int32_t (*ApplyActionsFn)(const GraphState*, const Action*, const size_t);
typedef int32_t (*GetConnectionsForUserGraphFn)(
    const GraphState*,
    const uint64_t*,
    const KeyType*,
    const bool,
    DsnpGraphEdge**,
    size_t*,
);
typedef int32_t (*GetConnectionsWithoutKeysFn)(const GraphState*, uint64_t**, size_t*);

struct GraphAPI {
    ContainsUserGraphFn contains_user_graph;
    LenFn len;
    RemoveUserGraphFn remove_user_graph;
    ImportUsersDataFn import_users_data;
    ExportUpdatesFn export_updates;
    ApplyActionsFn apply_actions;
    GetConnectionsForUserGraphFn get_connections_for_user_graph;
    GetConnectionsWithoutKeysFn get_connections_without_keys;
};

Environment* graph_environment_new(const char* config_file_path);
void graph_environment_free(Environment* environment);

GraphState* graph_state_new(Environment* environment);
void graph_state_free(GraphState* graph_state);

size_t graph_state_get_capacity(const GraphState* graph_state);
GraphState* graph_state_with_capacity(const Environment* environment, const size_t capacity);
bool graph_state_contains_user_graph(const GraphState* graph_state, const uint64_t* user_id);
size_t graph_state_len(const GraphState* graph_state);
void graph_state_remove_user_graph(const GraphState* graph_state, const uint64_t* user_id);
int32_t graph_state_import_users_data(
    const GraphState* graph_state,
    const ImportBundle* bundle,
    const size_t bundle_size,
);
int32_t graph_state_export_updates(
    const GraphState* graph_state,
    Update** updates_out,
    size_t* updates_out_len,
);
int32_t graph_state_apply_actions(
    const GraphState* graph_state,
    const Action* actions,
    const size_t actions_len,
);
int32_t graph_state_get_connections_for_user_graph(
    const GraphState* graph_state,
    const uint64_t* user_id,
    const KeyType* key_types,
    const bool with_keys,
    DsnpGraphEdge** edges_out,
    size_t* edges_out_len,
);
int32_t graph_state_get_connections_without_keys(
    const GraphState* graph_state,
    uint64_t** ids_out,
    size_t* ids_out_len,
);

#endif // GRAPH_SDK_FFI_H
