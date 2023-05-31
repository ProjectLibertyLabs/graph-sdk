package io.amplica.graphsdk.exceptions;

/**
 * Warning: If moved or renamed, the Rust glue code in JNI bridge MUST be synced
 */
public class AcquiringLockException extends BaseGraphSdkException{
    public AcquiringLockException() {}

    public AcquiringLockException(String message) {
        super(message);
    }

    public AcquiringLockException(Throwable throwable) {
        super(throwable);
    }

    public AcquiringLockException(String message, Throwable throwable) {
        super(message, throwable);
    }
}
