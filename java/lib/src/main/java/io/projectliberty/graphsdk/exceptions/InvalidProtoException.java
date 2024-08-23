package io.projectliberty.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class InvalidProtoException extends BaseGraphSdkException{
    public InvalidProtoException() {}

    public InvalidProtoException(String message) {
        super(message);
    }

    public InvalidProtoException(Throwable throwable) {
        super(throwable);
    }

    public InvalidProtoException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
