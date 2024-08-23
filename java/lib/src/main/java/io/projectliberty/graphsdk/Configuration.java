package io.projectliberty.graphsdk;

import com.google.protobuf.InvalidProtocolBufferException;
import io.projectliberty.graphsdk.models.Config;
import io.projectliberty.graphsdk.models.ConnectionType;
import io.projectliberty.graphsdk.models.Environment;
import io.projectliberty.graphsdk.models.EnvironmentType;

import java.util.HashMap;

public class Configuration {
    private static Configuration MAIN_NET_INSTANCE;
    private static Configuration ROCOCO_INSTANCE;
    private static Configuration TESTNET_PASEO_INSTANCE;

    private final Config inner;
    private final Environment environment;
    private final HashMap<ConnectionType, Integer> schemaIdMap = new HashMap<>();

    public Configuration(Config config) {
        this.inner = config;
        this.environment = Environment.newBuilder().setEnvironmentType(EnvironmentType.Dev).
                setConfig(this.inner).build();
        fillSchemaIdMap();
    }

    private Configuration(EnvironmentType environmentType) throws InvalidProtocolBufferException {
        this.environment = Environment.newBuilder().setEnvironmentType(environmentType).build();
        var rawConfig = Native.getConfig(this.environment.toByteArray());
        this.inner = Config.parseFrom(rawConfig);
        fillSchemaIdMap();
    }

    private void fillSchemaIdMap() {
        for (var entry : this.inner.getSchemaMapMap().entrySet()) {
            this.schemaIdMap.put(entry.getValue().getConnectionType(), entry.getKey());
        }
    }

    public int getSchemaId(ConnectionType connectionType) {
        return this.schemaIdMap.get(connectionType);
    }

    public int getGraphPublicKeySchemaId() {
        return this.inner.getGraphPublicKeySchemaId();
    }

    public Environment getEnvironment() {
        return this.environment;
    }

    public static Configuration getMainNet() throws InvalidProtocolBufferException {
        if(MAIN_NET_INSTANCE == null) {
            MAIN_NET_INSTANCE = new Configuration(EnvironmentType.MainNet);
        }
        return MAIN_NET_INSTANCE;
    }

    public static Configuration getTestnetPaseo() throws InvalidProtocolBufferException {
        if(TESTNET_PASEO_INSTANCE == null) {
            TESTNET_PASEO_INSTANCE = new Configuration(EnvironmentType.TestnetPaseo);
        }
        return TESTNET_PASEO_INSTANCE;
    }

    public static Configuration getRococo() throws InvalidProtocolBufferException {
        if(ROCOCO_INSTANCE == null) {
            ROCOCO_INSTANCE = new Configuration(EnvironmentType.Rococo);
        }
        return ROCOCO_INSTANCE;
    }
}
