package io.projectliberty.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class InvalidOutputException extends BaseGraphSdkException{
    public InvalidOutputException() {}

    public InvalidOutputException(String message) {
        super(message);
    }

    public InvalidOutputException(Throwable throwable) {
        super(throwable);
    }

    public InvalidOutputException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
