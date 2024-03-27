// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: bridge/common/protos/output.proto

package io.amplica.graphsdk.models;

public final class Output {
  private Output() {}
  public static void registerAllExtensions(
      com.google.protobuf.ExtensionRegistryLite registry) {
  }

  public static void registerAllExtensions(
      com.google.protobuf.ExtensionRegistry registry) {
    registerAllExtensions(
        (com.google.protobuf.ExtensionRegistryLite) registry);
  }
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_SchemaConfig_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_SchemaConfig_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Config_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Config_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Config_SchemaMapEntry_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Config_SchemaMapEntry_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Environment_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Environment_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Updates_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Updates_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Updates_Update_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Updates_Update_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Updates_Update_PersistPageUpdate_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Updates_Update_PersistPageUpdate_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Updates_Update_DeletePageUpdate_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Updates_Update_DeletePageUpdate_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Updates_Update_AddKeyUpdate_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Updates_Update_AddKeyUpdate_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_DsnpGraphEdges_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_DsnpGraphEdges_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_DsnpGraphEdges_DsnpGraphEdge_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_DsnpGraphEdges_DsnpGraphEdge_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_DsnpUsers_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_DsnpUsers_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_DsnpPublicKeys_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_DsnpPublicKeys_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_DsnpPublicKeys_DsnpPublicKey_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_DsnpPublicKeys_DsnpPublicKey_fieldAccessorTable;

