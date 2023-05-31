package io.amplica.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class InvalidHandleException extends BaseGraphSdkException{
    public InvalidHandleException() {}

    public InvalidHandleException(String message) {
        super(message);
    }

    public InvalidHandleException(Throwable throwable) {
        super(throwable);
    }

    public InvalidHandleException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
