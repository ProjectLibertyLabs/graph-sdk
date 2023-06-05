package io.amplica.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class UnexpectedResponseException extends BaseGraphSdkException{
    public UnexpectedResponseException() {}

    public UnexpectedResponseException(String message) {
        super(message);
    }

    public UnexpectedResponseException(Throwable throwable) {
        super(throwable);
    }

    public UnexpectedResponseException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
