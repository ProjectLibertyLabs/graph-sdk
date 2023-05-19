package frequency;

public class NativeHandleGuard implements AutoCloseable {
    /**
     * @see NativeHandleGuard
     */
    public static interface Owner {
        long unsafeNativeHandleWithoutGuard();
    }

    private final Owner owner;

    public NativeHandleGuard(Owner owner) {
        this.owner = owner;
    }

    /**
     * Returns the native handle owned by the Java object, or 0 if the owner is {@code null}.
     */
    public long nativeHandle() {
        if (owner == null) {
            return 0;
        }
        return owner.unsafeNativeHandleWithoutGuard();
    }

    public void close() {
        // Act as an optimization barrier, so the whole guard doesn't get inlined away.
        // (In Java 9 we'd use Reference.reachabilityFence() for the same effect.)
        Native.keepAlive(this.owner);
    }
}