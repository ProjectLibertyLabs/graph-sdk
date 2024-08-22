package io.projectliberty.graphsdk
import io.projectliberty.graphsdk.exceptions.GraphSdkException
import io.projectliberty.graphsdk.exceptions.InvalidHandleException
import io.projectliberty.graphsdk.Configuration
import io.projectliberty.graphsdk.Graph
import io.projectliberty.graphsdk.Logger
import org.junit.jupiter.api.Assertions.assertDoesNotThrow
import org.junit.jupiter.api.Assertions.assertNotEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import org.slf4j.event.Level
import java.util.regex.Pattern

class LibraryTest {

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
}
