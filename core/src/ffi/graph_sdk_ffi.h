#ifndef GRAPH_SDK_FFI_H
#define GRAPH_SDK_FFI_H

#include <stdint.h>

int32_t graph_sdk_init(const char* config_file_path);
int32_t graph_sdk_perform_operation(int32_t param);

#endif // GRAPH_SDK_FFI_H
