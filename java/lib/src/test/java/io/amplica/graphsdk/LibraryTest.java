package io.amplica.graphsdk;

import com.google.protobuf.ByteString;
import io.amplica.graphsdk.exceptions.GraphSdkException;
import io.amplica.graphsdk.exceptions.InvalidHandleException;
import io.amplica.graphsdk.models.*;
import nl.altindag.log.LogCaptor;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.slf4j.event.Level;

import java.util.List;
import java.util.regex.Pattern;

import static org.junit.jupiter.api.Assertions.*;

class LibraryTest {
    private static final LogCaptor logCaptor = LogCaptor.forName("dsnp-graph-sdk");

    private static boolean testLogsForPattern(Level level, String pattern_str) {
        List<String> logs = null;

        switch (level) {
            case DEBUG:
                logs = logCaptor.getDebugLogs();
                break;

            case INFO:
                logs = logCaptor.getInfoLogs();
                break;

            case WARN:
                logs = logCaptor.getWarnLogs();
                break;

            case ERROR:
                logs = logCaptor.getErrorLogs();

            default:
                break;
        }

        if (logs == null) {
            return false;
        }

        Pattern p = Pattern.compile(pattern_str);
        return logs.stream().anyMatch(str -> p.matcher(str).find());
    }

    @BeforeAll
    public static void logger_init_should_work() {
        assertDoesNotThrow(() -> Logger.initialize(Level.DEBUG));
        assertEquals(true, testLogsForPattern(Level.INFO, "Initializing dsnp-graph-sdk-jni"));
    }

    @BeforeEach
    public void resetBeforeTest() {
        logCaptor.clearLogs();
        Native.loggerSetMaxLevel(Level.DEBUG.toInt());
    }

    @Test
    void keep_alive_should_work() {
        Native.keepAlive(this);
    }

    @Test
    void initiate_main_net_state_should_work() throws Exception {
        // act
        var graph = new Graph(Configuration.getMainNet());

        // assert
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
        graph.finalize();
    }

    @Test
    void initiate_rococo_state_should_work() throws Exception {
        // act
        var graph = new Graph(Configuration.getRococo());

        // assert
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
        graph.finalize();
    }

    @Test
    void initiate_dev_state_should_work() throws Exception {
        // arrange
        var config = new Configuration(Environment.newBuilder().getConfigBuilder()
                .addDsnpVersions(DsnpVersion.Version1_0)
                .setMaxPageId(10)
                .build());

        // act
        var graph = new Graph(config);

        // assert
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
        graph.finalize();
    }

    @Test
    void invalid_handle_should_throw_InvalidHandleException() throws Exception {
        // arrange
        var graph = new Graph(Configuration.getMainNet());
        graph.setUnsafeHandle(0);

        // act
        InvalidHandleException exception = assertThrows(InvalidHandleException.class, () -> {
            graph.getUsersLength();
        });

        // assert
        String expectedMessage = "invalid handle";
        String actualMessage = exception.getMessage();
        assertTrue(actualMessage.contains(expectedMessage));
    }

    @Test
    void graph_get_user_length_should_work() throws Exception {
        // arrange
        var graph = new Graph(Configuration.getMainNet());

        // act
        var length = graph.getUsersLength();

        // assert
        assertEquals(0, length);
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
    }

    @Test
    void graph_containsUserGraph_should_work() throws Exception {
        // arrange
        var graph = new Graph(Configuration.getMainNet());

        // act
        var exists = graph.containsUserGraph(1);

        // assert
        assertFalse(exists);
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
    }

    @Test
    void graph_applyActions_addingConnection_should_work() throws Exception {
        // arrange
        var ownerUserId = 1;
        var schemaId = 1;
        var connectionUserId = 1000;
        var actions = Actions.newBuilder().addActions(
                Actions.Action.newBuilder().setConnectAction(
                        Actions.Action.ConnectAction.newBuilder()
                                .setOwnerDsnpUserId(ownerUserId)
                                .setConnection(
                                        Connection.newBuilder().setDsnpUserId(connectionUserId).setSchemaId(schemaId)
                                                .build())
                                .build()))
                .build();
        var graph = new Graph(Configuration.getMainNet());

        // act
        graph.applyActions(actions);
        var connections = graph.getConnections(ownerUserId, ConnectionType.FollowPublic, true);
        var updates = graph.exportUpdates();

        // assert
        assertEquals(1, connections.size());
        assertEquals(connectionUserId, connections.get(0).getUserId());
        assertTrue(connections.get(0).getSince() > 0);

        assertEquals(1, updates.size());
        assertTrue(updates.get(0).hasPersist());
        assertEquals(updates.get(0).getPersist().getPageId(), 0);
        assertEquals(updates.get(0).getPersist().getSchemaId(), schemaId);
        assertEquals(updates.get(0).getPersist().getOwnerDsnpUserId(), ownerUserId);
        assertEquals(updates.get(0).getPersist().getPrevHash(), 0);

        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
    }

