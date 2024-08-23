package io.projectliberty.graphsdk;

import java.io.*;
import java.nio.file.Files;

public final class Native {
    static {
        loadLibrary();
    }

    private Native() {
    }

    private static void loadLibrary() {
        try {
            String target = getTarget();
            String libraryName = System.mapLibraryName("dsnp_graph_sdk_jni");
            String resourcePath = "/" + target + "_" + libraryName;
            try (InputStream in = Native.class.getResourceAsStream(resourcePath)) {
                if (in != null) {
                    copyToTempFileAndLoad(in, libraryName);
                } else {
                    // if it didn't find native, it will fallback to default library name
                    // which doesn't include the target. This is useful in local development
                    String fallbackResourcePath = "/" + libraryName;
                    try (InputStream fallbackIn = Native.class.getResourceAsStream(fallbackResourcePath)) {
                        if (fallbackIn != null) {
                            copyToTempFileAndLoad(fallbackIn, libraryName);
                        } else {
                            System.loadLibrary("dsnp_graph_sdk_jni");
                        }
                    }
                }
            }
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    private static String getTarget() {
        String os = System.getProperty("os.name", "unknown").toLowerCase();
        String arch = System.getProperty("os.arch", "unknown").toLowerCase();

        // Windows
        if (os.contains("win") && (arch.contains("amd64") || arch.contains("x86_64"))) {
            return "x86_64-pc-windows-msvc";

            // MacOS
        } else if (os.contains("mac") && (arch.contains("amd64") || arch.contains("x86_64"))) {
            return "x86_64-apple-darwin";
        } else if (os.contains("mac") && arch.contains("aarch64")) {
            return "aarch64-apple-darwin";

            // Linux
        } else if (os.contains("nux") && (arch.contains("amd64") || arch.contains("x86_64"))) {
            return "x86_64-unknown-linux-gnu";
        } else if (os.contains("nux") && arch.contains("aarch64")) {
            return "aarch64-unknown-linux-gnu";
        } else {
            throw new RuntimeException("OS: " + os + " and ARCH:" + arch + " native build does not exist!");
        }
    }

    private static void copyToTempFileAndLoad(InputStream in, String name) throws IOException {
        File tempFile = Files.createTempFile(null, name).toFile();
        tempFile.deleteOnExit();

        try (OutputStream out = new FileOutputStream(tempFile)) {
            byte[] buffer = new byte[4096];
            int read;

            while ((read = in.read(buffer)) != -1) {
                out.write(buffer, 0, read);
            }
        }
        System.load(tempFile.getAbsolutePath());
    }

    /**
     * Keeps an object from being garbage-collected until this call completes.
     * <p>
     * This can be used to keep a Java wrapper around a Rust object handle alive
     * while
     * earlier calls use that Rust object handle. That is, you should call
     * {@code keepAlive}
     * <em>after</em> the code where an object must not be garbage-collected.
     * However, most of the time {@link NativeHandleGuard} is a better choice,
     * since the lifetime of the guard is clear.
     * <p>
     * Effectively equivalent to Java 9's <a href=
     * "https://docs.oracle.com/javase/9/docs/api/java/lang/ref/Reference.html#reachabilityFence-java.lang.Object-"><code>reachabilityFence()</code></a>.
     * Uses {@code native} because the JVM can't look into the implementation of the
     * method
     * and optimize away the use of {@code obj}. (The actual implementation does
     * nothing.)
     */
    public static native void keepAlive(Object obj);

    public static native void loggerInitialize(int max_level, Object loggerObject);

    public static native void loggerSetMaxLevel(int max_level);

    public static native long initializeGraphState(byte[] environment);

    public static native long freeGraphState(long stateHandle);

    public static native byte[] getConfig(byte[] environment);

    public static native boolean containsUserGraph(long stateHandle, long dsnpUserId);

    public static native int getGraphUsersLength(long stateHandle);

    public static native void removeUserGraph(long stateHandle, long dsnpUserId);

    public static native void importUserData(long stateHandle, byte[] imports);

    public static native byte[] exportUpdates(long stateHandle);

    public static native byte[] exportUserGraphUpdates(long stateHandle, long dsnpUserId);

    public static native void applyActions(long stateHandle, byte[] actions);

    public static native void commit(long stateHandle);

    public static native void rollback(long stateHandle);

    public static native byte[] forceCalculateGraphs(long stateHandle, long dsnpUserId);

    public static native byte[] getConnectionsForUserGraph(long stateHandle, long dsnpUserId, int schemaId,
            boolean includePending);

    public static native byte[] getUsersWithoutKeys(long stateHandle);

    public static native byte[] getOneSidedPrivateFriendshipConnections(long stateHandle, long dsnpUserId);

    public static native byte[] getPublicKeys(long stateHandle, long dsnpUserId);

    public static native byte[] deserializeDsnpKeys(byte[] dsnpKeys);

    public static native byte[] generateKeyPair(long graph_key_type);
}
