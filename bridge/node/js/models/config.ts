enum DsnpVersion {
    Version1_0 = "1.0",
  }
  
  enum ConnectionType {
    Follow = "follow",
    Friendship = "friendship",
  }
  
  interface SchemaConfig {
    dsnp_version: DsnpVersion;
    connection_type: ConnectionType;
  }
  
  interface Config {
    sdk_max_users_graph_size: number;
    sdk_max_stale_friendship_days: number;
    max_graph_page_size_bytes: number;
    max_page_id: number;
    max_key_page_size_bytes: number;
    schema_map: { [key: string]: SchemaConfig };
    dsnp_versions: DsnpVersion[];
  }
  
  export { Config, ConnectionType, DsnpVersion, SchemaConfig };
  