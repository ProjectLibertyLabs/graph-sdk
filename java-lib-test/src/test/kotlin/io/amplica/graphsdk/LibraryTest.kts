package io.amplica.graphsdk

import io.amplica.graphsdk.exceptions.GraphSdkException
import io.amplica.graphsdk.exceptions.InvalidHandleException
import io.amplica.graphsdk.models.*
import nl.altindag.log.LogCaptor
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.slf4j.event.Level
import java.util.regex.Pattern
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class LibraryTest {
    private val logCaptor = LogCaptor.forName("dsnp-graph-sdk")

    private fun testLogsForPattern(level: Level, patternStr: String): Boolean {
        val logs = when (level) {
            Level.DEBUG -> logCaptor.debugLogs
            Level.INFO -> logCaptor.infoLogs
            Level.WARN -> logCaptor.warnLogs
            Level.ERROR -> logCaptor.errorLogs
            else -> null
        }

        if (logs == null) {
            return false
        }

        val pattern = Pattern.compile(patternStr)
        return logs.any { pattern.matcher(it).find() }
    }

    @BeforeAll
    fun loggerInitShouldWork() {
        logCaptor.disableConsoleOutput()

        assertDoesNotThrow { Logger.initialize(Level.DEBUG) }
        assertTrue(testLogsForPattern(Level.INFO, "Initializing dsnp-graph-sdk-jni"))
    }

    @BeforeEach
    fun resetBeforeTest() {
        logCaptor.clearLogs()
        Native.loggerSetMaxLevel(Level.DEBUG.toInt())
    }

    @Test
    fun keepAliveShouldWork() {
        Native.keepAlive(this)
    }

    @Test
    fun initiateMainNetStateShouldWork() {
        // act
        val graph = Graph(Configuration.getMainNet())

        // assert
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard())
        graph.finalize()
    }

    @Test
    fun initiateRococoStateShouldWork() {
        // act
        val graph = Graph(Configuration.getRococo())

        // assert
        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard())
        graph.finalize()
    }

    @Test
    fun invalidHandleExceptionShouldBeThrown() {
        // act, assert
        assertFailsWith<InvalidHandleException> {
            Native.invalidHandleOperation()
        }
    }

    @Test
    fun graphSdkExceptionShouldBeThrown() {
        // act, assert
        assertFailsWith<GraphSdkException> {
            Native.graphSdkExceptionOperation()
        }
    }
}