    @Test
    void graph_applyActions_addingConnection_with_invalid_request_should_throw_exception() throws Exception {
        // arrange
        var ownerUserId = 1;
        var schemaId = 1;
        var connectionUserId = 1000;
        var invalid_actions = Actions.newBuilder().addActions(
                Actions.Action.newBuilder().setDisconnectAction(
                        Actions.Action.DisconnectAction.newBuilder()
                                .setOwnerDsnpUserId(ownerUserId)
                                .setConnection(
                                        Connection.newBuilder().setDsnpUserId(connectionUserId)
                                                .setSchemaId(schemaId)
                                                .build())
                                .build()))
                .build();
        var graph = new Graph(Configuration.getMainNet());

        // act
        GraphSdkException exception = assertThrows(GraphSdkException.class, () -> {
            graph.applyActions(invalid_actions);
        });

        // assert
        String expectedMessage = "does not exist";
        String actualMessage = exception.getMessage();
        assertTrue(actualMessage.contains(expectedMessage));
    }

    @Test
    void graph_applyActions_addingConnection_with_incomplete_request_should_throw_exception() throws Exception {
        // arrange
        var schemaId = 1;
        var connectionUserId = 1000;
        // no dsnp user is set
        var invalid_actions = Actions.newBuilder().addActions(
                Actions.Action.newBuilder().setConnectAction(
                        Actions.Action.ConnectAction.newBuilder()
                                .setConnection(
                                        Connection.newBuilder().setDsnpUserId(connectionUserId)
                                                .setSchemaId(schemaId)
                                                .build())
                                .build()))
                .build();
        var graph = new Graph(Configuration.getMainNet());

        // act
        GraphSdkException exception = assertThrows(GraphSdkException.class, () -> {
            graph.applyActions(invalid_actions);
        });

        // assert
        String expectedMessage = "Invalid user id";
        String actualMessage = exception.getMessage();
        assertTrue(actualMessage.contains(expectedMessage));
    }

    @Test
    void graph_applyActions_addingKey_should_work() throws Exception {
        // arrange
        var ownerUserId = 1;
        var publicKey = "0fea2cafabdc83752be36fa5349640da2c828add0a290df13cd2d8173eb2496f";
        var actions = Actions.newBuilder().addActions(
                Actions.Action.newBuilder().setAddKeyAction(
                        Actions.Action.AddGraphKey.newBuilder()
                                .setOwnerDsnpUserId(ownerUserId)
                                .setNewPublicKey(ByteString.fromHex(publicKey))
                                .build()))
                .build();
        var graph = new Graph(Configuration.getMainNet());

        // act
        graph.applyActions(actions);
        var updates = graph.exportUpdates();

        // assert
        assertEquals(1, updates.size());
        assertTrue(updates.get(0).hasAddKey());
        assertEquals(updates.get(0).getAddKey().getPrevHash(), 0);
        assertEquals(updates.get(0).getAddKey().getOwnerDsnpUserId(), ownerUserId);
        assertTrue(updates.get(0).getAddKey().getPayload().size() > 0);
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
    }

    @Test
    void graph_removeUserGraph_should_work() throws Exception {
        // arrange
        var ownerUserId = 1;
        var schemaId = 1;
        var connectionUserId = 1000;
        var actions = Actions.newBuilder().addActions(
                Actions.Action.newBuilder().setConnectAction(
                        Actions.Action.ConnectAction.newBuilder()
                                .setOwnerDsnpUserId(ownerUserId)
                                .setConnection(
                                        Connection.newBuilder().setDsnpUserId(connectionUserId).setSchemaId(schemaId)
                                                .build())
                                .build()))
                .build();
        var graph = new Graph(Configuration.getMainNet());
        graph.applyActions(actions);

        // act
        graph.removeUserGraph(ownerUserId);

        // assert
        var exists = graph.containsUserGraph(ownerUserId);
        assertFalse(exists);
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
    }

