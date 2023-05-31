package io.amplica.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class InvalidRequestException extends BaseGraphSdkException{
    public InvalidRequestException() {}

    public InvalidRequestException(String message) {
        super(message);
    }

    public InvalidRequestException(Throwable throwable) {
        super(throwable);
    }

    public InvalidRequestException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
