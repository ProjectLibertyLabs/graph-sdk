// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: bridge/common/protos/input.proto

package io.projectliberty.graphsdk.models;

public final class Input {
  private Input() {}
  public static void registerAllExtensions(
      com.google.protobuf.ExtensionRegistryLite registry) {
  }

  public static void registerAllExtensions(
      com.google.protobuf.ExtensionRegistry registry) {
    registerAllExtensions(
        (com.google.protobuf.ExtensionRegistryLite) registry);
  }
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_KeyData_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_KeyData_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_PageData_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_PageData_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_DsnpKeys_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_DsnpKeys_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_ImportBundles_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_ImportBundles_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_ImportBundles_ImportBundle_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_ImportBundles_ImportBundle_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_ImportBundles_ImportBundle_GraphKeyPair_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_ImportBundles_ImportBundle_GraphKeyPair_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Connection_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Connection_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Actions_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Actions_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Actions_ActionOptions_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Actions_ActionOptions_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Actions_Action_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Actions_Action_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Actions_Action_ConnectAction_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Actions_Action_ConnectAction_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Actions_Action_DisconnectAction_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Actions_Action_DisconnectAction_fieldAccessorTable;
  static final com.google.protobuf.Descriptors.Descriptor
    internal_static_Actions_Action_AddGraphKey_descriptor;
  static final 
    com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internal_static_Actions_Action_AddGraphKey_fieldAccessorTable;

