#ifndef GRAPH_SDK_FFI_H
#define GRAPH_SDK_FFI_H

#include <stdint.h>

typedef struct GraphState GraphState;
typedef struct UsersData UsersData;
typedef struct UserIds UserIds;
typedef struct Environment Environment;

Environment* graph_environment_new(const char* config_file_path);
void graph_environment_free(Environment* environment);

GraphState* graph_state_new(Environment* environment);
void graph_state_free(GraphState* graph_state);

int32_t graph_state_contains_user_graph(GraphState* graph_state, uint64_t user_id);
size_t graph_state_len(GraphState* graph_state);
void graph_state_remove_user_graph(GraphState* graph_state, uint64_t user_id);

#endif // GRAPH_SDK_FFI_H
