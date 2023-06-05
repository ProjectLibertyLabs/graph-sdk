package io.amplica.graphsdk.exceptions;

/**
 * Base Exception class for Graph Sdk
 */
public abstract class BaseGraphSdkException extends Exception{
    public BaseGraphSdkException() {}

    public BaseGraphSdkException(String message) {
        super(message);
    }

    public BaseGraphSdkException(Throwable throwable) {
        super(throwable);
    }

    public BaseGraphSdkException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