    @Test
    void graph_importUserData_should_work() throws Exception {
        // arrange
        var ownerUserId = 20;
        var pageId = 5;
        var schemaId = 1;
        var contentHash = 1000;
        var publicKey = ByteString.fromHex("0fea2cafabdc83752be36fa5349640da2c828add0a290df13cd2d8173eb2496f");
        var bundles = ImportBundles.newBuilder()
                .addBundles(
                        ImportBundles.ImportBundle.newBuilder()
                                .setDsnpUserId(ownerUserId)
                                .setSchemaId(schemaId)
                                .setDsnpKeys(DsnpKeys.newBuilder()
                                        .setDsnpUserId(ownerUserId)
                                        .setKeysHash(1)
                                        .addKeys(
                                                KeyData.newBuilder().setIndex(0)
                                                        .setContent(ByteString.copyFrom(new byte[] { 64, 15, -22, 44,
                                                                -81, -85, -36, -125, 117, 43, -29, 111, -91, 52, -106,
                                                                64, -38, 44, -126, -118, -35, 10, 41, 13, -15, 60, -46,
                                                                -40, 23, 62, -78, 73, 111 }))
                                                        .build())
                                        .build())
                                .addPages(
                                        PageData.newBuilder()
                                                .setPageId(pageId)
                                                .setContentHash(contentHash)
                                                .setContent(ByteString.copyFrom(
                                                        new byte[] { 20, 99, -70, -64, -33, 118, -13, 44, 35, 3, 0 }))
                                                .build())
                                .build())
                .build();
        var graph = new Graph(Configuration.getMainNet());

        // act
        graph.importUserData(bundles);
        var keys = graph.getPublicKeys(ownerUserId);

        // assert
        var exists = graph.containsUserGraph(ownerUserId);
        assertTrue(exists);

        var connections = graph.getConnections(ownerUserId, ConnectionType.FollowPublic, false);
        assertTrue(connections.size() > 0);

        assertEquals(1, keys.size());
        assertEquals(publicKey, keys.get(0).getKey());
        assertEquals(0, keys.get(0).getKeyId());
    }

    @Test
    void graph_usersWithoutImportedKeys_should_work() throws Exception {
        // arrange
        var graph = new Graph(Configuration.getMainNet());

        // act
        var users = graph.getUsersWithoutImportedKeys();

        // assert
        assertEquals(0, users.size());
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
    }

    @Test
    void graph_forceCalculateGraph_should_work() throws Exception {
        // arrange
        var ownerUserId = 20;
        var pageId = 5;
        var schemaId = 1;
        var contentHash = 1000;
        var bundles = ImportBundles.newBuilder()
                .addBundles(
                        ImportBundles.ImportBundle.newBuilder()
                                .setDsnpUserId(ownerUserId)
                                .setSchemaId(schemaId)
                                .setDsnpKeys(DsnpKeys.newBuilder()
                                        .setDsnpUserId(ownerUserId)
                                        .setKeysHash(0)
                                        .build())
                                .addPages(
                                        PageData.newBuilder()
                                                .setPageId(pageId)
                                                .setContentHash(contentHash)
                                                .setContent(ByteString.copyFrom(
                                                        new byte[] { 20, 99, -70, -64, -33, 118, -13, 44, 35, 3, 0 }))
                                                .build())
                                .build())
                .build();
        var graph = new Graph(Configuration.getMainNet());
        graph.importUserData(bundles);

        // act
        var updates = graph.forceRecalculateGraph(ownerUserId);

        // assert
        var exists = graph.containsUserGraph(ownerUserId);
        assertTrue(exists);

        assertEquals(1, updates.size());
        assertTrue(updates.get(0).hasPersist());
        assertEquals(updates.get(0).getPersist().getPageId(), pageId);
        assertEquals(updates.get(0).getPersist().getSchemaId(), schemaId);
        assertEquals(updates.get(0).getPersist().getOwnerDsnpUserId(), ownerUserId);
        assertEquals(updates.get(0).getPersist().getPrevHash(), contentHash);
    }

