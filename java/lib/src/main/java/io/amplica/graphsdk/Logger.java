package io.amplica.graphsdk;

import org.slf4j.LoggerFactory;
import org.slf4j.event.Level;
import io.amplica.graphsdk.Native;

public class Logger {
    public static void initialize(Level level) {
        org.slf4j.Logger logger = LoggerFactory.getLogger("dsnp-graph-sdk");
        Native.loggerInitialize(level.toInt(), logger);
    }

    public static void initialize() {
        initialize(Level.INFO);
    }
}
