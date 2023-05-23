package io.amplica.graphsdk;

import io.amplica.graphsdk.models.Environment;

public class Graph implements NativeHandleGuard.Owner {
    private final long unsafeHandle;

    public Graph(Environment environment) {
        this.unsafeHandle = Native.initializeGraphState(environment.toByteArray());
    }

    @Override
    public long unsafeNativeHandleWithoutGuard() {
        return this.unsafeHandle;
    }

    @Override @SuppressWarnings("deprecation")
    public void finalize() {
        Native.freeGraphState(this.unsafeHandle);
    }
}
