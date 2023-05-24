package frequency;

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
            String libraryName = System.mapLibraryName("dsnp_graph_sdk_jni");
            try (InputStream in = Native.class.getResourceAsStream("/" + libraryName)) {
                if (in != null) {
                    copyToTempFileAndLoad(in, libraryName);
                } else {
                    System.loadLibrary("dsnp_graph_sdk_jni");
                }
            }
        } catch (Exception e) {
            throw new RuntimeException(e);
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
     *
     * This can be used to keep a Java wrapper around a Rust object handle alive
     * while
     * earlier calls use that Rust object handle. That is, you should call
     * {@code keepAlive}
     * <em>after</em> the code where an object must not be garbage-collected.
     * However, most of the time {@link NativeHandleGuard} is a better choice,
     * since the lifetime of the guard is clear.
     *
     * Effectively equivalent to Java 9's <a href=
     * "https://docs.oracle.com/javase/9/docs/api/java/lang/ref/Reference.html#reachabilityFence-java.lang.Object-"><code>reachabilityFence()</code></a>.
     * Uses {@code native} because the JVM can't look into the implementation of the
     * method
     * and optimize away the use of {@code obj}. (The actual implementation does
     * nothing.)
     */
    public static native void keepAlive(Object obj);

    public static native String hello(String input);

    public static native void loggerInitialize(int max_level, Object loggerObject);

    public static native void loggerSetMaxLevel(int max_level);
}
