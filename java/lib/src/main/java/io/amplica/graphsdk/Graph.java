package io.amplica.graphsdk;

import com.google.protobuf.InvalidProtocolBufferException;
import io.amplica.graphsdk.exceptions.BaseGraphSdkException;
import io.amplica.graphsdk.models.*;
import io.amplica.graphsdk.models.ImportBundles.ImportBundle.GraphKeyPair;

import java.util.List;

public class Graph implements NativeHandleGuard.Owner {
    private long unsafeHandle;
    private final Configuration configuration;

    public Graph(Configuration configuration) throws BaseGraphSdkException {
        this.configuration = configuration;
        this.unsafeHandle = Native.initializeGraphState(configuration.getEnvironment().toByteArray());
    }

    public void setUnsafeHandle(long unsafeHandle) {
        this.unsafeHandle = unsafeHandle;
    }

    public boolean containsUserGraph(long dsnpUserId) throws BaseGraphSdkException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            return Native.containsUserGraph(guard.nativeHandle(), dsnpUserId);
        }
    }

    public int getUsersLength() throws BaseGraphSdkException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            return Native.getGraphUsersLength(guard.nativeHandle());
        }
    }

    public void removeUserGraph(long dsnpUserId) throws BaseGraphSdkException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            Native.removeUserGraph(guard.nativeHandle(), dsnpUserId);
        }
    }

    public void importUserData(ImportBundles bundle) throws BaseGraphSdkException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            Native.importUserData(guard.nativeHandle(), bundle.toByteArray());
        }
    }

    public List<Updates.Update> exportUpdates() throws BaseGraphSdkException, InvalidProtocolBufferException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            var raw = Native.exportUpdates(guard.nativeHandle());
            return Updates.parseFrom(raw).getUpdateList();
        }
    }

    public List<Updates.Update> exportUserGraphUpdates(long dsnpUserId)
            throws BaseGraphSdkException, InvalidProtocolBufferException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            var raw = Native.exportUserGraphUpdates(guard.nativeHandle(), dsnpUserId);
            return Updates.parseFrom(raw).getUpdateList();
        }
    }

    public void applyActions(Actions actions) throws BaseGraphSdkException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            Native.applyActions(guard.nativeHandle(), actions.toByteArray());
        }
    }

    public void commit() throws BaseGraphSdkException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            Native.commit(guard.nativeHandle());
        }
    }

    public void rollback() throws BaseGraphSdkException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            Native.rollback(guard.nativeHandle());
        }
    }

    public List<Updates.Update> forceRecalculateGraph(long dsnpUserId)
            throws BaseGraphSdkException, InvalidProtocolBufferException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            var raw = Native.forceCalculateGraphs(guard.nativeHandle(), dsnpUserId);
            return Updates.parseFrom(raw).getUpdateList();
        }
    }

    public List<DsnpGraphEdges.DsnpGraphEdge> getConnections(long dsnpUserId, ConnectionType connectionType,
            boolean includePending) throws BaseGraphSdkException, InvalidProtocolBufferException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            var raw = Native.getConnectionsForUserGraph(guard.nativeHandle(), dsnpUserId,
                    this.configuration.getSchemaId(connectionType), includePending);
            return DsnpGraphEdges.parseFrom(raw).getEdgeList();
        }
    }

    public List<Long> getUsersWithoutImportedKeys() throws BaseGraphSdkException, InvalidProtocolBufferException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            var raw = Native.getUsersWithoutKeys(guard.nativeHandle());
            return DsnpUsers.parseFrom(raw).getUserList();
        }
    }

    // TODO: add test
    public List<DsnpGraphEdges.DsnpGraphEdge> getOneSidedPrivateFriendships(long dsnpUserId)
            throws BaseGraphSdkException, InvalidProtocolBufferException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            var raw = Native.getOneSidedPrivateFriendshipConnections(guard.nativeHandle(), dsnpUserId);
            return DsnpGraphEdges.parseFrom(raw).getEdgeList();
        }
    }

    public List<DsnpPublicKeys.DsnpPublicKey> getPublicKeys(long dsnpUserId)
            throws BaseGraphSdkException, InvalidProtocolBufferException {
        try (NativeHandleGuard guard = new NativeHandleGuard(this)) {
            var raw = Native.getPublicKeys(guard.nativeHandle(), dsnpUserId);
            return DsnpPublicKeys.parseFrom(raw).getPublicKeyList();
        }
    }

    public static List<DsnpPublicKeys.DsnpPublicKey> deserializeDsnpKeys(DsnpKeys keys)
            throws BaseGraphSdkException, InvalidProtocolBufferException {
        var raw = Native.deserializeDsnpKeys(keys.toByteArray());
        return DsnpPublicKeys.parseFrom(raw).getPublicKeyList();
    }

    public static GraphKeyPair generateKeyPair(GraphKeyType key_type)
            throws BaseGraphSdkException, InvalidProtocolBufferException {
        var raw = Native.generateKeyPair(key_type.getNumber());
        return GraphKeyPair.parseFrom(raw);
    }

    @Override
    public long unsafeNativeHandleWithoutGuard() {
        return this.unsafeHandle;
    }

    @Override
    @SuppressWarnings("deprecation")
    protected void finalize() {
        Native.freeGraphState(this.unsafeHandle);
    }
}