  public static com.google.protobuf.Descriptors.FileDescriptor
      getDescriptor() {
    return descriptor;
  }
  private static  com.google.protobuf.Descriptors.FileDescriptor
      descriptor;
  static {
    java.lang.String[] descriptorData = {
      "\n bridge/common/protos/input.proto\")\n\007Ke" +
      "yData\022\r\n\005index\030\001 \001(\r\022\017\n\007content\030\002 \001(\014\"B\n" +
      "\010PageData\022\017\n\007page_id\030\001 \001(\r\022\017\n\007content\030\002 " +
      "\001(\014\022\024\n\014content_hash\030\003 \001(\r\"K\n\010DsnpKeys\022\024\n" +
      "\014dsnp_user_id\030\001 \001(\004\022\021\n\tkeys_hash\030\002 \001(\r\022\026" +
      "\n\004keys\030\003 \003(\0132\010.KeyData\"\330\002\n\rImportBundles" +
      "\022,\n\007bundles\030\001 \003(\0132\033.ImportBundles.Import" +
      "Bundle\032\230\002\n\014ImportBundle\022\024\n\014dsnp_user_id\030" +
      "\001 \001(\004\022\021\n\tschema_id\030\002 \001(\r\022;\n\tkey_pairs\030\003 " +
      "\003(\0132(.ImportBundles.ImportBundle.GraphKe" +
      "yPair\022!\n\tdsnp_keys\030\004 \001(\0132\t.DsnpKeysH\000\210\001\001" +
      "\022\030\n\005pages\030\005 \003(\0132\t.PageData\032W\n\014GraphKeyPa" +
      "ir\022\037\n\010key_type\030\001 \001(\0162\r.GraphKeyType\022\022\n\np" +
      "ublic_key\030\002 \001(\014\022\022\n\nsecret_key\030\003 \001(\014B\014\n\n_" +
      "dsnp_keys\"5\n\nConnection\022\024\n\014dsnp_user_id\030" +
      "\001 \001(\004\022\021\n\tschema_id\030\002 \001(\r\"\262\005\n\007Actions\022 \n\007" +
      "actions\030\001 \003(\0132\017.Actions.Action\022,\n\007option" +
      "s\030\002 \001(\0132\026.Actions.ActionOptionsH\000\210\001\001\032u\n\r" +
      "ActionOptions\022#\n\033ignore_existing_connect" +
      "ions\030\001 \001(\010\022\"\n\032ignore_missing_connections" +
      "\030\002 \001(\010\022\033\n\023disable_auto_commit\030\003 \001(\010\032\323\003\n\006" +
      "Action\0227\n\016connect_action\030\001 \001(\0132\035.Actions" +
      ".Action.ConnectActionH\000\022=\n\021disconnect_ac" +
      "tion\030\002 \001(\0132 .Actions.Action.DisconnectAc" +
      "tionH\000\0225\n\016add_key_action\030\003 \001(\0132\033.Actions" +
      ".Action.AddGraphKeyH\000\032}\n\rConnectAction\022\032" +
      "\n\022owner_dsnp_user_id\030\001 \001(\004\022\037\n\nconnection" +
      "\030\002 \001(\0132\013.Connection\022!\n\tdsnp_keys\030\003 \001(\0132\t" +
      ".DsnpKeysH\000\210\001\001B\014\n\n_dsnp_keys\032O\n\020Disconne" +
      "ctAction\022\032\n\022owner_dsnp_user_id\030\001 \001(\004\022\037\n\n" +
      "connection\030\002 \001(\0132\013.Connection\032A\n\013AddGrap" +
      "hKey\022\032\n\022owner_dsnp_user_id\030\001 \001(\004\022\026\n\016new_" +
      "public_key\030\002 \001(\014B\007\n\005innerB\n\n\010_options*\032\n" +
      "\014GraphKeyType\022\n\n\006X25519\020\000B%\n!io.projectl" +
      "iberty.graphsdk.modelsP\001b\006proto3"
    };
    descriptor = com.google.protobuf.Descriptors.FileDescriptor
      .internalBuildGeneratedFileFrom(descriptorData,
        new com.google.protobuf.Descriptors.FileDescriptor[] {
        });
    internal_static_KeyData_descriptor =
      getDescriptor().getMessageTypes().get(0);
    internal_static_KeyData_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_KeyData_descriptor,
        new java.lang.String[] { "Index", "Content", });
    internal_static_PageData_descriptor =
      getDescriptor().getMessageTypes().get(1);
    internal_static_PageData_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_PageData_descriptor,
        new java.lang.String[] { "PageId", "Content", "ContentHash", });
    internal_static_DsnpKeys_descriptor =
      getDescriptor().getMessageTypes().get(2);
    internal_static_DsnpKeys_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_DsnpKeys_descriptor,
        new java.lang.String[] { "DsnpUserId", "KeysHash", "Keys", });
    internal_static_ImportBundles_descriptor =
      getDescriptor().getMessageTypes().get(3);
    internal_static_ImportBundles_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_ImportBundles_descriptor,
        new java.lang.String[] { "Bundles", });
    internal_static_ImportBundles_ImportBundle_descriptor =
      internal_static_ImportBundles_descriptor.getNestedTypes().get(0);
    internal_static_ImportBundles_ImportBundle_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_ImportBundles_ImportBundle_descriptor,
        new java.lang.String[] { "DsnpUserId", "SchemaId", "KeyPairs", "DsnpKeys", "Pages", "DsnpKeys", });
    internal_static_ImportBundles_ImportBundle_GraphKeyPair_descriptor =
      internal_static_ImportBundles_ImportBundle_descriptor.getNestedTypes().get(0);
    internal_static_ImportBundles_ImportBundle_GraphKeyPair_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_ImportBundles_ImportBundle_GraphKeyPair_descriptor,
        new java.lang.String[] { "KeyType", "PublicKey", "SecretKey", });
    internal_static_Connection_descriptor =
      getDescriptor().getMessageTypes().get(4);
    internal_static_Connection_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Connection_descriptor,
        new java.lang.String[] { "DsnpUserId", "SchemaId", });
    internal_static_Actions_descriptor =
      getDescriptor().getMessageTypes().get(5);
    internal_static_Actions_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Actions_descriptor,
        new java.lang.String[] { "Actions", "Options", "Options", });
    internal_static_Actions_ActionOptions_descriptor =
      internal_static_Actions_descriptor.getNestedTypes().get(0);
    internal_static_Actions_ActionOptions_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Actions_ActionOptions_descriptor,
        new java.lang.String[] { "IgnoreExistingConnections", "IgnoreMissingConnections", "DisableAutoCommit", });
    internal_static_Actions_Action_descriptor =
      internal_static_Actions_descriptor.getNestedTypes().get(1);
    internal_static_Actions_Action_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Actions_Action_descriptor,
        new java.lang.String[] { "ConnectAction", "DisconnectAction", "AddKeyAction", "Inner", });
    internal_static_Actions_Action_ConnectAction_descriptor =
      internal_static_Actions_Action_descriptor.getNestedTypes().get(0);
    internal_static_Actions_Action_ConnectAction_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Actions_Action_ConnectAction_descriptor,
        new java.lang.String[] { "OwnerDsnpUserId", "Connection", "DsnpKeys", "DsnpKeys", });
    internal_static_Actions_Action_DisconnectAction_descriptor =
      internal_static_Actions_Action_descriptor.getNestedTypes().get(1);
    internal_static_Actions_Action_DisconnectAction_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Actions_Action_DisconnectAction_descriptor,
        new java.lang.String[] { "OwnerDsnpUserId", "Connection", });
    internal_static_Actions_Action_AddGraphKey_descriptor =
      internal_static_Actions_Action_descriptor.getNestedTypes().get(2);
    internal_static_Actions_Action_AddGraphKey_fieldAccessorTable = new
      com.google.protobuf.GeneratedMessageV3.FieldAccessorTable(
        internal_static_Actions_Action_AddGraphKey_descriptor,
        new java.lang.String[] { "OwnerDsnpUserId", "NewPublicKey", });
  }

  // @@protoc_insertion_point(outer_class_scope)
}