  public static com.google.protobuf.Descriptors.FileDescriptor
      getDescriptor() {
    return descriptor;
  }
  private static  com.google.protobuf.Descriptors.FileDescriptor
      descriptor;
  static {
    java.lang.String[] descriptorData = {
      "\n!bridge/common/protos/output.proto\"\\\n\014S" +
      "chemaConfig\022\"\n\014dsnp_version\030\001 \001(\0162\014.Dsnp" +
      "Version\022(\n\017connection_type\030\002 \001(\0162\017.Conne" +
      "ctionType\"\276\002\n\006Config\022%\n\035sdk_max_stale_fr" +
      "iendship_days\030\001 \001(\r\022!\n\031max_graph_page_si" +
      "ze_bytes\030\002 \001(\r\022\023\n\013max_page_id\030\003 \001(\r\022\037\n\027m" +
      "ax_key_page_size_bytes\030\004 \001(\r\022*\n\nschema_m" +
      "ap\030\005 \003(\0132\026.Config.SchemaMapEntry\022#\n\rdsnp" +
      "_versions\030\006 \003(\0162\014.DsnpVersion\022\"\n\032graph_p" +
      "ublic_key_schema_id\030\007 \001(\r\032?\n\016SchemaMapEn" +
      "try\022\013\n\003key\030\001 \001(\r\022\034\n\005value\030\002 \001(\0132\r.Schema" +
      "Config:\0028\001\"b\n\013Environment\022*\n\020environment" +
      "_type\030\001 \001(\0162\020.EnvironmentType\022\034\n\006config\030" +
      "\002 \001(\0132\007.ConfigH\000\210\001\001B\t\n\007_config\"\211\004\n\007Updat" +
      "es\022\037\n\006update\030\001 \003(\0132\017.Updates.Update\032\334\003\n\006" +
      "Update\0224\n\007persist\030\001 \001(\0132!.Updates.Update" +
      ".PersistPageUpdateH\000\0222\n\006delete\030\002 \001(\0132 .U" +
      "pdates.Update.DeletePageUpdateH\000\022/\n\007add_" +
      "key\030\003 \001(\0132\034.Updates.Update.AddKeyUpdateH" +
      "\000\032w\n\021PersistPageUpdate\022\032\n\022owner_dsnp_use" +
      "r_id\030\001 \001(\004\022\021\n\tschema_id\030\002 \001(\r\022\017\n\007page_id" +
      "\030\003 \001(\r\022\021\n\tprev_hash\030\004 \001(\r\022\017\n\007payload\030\005 \001" +
      "(\014\032e\n\020DeletePageUpdate\022\032\n\022owner_dsnp_use" +
      "r_id\030\001 \001(\004\022\021\n\tschema_id\030\002 \001(\r\022\017\n\007page_id" +
      "\030\003 \001(\r\022\021\n\tprev_hash\030\004 \001(\r\032N\n\014AddKeyUpdat" +
      "e\022\032\n\022owner_dsnp_user_id\030\001 \001(\004\022\021\n\tprev_ha" +
      "sh\030\002 \001(\r\022\017\n\007payload\030\003 \001(\014B\007\n\005inner\"n\n\016Ds" +
      "npGraphEdges\022+\n\004edge\030\001 \003(\0132\035.DsnpGraphEd" +
      "ges.DsnpGraphEdge\032/\n\rDsnpGraphEdge\022\017\n\007us" +
      "er_id\030\001 \001(\004\022\r\n\005since\030\002 \001(\004\"\031\n\tDsnpUsers\022" +
      "\014\n\004user\030\001 \003(\004\"q\n\016DsnpPublicKeys\0221\n\npubli" +
      "c_key\030\001 \003(\0132\035.DsnpPublicKeys.DsnpPublicK" +
      "ey\032,\n\rDsnpPublicKey\022\013\n\003key\030\001 \001(\014\022\016\n\006key_" +
      "id\030\002 \001(\004*&\n\013PrivacyType\022\n\n\006Public\020\000\022\013\n\007P" +
      "rivate\020\001*b\n\016ConnectionType\022\020\n\014FollowPubl" +
      "ic\020\000\022\021\n\rFollowPrivate\020\001\022\024\n\020FriendshipPub" +
      "lic\020\002\022\025\n\021FriendshipPrivate\020\003*\035\n\013DsnpVers" +
      "ion\022\016\n\nVersion1_0\020\000*E\n\017EnvironmentType\022\013" +
      "\n\007MainNet\020\000\022\020\n\014TestnetPaseo\020\001\022\n\n\006Rococo\020" +
      "\002\022\007\n\003Dev\020\003B\036\n\032io.amplica.graphsdk.models" +
      "P\001b\006proto3"
    };
    descriptor = com.google.protobuf.Descriptors.FileDescriptor
      .internalBuildGeneratedFileFrom(descriptorData,
        new com.google.protobuf.Descriptors.FileDescriptor[] {
        });
    internal_static_SchemaConfig_descriptor =
      getDescriptor().getMessageTypes().get(0);
    internal_static_SchemaConfig_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_SchemaConfig_descriptor,
        new java.lang.String[] { "DsnpVersion", "ConnectionType", });
    internal_static_Config_descriptor =
      getDescriptor().getMessageTypes().get(1);
    internal_static_Config_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Config_descriptor,
        new java.lang.String[] { "SdkMaxStaleFriendshipDays", "MaxGraphPageSizeBytes", "MaxPageId", "MaxKeyPageSizeBytes", "SchemaMap", "DsnpVersions", "GraphPublicKeySchemaId", });
    internal_static_Config_SchemaMapEntry_descriptor =
      internal_static_Config_descriptor.getNestedTypes().get(0);
    internal_static_Config_SchemaMapEntry_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Config_SchemaMapEntry_descriptor,
        new java.lang.String[] { "Key", "Value", });
    internal_static_Environment_descriptor =
      getDescriptor().getMessageTypes().get(2);
    internal_static_Environment_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Environment_descriptor,
        new java.lang.String[] { "EnvironmentType", "Config", "Config", });
    internal_static_Updates_descriptor =
      getDescriptor().getMessageTypes().get(3);
    internal_static_Updates_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Updates_descriptor,
        new java.lang.String[] { "Update", });
    internal_static_Updates_Update_descriptor =
      internal_static_Updates_descriptor.getNestedTypes().get(0);
    internal_static_Updates_Update_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Updates_Update_descriptor,
        new java.lang.String[] { "Persist", "Delete", "AddKey", "Inner", });
    internal_static_Updates_Update_PersistPageUpdate_descriptor =
      internal_static_Updates_Update_descriptor.getNestedTypes().get(0);
    internal_static_Updates_Update_PersistPageUpdate_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Updates_Update_PersistPageUpdate_descriptor,
        new java.lang.String[] { "OwnerDsnpUserId", "SchemaId", "PageId", "PrevHash", "Payload", });
    internal_static_Updates_Update_DeletePageUpdate_descriptor =
      internal_static_Updates_Update_descriptor.getNestedTypes().get(1);
    internal_static_Updates_Update_DeletePageUpdate_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Updates_Update_DeletePageUpdate_descriptor,
        new java.lang.String[] { "OwnerDsnpUserId", "SchemaId", "PageId", "PrevHash", });
    internal_static_Updates_Update_AddKeyUpdate_descriptor =
      internal_static_Updates_Update_descriptor.getNestedTypes().get(2);
    internal_static_Updates_Update_AddKeyUpdate_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Updates_Update_AddKeyUpdate_descriptor,
        new java.lang.String[] { "OwnerDsnpUserId", "PrevHash", "Payload", });
    internal_static_DsnpGraphEdges_descriptor =
      getDescriptor().getMessageTypes().get(4);
    internal_static_DsnpGraphEdges_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_DsnpGraphEdges_descriptor,
        new java.lang.String[] { "Edge", });
    internal_static_DsnpGraphEdges_DsnpGraphEdge_descriptor =
      internal_static_DsnpGraphEdges_descriptor.getNestedTypes().get(0);
    internal_static_DsnpGraphEdges_DsnpGraphEdge_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_DsnpGraphEdges_DsnpGraphEdge_descriptor,
        new java.lang.String[] { "UserId", "Since", });
    internal_static_DsnpUsers_descriptor =
      getDescriptor().getMessageTypes().get(5);
    internal_static_DsnpUsers_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_DsnpUsers_descriptor,
        new java.lang.String[] { "User", });
    internal_static_DsnpPublicKeys_descriptor =
      getDescriptor().getMessageTypes().get(6);
    internal_static_DsnpPublicKeys_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_DsnpPublicKeys_descriptor,
        new java.lang.String[] { "PublicKey", });
    internal_static_DsnpPublicKeys_DsnpPublicKey_descriptor =
      internal_static_DsnpPublicKeys_descriptor.getNestedTypes().get(0);
    internal_static_DsnpPublicKeys_DsnpPublicKey_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_DsnpPublicKeys_DsnpPublicKey_descriptor,
        new java.lang.String[] { "Key", "KeyId", });
  }

  // @@protoc_insertion_point(outer_class_scope)
}
