package amplica.graphsdk;

import io.amplica.graphsdk.Graph;
import io.amplica.graphsdk.Native;
import io.amplica.graphsdk.models.DsnpVersion;
import io.amplica.graphsdk.models.Environment;
import io.amplica.graphsdk.models.EnvironmentType;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class LibraryTest {
    @Test void hello_should_work() {
        String result = Native.hello("Java");
        assertEquals(result, "Hello, Java!");
    }

    @Test void keep_alive_should_work() {
        Native.keepAlive(this);
    }

    @Test void initiate_main_net_state_should_work() {
        var env = Environment.newBuilder().setEnvironmentType(EnvironmentType.MainNet).build();
        var graph = new Graph(env);

        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
        graph.finalize();
    }

    @Test void initiate_rococo_state_should_work() {
        var env = Environment.newBuilder().setEnvironmentType(EnvironmentType.Rococo).build();
        var graph = new Graph(env);

        assertNotEquals(0, graph.unsafeNativeHandleWithoutGuard());
        graph.finalize();
    }

    @Test void initiate_dev_state_should_work() {
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
}
