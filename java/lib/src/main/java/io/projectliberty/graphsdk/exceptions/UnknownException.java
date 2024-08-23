package io.projectliberty.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class UnknownException extends BaseGraphSdkException{
    public UnknownException() {}

    public UnknownException(String message) {
        super(message);
    }

    public UnknownException(Throwable throwable) {
        super(throwable);
    }

    public UnknownException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
