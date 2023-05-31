package io.amplica.graphsdk;

import com.google.protobuf.InvalidProtocolBufferException;
import io.amplica.graphsdk.exceptions.BaseGraphSdkException;
import io.amplica.graphsdk.models.*;

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
        return Native.containsUserGraph(this.unsafeHandle, dsnpUserId);
    }

    public int getUsersLength() throws BaseGraphSdkException {
        return Native.getGraphUsersLength(this.unsafeHandle);
    }

    public void removeUserGraph(long dsnpUserId) throws BaseGraphSdkException {
        Native.removeUserGraph(this.unsafeHandle, dsnpUserId);
    }

    public void importUserData(ImportBundles bundle) throws BaseGraphSdkException {
        Native.importUserData(this.unsafeHandle, bundle.toByteArray());
    }

    public List<Updates.Update> exportUpdates() throws BaseGraphSdkException, InvalidProtocolBufferException {
        var raw = Native.exportUpdates(this.unsafeHandle);
        return Updates.parseFrom(raw).getUpdateList();
    }

    public void applyActions(Actions actions) throws BaseGraphSdkException {
        Native.applyActions(this.unsafeHandle, actions.toByteArray());
    }

    public List<Updates.Update> forceRecalculateGraph(long dsnpUserId) throws BaseGraphSdkException, InvalidProtocolBufferException {
        var raw = Native.forceCalculateGraphs(this.unsafeHandle, dsnpUserId);
        return Updates.parseFrom(raw).getUpdateList();
    }

    public List<DsnpGraphEdges.DsnpGraphEdge> getConnections(long dsnpUserId, ConnectionType connectionType, boolean includePending) throws BaseGraphSdkException, InvalidProtocolBufferException {
        var raw = Native.getConnectionsForUserGraph(this.unsafeHandle, dsnpUserId, this.configuration.getSchemaId(connectionType), includePending);
        return DsnpGraphEdges.parseFrom(raw).getEdgeList();
    }

    public List<Long> getUsersWithoutImportedKeys() throws BaseGraphSdkException, InvalidProtocolBufferException {
        var raw = Native.getUsersWithoutKeys(this.unsafeHandle);
        return DsnpUsers.parseFrom(raw).getUserList();
    }

    // TODO: add test
    public List<DsnpGraphEdges.DsnpGraphEdge> getOneSidedPrivateFriendships(long dsnpUserId) throws BaseGraphSdkException, InvalidProtocolBufferException {
        var raw = Native.getOneSidedPrivateFriendshipConnections(this.unsafeHandle, dsnpUserId);
        return DsnpGraphEdges.parseFrom(raw).getEdgeList();
    }

    public List<DsnpPublicKeys.DsnpPublicKey> getPublicKeys(long dsnpUserId) throws BaseGraphSdkException, InvalidProtocolBufferException {
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
