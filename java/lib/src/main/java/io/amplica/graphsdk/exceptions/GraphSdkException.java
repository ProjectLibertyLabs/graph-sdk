package io.amplica.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class GraphSdkException extends BaseGraphSdkException{
    public GraphSdkException() {}

    public GraphSdkException(String message) {
        super(message);
    }

    public GraphSdkException(Throwable throwable) {
        super(throwable);
    }

    public GraphSdkException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
