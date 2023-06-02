package io.amplica.graphsdk
import io.amplica.graphsdk.exceptions.GraphSdkException
import io.amplica.graphsdk.exceptions.InvalidHandleException
import io.amplica.graphsdk.Configuration
import io.amplica.graphsdk.Graph
import io.amplica.graphsdk.Logger
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