    @Test
    void Graph_deserializeDsnpKeys_should_work() throws Exception {
        // arrange
        var ownerUserId = 20;
        var publicKey = ByteString.fromHex("0fea2cafabdc83752be36fa5349640da2c828add0a290df13cd2d8173eb2496f");
        var index = 4;
        var dsnpKeys = DsnpKeys.newBuilder()
                .setDsnpUserId(ownerUserId)
                .setKeysHash(1)
                .addKeys(KeyData.newBuilder().setIndex(index)
                        .setContent(ByteString.copyFrom(new byte[] { 64, 15, -22, 44, -81, -85, -36, -125, 117, 43, -29,
                                111, -91, 52, -106, 64, -38, 44, -126, -118, -35, 10, 41, 13, -15, 60, -46, -40, 23, 62,
                                -78, 73, 111 }))
                        .build())
                .build();

        // act
        var keys = Graph.deserializeDsnpKeys(dsnpKeys);

        // assert
        assertEquals(1, keys.size());
        assertEquals(publicKey, keys.get(0).getKey());
        assertEquals(index, keys.get(0).getKeyId());
    }

    @Test
    void Graph_generateKeyPair_should_work() throws Exception {
        // arrange
        var keyPair = Graph.generateKeyPair(GraphKeyType.X25519);
        assertNotNull(keyPair);
    }

    @Test
    void logger_double_initialize_should_fail() {
        Logger.initialize();
        assertEquals(true, testLogsForPattern(Level.WARN, "Duplicate logger initialization ignored"));
    }

    @Test
    void logger_debug_should_log() {
        log(Level.DEBUG.toInt(), "This is a debug log");
        log(Level.INFO.toInt(), "This is an info log");
        log(Level.WARN.toInt(), "This is a warning log");
        log(Level.ERROR.toInt(), "This is an error log");
        assertEquals(true, testLogsForPattern(Level.DEBUG, "This is a debug log"));
        assertEquals(true, testLogsForPattern(Level.INFO, "This is an info log"));
        assertEquals(true, testLogsForPattern(Level.WARN, "This is a warning log"));
        assertEquals(true, testLogsForPattern(Level.ERROR, "This is an error log"));
    }

    @Test
    void logger_info_should_log() {
        Native.loggerSetMaxLevel(Level.INFO.toInt());
        log(Level.DEBUG.toInt(), "This is a debug log");
        log(Level.INFO.toInt(), "This is an info log");
        log(Level.WARN.toInt(), "This is a warning log");
        log(Level.ERROR.toInt(), "This is an error log");
        assertEquals(false, testLogsForPattern(Level.DEBUG, "This is a debug log"));
        assertEquals(true, testLogsForPattern(Level.INFO, "This is an info log"));
        assertEquals(true, testLogsForPattern(Level.WARN, "This is a warning log"));
        assertEquals(true, testLogsForPattern(Level.ERROR, "This is an error log"));
    }

    @Test
    void logger_warn_should_log() {
        Native.loggerSetMaxLevel(Level.WARN.toInt());
        log(Level.DEBUG.toInt(), "This is a debug log");
        log(Level.INFO.toInt(), "This is an info log");
        log(Level.WARN.toInt(), "This is a warning log");
        log(Level.ERROR.toInt(), "This is an error log");
        assertEquals(false, testLogsForPattern(Level.DEBUG, "This is a debug log"));
        assertEquals(false, testLogsForPattern(Level.INFO, "This is an info log"));
        assertEquals(true, testLogsForPattern(Level.WARN, "This is a warning log"));
        assertEquals(true, testLogsForPattern(Level.ERROR, "This is an error log"));
    }

    @Test
    void logger_error_should_log() {
        Native.loggerSetMaxLevel(Level.ERROR.toInt());
        log(Level.DEBUG.toInt(), "This is a debug log");
        log(Level.INFO.toInt(), "This is an info log");
        log(Level.WARN.toInt(), "This is a warning log");
        log(Level.ERROR.toInt(), "This is an error log");
        assertEquals(false, testLogsForPattern(Level.DEBUG, "This is a debug log"));
        assertEquals(false, testLogsForPattern(Level.INFO, "This is an info log"));
        assertEquals(false, testLogsForPattern(Level.WARN, "This is a warning log"));
        assertEquals(true, testLogsForPattern(Level.ERROR, "This is an error log"));
    }

    private static native void log(int level, String message);
}
