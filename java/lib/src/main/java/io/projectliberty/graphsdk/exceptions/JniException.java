package io.projectliberty.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class JniException extends BaseGraphSdkException{
    public JniException() {}

    public JniException(String message) {
        super(message);
    }

    public JniException(Throwable throwable) {
        super(throwable);
    }

    public JniException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
