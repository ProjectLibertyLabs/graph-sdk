#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>
#include "graph_sdk_ffi.h"
typedef void (*print_hello_graph_t)(void);

int main() {
    // Load the Rust library
    void* handle = dlopen("libgraph_sdk_ffi.so", RTLD_LAZY);
    if (!handle) {
        printf("Error loading library: %s\n", dlerror());
        exit(1);
    }

    // Get a pointer to the print_hello_graph function
    print_hello_graph_t print_hello_graph_func = (print_hello_graph_t) dlsym(handle, "print_hello_graph");
    const char* dlsym_error = dlerror();
    if (dlsym_error) {
        printf("Error getting symbol: %s\n", dlsym_error);
        dlclose(handle);
        exit(1);
    }

    // Call the print_hello_graph function
    print_hello_graph_func();

    // Unload the Rust library
    dlclose(handle);
    return 0;
}
