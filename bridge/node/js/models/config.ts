enum DsnpVersion {
    Version1_0 = "1.0",
  }
  
  enum ConnectionType {
    Follow = "follow",
    Friendship = "friendship",
  }
  
  enum PrivacyType {
    Public = "public",
    Private = "private",
  }
  
  interface SchemaConfig {
    dsnpVersion: DsnpVersion;
    connectionType: ConnectionType,
    privacyType: PrivacyType,
  }
  
  interface Config {
    sdkMaxUsersGraphSize: number;
    sdkMaxStaleFriendshipDays: number;
    maxGraphPageSizeBytes: number;
    maxPageId: number;
    maxKeyPageSizeBytes: number;
    schemaMap: { [key: string]: SchemaConfig };
    dsnpVersions: DsnpVersion[];
  }
  
  export { Config, ConnectionType, DsnpVersion, SchemaConfig };
  