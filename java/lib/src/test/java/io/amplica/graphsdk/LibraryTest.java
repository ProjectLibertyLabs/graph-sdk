package io.amplica.graphsdk;

import io.amplica.graphsdk.Graph;
import io.amplica.graphsdk.Logger;
import io.amplica.graphsdk.Native;
import io.amplica.graphsdk.models.DsnpVersion;
import io.amplica.graphsdk.models.Environment;
import io.amplica.graphsdk.models.EnvironmentType;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.slf4j.event.Level;
import nl.altindag.log.LogCaptor;

import java.util.List;
import java.util.regex.Pattern;

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
        logCaptor.disableConsoleOutput();

        assertDoesNotThrow(() -> Logger.initialize(Level.DEBUG));
        assertEquals(true, testLogsForPattern(Level.INFO, "Initializing dsnp-graph-sdk-jni"));
    }

    @BeforeEach
    public void resetBeforeTest() {
        logCaptor.clearLogs();
        Native.loggerSetMaxLevel(Level.DEBUG.toInt());
    }

    @Test
    void hello_should_work() {
        String result = Native.hello("Java");
        assertEquals(result, "Hello, Java!");
    }

    @Test
    void keep_alive_should_work() {
        Native.keepAlive(this);
    }

    @Test
    void initiate_main_net_state_should_work() {
        var env = Environment.newBuilder().setEnvironmentType(EnvironmentType.MainNet).build();
        var graph = new Graph(env);

        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
        graph.finalize();
    }

    @Test
    void initiate_rococo_state_should_work() {
        var env = Environment.newBuilder().setEnvironmentType(EnvironmentType.Rococo).build();
        var graph = new Graph(env);

        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
        graph.finalize();
    }

    @Test
    void initiate_dev_state_should_work() {
        var config = Environment.newBuilder().getConfigBuilder()
                .addDsnpVersions(DsnpVersion.Version1_0)
                .setMaxPageId(10)
                .build();
        var env = Environment.newBuilder().setEnvironmentType(EnvironmentType.Dev)
                .setConfig(config)
                .build();
        var graph = new Graph(env);

        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
        graph.finalize();
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
