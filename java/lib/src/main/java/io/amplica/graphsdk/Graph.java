package io.amplica.graphsdk;

import com.google.protobuf.InvalidProtocolBufferException;
import io.amplica.graphsdk.models.*;

import java.util.List;


public class Graph implements NativeHandleGuard.Owner {
    private final long unsafeHandle;
    private final Configuration configuration;

    public Graph(Configuration configuration) {
        this.configuration = configuration;
        this.unsafeHandle = Native.initializeGraphState(configuration.getEnvironment().toByteArray());
    }

    public boolean containsUserGraph(long dsnpUserId) throws Exception {
        return Native.containsUserGraph(this.unsafeHandle, dsnpUserId);
    }

    public int getUsersLength() throws Exception {
        return Native.getGraphUsersLength(this.unsafeHandle);
    }

    public void removeUserGraph(long dsnpUserId) throws Exception {
        Native.removeUserGraph(this.unsafeHandle, dsnpUserId);
    }

    public void importUserData(ImportBundles bundle) throws Exception {
        Native.importUserData(this.unsafeHandle, bundle.toByteArray());
    }

    public List<Updates.Update> exportUpdates() throws Exception {
        var raw = Native.exportUpdates(this.unsafeHandle);
        return Updates.parseFrom(raw).getUpdateList();
    }

    public void applyActions(Actions actions) throws Exception {
        Native.applyActions(this.unsafeHandle, actions.toByteArray());
    }

    public List<Updates.Update> forceRecalculateGraph(long dsnpUserId) throws Exception {
        var raw = Native.forceCalculateGraphs(this.unsafeHandle, dsnpUserId);
        return Updates.parseFrom(raw).getUpdateList();
    }

    public List<DsnpGraphEdges.DsnpGraphEdge> getConnections(long dsnpUserId, ConnectionType connectionType, boolean includePending) throws InvalidProtocolBufferException {
        var raw = Native.getConnectionsForUserGraph(this.unsafeHandle, dsnpUserId, this.configuration.getSchemaId(connectionType), includePending);
        return DsnpGraphEdges.parseFrom(raw).getEdgeList();
    }

    public List<Long> getUsersWithoutImportedKeys() throws InvalidProtocolBufferException {
        var raw = Native.getUsersWithoutKeys(this.unsafeHandle);
        return DsnpUsers.parseFrom(raw).getUserList();
    }

    // TODO: add test
    public List<DsnpGraphEdges.DsnpGraphEdge> getOneSidedPrivateFriendships(long dsnpUserId) throws InvalidProtocolBufferException {
        var raw = Native.getOneSidedPrivateFriendshipConnections(this.unsafeHandle, dsnpUserId);
        return DsnpGraphEdges.parseFrom(raw).getEdgeList();
    }

    public List<DsnpPublicKeys.DsnpPublicKey> getPublicKeys(long dsnpUserId) throws InvalidProtocolBufferException {
        var raw = Native.getPublicKeys(this.unsafeHandle, dsnpUserId);
        return DsnpPublicKeys.parseFrom(raw).getPublicKeyList();
    }

    @Override
    public long unsafeNativeHandleWithoutGuard() {
        return this.unsafeHandle;
    }

    @Override
    @SuppressWarnings("deprecation")
    public void finalize() {
        Native.freeGraphState(this.unsafeHandle);
    }
}
