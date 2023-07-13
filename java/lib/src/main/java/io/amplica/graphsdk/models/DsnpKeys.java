// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: bridge/common/protos/input.proto

package io.amplica.graphsdk.models;

/**
 * Protobuf type {@code DsnpKeys}
 */
public final class DsnpKeys extends
    com.google.protobuf.GeneratedMessageV3 implements
    // @@protoc_insertion_point(message_implements:DsnpKeys)
    DsnpKeysOrBuilder {
private static final long serialVersionUID = 0L;
  // Use DsnpKeys.newBuilder() to construct.
  private DsnpKeys(com.google.protobuf.GeneratedMessageV3.Builder<?> builder) {
    super(builder);
  }
  private DsnpKeys() {
    keys_ = java.util.Collections.emptyList();
  }

  @java.lang.Override
  @SuppressWarnings({"unused"})
  protected java.lang.Object newInstance(
      UnusedPrivateParameter unused) {
    return new DsnpKeys();
  }

  @java.lang.Override
  public final com.google.protobuf.UnknownFieldSet
  getUnknownFields() {
    return this.unknownFields;
  }
  public static final com.google.protobuf.Descriptors.Descriptor
      getDescriptor() {
    return io.amplica.graphsdk.models.Input.internal_static_DsnpKeys_descriptor;
  }

  @java.lang.Override
  protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internalGetFieldAccessorTable() {
    return io.amplica.graphsdk.models.Input.internal_static_DsnpKeys_fieldAccessorTable
        .ensureFieldAccessorsInitialized(
            io.amplica.graphsdk.models.DsnpKeys.class, io.amplica.graphsdk.models.DsnpKeys.Builder.class);
  }

  public static final int DSNP_USER_ID_FIELD_NUMBER = 1;
  private long dsnpUserId_ = 0L;
  /**
   * <code>uint64 dsnp_user_id = 1;</code>
   * @return The dsnpUserId.
   */
  @java.lang.Override
  public long getDsnpUserId() {
    return dsnpUserId_;
  }

  public static final int KEYS_HASH_FIELD_NUMBER = 2;
  private int keysHash_ = 0;
  /**
   * <code>uint32 keys_hash = 2;</code>
   * @return The keysHash.
   */
  @java.lang.Override
  public int getKeysHash() {
    return keysHash_;
  }

  public static final int KEYS_FIELD_NUMBER = 3;
  @SuppressWarnings("serial")
  private java.util.List<io.amplica.graphsdk.models.KeyData> keys_;
  /**
   * <code>repeated .KeyData keys = 3;</code>
   */
  @java.lang.Override
  public java.util.List<io.amplica.graphsdk.models.KeyData> getKeysList() {
    return keys_;
  }
  /**
   * <code>repeated .KeyData keys = 3;</code>
   */
  @java.lang.Override
  public java.util.List<? extends io.amplica.graphsdk.models.KeyDataOrBuilder> 
      getKeysOrBuilderList() {
    return keys_;
  }
  /**
   * <code>repeated .KeyData keys = 3;</code>
   */
  @java.lang.Override
  public int getKeysCount() {
    return keys_.size();
  }
  /**
   * <code>repeated .KeyData keys = 3;</code>
   */
  @java.lang.Override
  public io.amplica.graphsdk.models.KeyData getKeys(int index) {
    return keys_.get(index);
  }
  /**
   * <code>repeated .KeyData keys = 3;</code>
   */
  @java.lang.Override
  public io.amplica.graphsdk.models.KeyDataOrBuilder getKeysOrBuilder(
      int index) {
    return keys_.get(index);
  }

  private byte memoizedIsInitialized = -1;
  @java.lang.Override
  public final boolean isInitialized() {
    byte isInitialized = memoizedIsInitialized;
    if (isInitialized == 1) return true;
    if (isInitialized == 0) return false;

    memoizedIsInitialized = 1;
    return true;
  }

  @java.lang.Override
  public void writeTo(com.google.protobuf.CodedOutputStream output)
                      throws java.io.IOException {
    if (dsnpUserId_ != 0L) {
      output.writeUInt64(1, dsnpUserId_);
    }
    if (keysHash_ != 0) {
      output.writeUInt32(2, keysHash_);
    }
    for (int i = 0; i < keys_.size(); i++) {
      output.writeMessage(3, keys_.get(i));
    }
    getUnknownFields().writeTo(output);
  }

  @java.lang.Override
  public int getSerializedSize() {
    int size = memoizedSize;
    if (size != -1) return size;

    size = 0;
    if (dsnpUserId_ != 0L) {
      size += com.google.protobuf.CodedOutputStream
        .computeUInt64Size(1, dsnpUserId_);
    }
    if (keysHash_ != 0) {
      size += com.google.protobuf.CodedOutputStream
        .computeUInt32Size(2, keysHash_);
    }
    for (int i = 0; i < keys_.size(); i++) {
      size += com.google.protobuf.CodedOutputStream
        .computeMessageSize(3, keys_.get(i));
    }
    size += getUnknownFields().getSerializedSize();
    memoizedSize = size;
    return size;
  }

  @java.lang.Override
  public boolean equals(final java.lang.Object obj) {
    if (obj == this) {
     return true;
    }
    if (!(obj instanceof io.amplica.graphsdk.models.DsnpKeys)) {
      return super.equals(obj);
    }
    io.amplica.graphsdk.models.DsnpKeys other = (io.amplica.graphsdk.models.DsnpKeys) obj;

    if (getDsnpUserId()
        != other.getDsnpUserId()) return false;
    if (getKeysHash()
        != other.getKeysHash()) return false;
    if (!getKeysList()
        .equals(other.getKeysList())) return false;
    if (!getUnknownFields().equals(other.getUnknownFields())) return false;
    return true;
  }

  @java.lang.Override
  public int hashCode() {
    if (memoizedHashCode != 0) {
      return memoizedHashCode;
    }
    int hash = 41;
    hash = (19 * hash) + getDescriptor().hashCode();
    hash = (37 * hash) + DSNP_USER_ID_FIELD_NUMBER;
    hash = (53 * hash) + com.google.protobuf.Internal.hashLong(
        getDsnpUserId());
    hash = (37 * hash) + KEYS_HASH_FIELD_NUMBER;
    hash = (53 * hash) + getKeysHash();
    if (getKeysCount() > 0) {
      hash = (37 * hash) + KEYS_FIELD_NUMBER;
      hash = (53 * hash) + getKeysList().hashCode();
    }
    hash = (29 * hash) + getUnknownFields().hashCode();
    memoizedHashCode = hash;
    return hash;
  }

  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(
      java.nio.ByteBuffer data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(
      java.nio.ByteBuffer data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(
      com.google.protobuf.ByteString data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(
      com.google.protobuf.ByteString data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(byte[] data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(
      byte[] data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseDelimitedFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseDelimitedFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input, extensionRegistry);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(
      com.google.protobuf.CodedInputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static io.amplica.graphsdk.models.DsnpKeys parseFrom(
      com.google.protobuf.CodedInputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }

  @java.lang.Override
  public Builder newBuilderForType() { return newBuilder(); }
  public static Builder newBuilder() {
    return DEFAULT_INSTANCE.toBuilder();
  }
  public static Builder newBuilder(io.amplica.graphsdk.models.DsnpKeys prototype) {
    return DEFAULT_INSTANCE.toBuilder().mergeFrom(prototype);
  }
  @java.lang.Override
  public Builder toBuilder() {
    return this == DEFAULT_INSTANCE
        ? new Builder() : new Builder().mergeFrom(this);
  }

  @java.lang.Override
  protected Builder newBuilderForType(
      com.google.protobuf.GeneratedMessageV3.BuilderParent parent) {
    Builder builder = new Builder(parent);
    return builder;
  }
  /**
   * Protobuf type {@code DsnpKeys}
   */
  public static final class Builder extends
      com.google.protobuf.GeneratedMessageV3.Builder<Builder> implements
      // @@protoc_insertion_point(builder_implements:DsnpKeys)
      io.amplica.graphsdk.models.DsnpKeysOrBuilder {
    public static final com.google.protobuf.Descriptors.Descriptor
        getDescriptor() {
      return io.amplica.graphsdk.models.Input.internal_static_DsnpKeys_descriptor;
    }

    @java.lang.Override
    protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
        internalGetFieldAccessorTable() {
      return io.amplica.graphsdk.models.Input.internal_static_DsnpKeys_fieldAccessorTable
          .ensureFieldAccessorsInitialized(
              io.amplica.graphsdk.models.DsnpKeys.class, io.amplica.graphsdk.models.DsnpKeys.Builder.class);
    }

    // Construct using io.amplica.graphsdk.models.DsnpKeys.newBuilder()
    private Builder() {

    }

    private Builder(
        com.google.protobuf.GeneratedMessageV3.BuilderParent parent) {
      super(parent);

    }
    @java.lang.Override
    public Builder clear() {
      super.clear();
      bitField0_ = 0;
      dsnpUserId_ = 0L;
      keysHash_ = 0;
      if (keysBuilder_ == null) {
        keys_ = java.util.Collections.emptyList();
      } else {
        keys_ = null;
        keysBuilder_.clear();
      }
      bitField0_ = (bitField0_ & ~0x00000004);
      return this;
    }

    @java.lang.Override
    public com.google.protobuf.Descriptors.Descriptor
        getDescriptorForType() {
      return io.amplica.graphsdk.models.Input.internal_static_DsnpKeys_descriptor;
    }

    @java.lang.Override
    public io.amplica.graphsdk.models.DsnpKeys getDefaultInstanceForType() {
      return io.amplica.graphsdk.models.DsnpKeys.getDefaultInstance();
    }

    @java.lang.Override
    public io.amplica.graphsdk.models.DsnpKeys build() {
      io.amplica.graphsdk.models.DsnpKeys result = buildPartial();
      if (!result.isInitialized()) {
        throw newUninitializedMessageException(result);
      }
      return result;
    }

    @java.lang.Override
    public io.amplica.graphsdk.models.DsnpKeys buildPartial() {
      io.amplica.graphsdk.models.DsnpKeys result = new io.amplica.graphsdk.models.DsnpKeys(this);
      buildPartialRepeatedFields(result);
      if (bitField0_ != 0) { buildPartial0(result); }
      onBuilt();
      return result;
    }

    private void buildPartialRepeatedFields(io.amplica.graphsdk.models.DsnpKeys result) {
      if (keysBuilder_ == null) {
        if (((bitField0_ & 0x00000004) != 0)) {
          keys_ = java.util.Collections.unmodifiableList(keys_);
          bitField0_ = (bitField0_ & ~0x00000004);
        }
        result.keys_ = keys_;
      } else {
        result.keys_ = keysBuilder_.build();
      }
    }

    private void buildPartial0(io.amplica.graphsdk.models.DsnpKeys result) {
      int from_bitField0_ = bitField0_;
      if (((from_bitField0_ & 0x00000001) != 0)) {
        result.dsnpUserId_ = dsnpUserId_;
      }
      if (((from_bitField0_ & 0x00000002) != 0)) {
        result.keysHash_ = keysHash_;
      }
    }

    @java.lang.Override
    public Builder clone() {
      return super.clone();
    }
    @java.lang.Override
    public Builder setField(
        com.google.protobuf.Descriptors.FieldDescriptor field,
        java.lang.Object value) {
      return super.setField(field, value);
    }
    @java.lang.Override
    public Builder clearField(
        com.google.protobuf.Descriptors.FieldDescriptor field) {
      return super.clearField(field);
    }
    @java.lang.Override
    public Builder clearOneof(
        com.google.protobuf.Descriptors.OneofDescriptor oneof) {
      return super.clearOneof(oneof);
    }
    @java.lang.Override
    public Builder setRepeatedField(
        com.google.protobuf.Descriptors.FieldDescriptor field,
        int index, java.lang.Object value) {
      return super.setRepeatedField(field, index, value);
    }
    @java.lang.Override
    public Builder addRepeatedField(
        com.google.protobuf.Descriptors.FieldDescriptor field,
        java.lang.Object value) {
      return super.addRepeatedField(field, value);
    }
    @java.lang.Override
    public Builder mergeFrom(com.google.protobuf.Message other) {
      if (other instanceof io.amplica.graphsdk.models.DsnpKeys) {
        return mergeFrom((io.amplica.graphsdk.models.DsnpKeys)other);
      } else {
        super.mergeFrom(other);
        return this;
      }
    }

    public Builder mergeFrom(io.amplica.graphsdk.models.DsnpKeys other) {
      if (other == io.amplica.graphsdk.models.DsnpKeys.getDefaultInstance()) return this;
      if (other.getDsnpUserId() != 0L) {
        setDsnpUserId(other.getDsnpUserId());
      }
      if (other.getKeysHash() != 0) {
        setKeysHash(other.getKeysHash());
      }
      if (keysBuilder_ == null) {
        if (!other.keys_.isEmpty()) {
          if (keys_.isEmpty()) {
            keys_ = other.keys_;
            bitField0_ = (bitField0_ & ~0x00000004);
          } else {
            ensureKeysIsMutable();
            keys_.addAll(other.keys_);
          }
          onChanged();
        }
      } else {
        if (!other.keys_.isEmpty()) {
          if (keysBuilder_.isEmpty()) {
            keysBuilder_.dispose();
            keysBuilder_ = null;
            keys_ = other.keys_;
            bitField0_ = (bitField0_ & ~0x00000004);
            keysBuilder_ = 
              com.google.protobuf.GeneratedMessageV3.alwaysUseFieldBuilders ?
                 getKeysFieldBuilder() : null;
          } else {
            keysBuilder_.addAllMessages(other.keys_);
          }
        }
      }
      this.mergeUnknownFields(other.getUnknownFields());
      onChanged();
      return this;
    }

    @java.lang.Override
    public final boolean isInitialized() {
      return true;
    }

    @java.lang.Override
    public Builder mergeFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws java.io.IOException {
      if (extensionRegistry == null) {
        throw new java.lang.NullPointerException();
      }
      try {
        boolean done = false;
        while (!done) {
          int tag = input.readTag();
          switch (tag) {
            case 0:
              done = true;
              break;
            case 8: {
              dsnpUserId_ = input.readUInt64();
              bitField0_ |= 0x00000001;
              break;
            } // case 8
            case 16: {
              keysHash_ = input.readUInt32();
              bitField0_ |= 0x00000002;
              break;
            } // case 16
            case 26: {
              io.amplica.graphsdk.models.KeyData m =
                  input.readMessage(
                      io.amplica.graphsdk.models.KeyData.parser(),
                      extensionRegistry);
              if (keysBuilder_ == null) {
                ensureKeysIsMutable();
                keys_.add(m);
              } else {
                keysBuilder_.addMessage(m);
              }
              break;
            } // case 26
            default: {
              if (!super.parseUnknownField(input, extensionRegistry, tag)) {
                done = true; // was an endgroup tag
              }
              break;
            } // default:
          } // switch (tag)
        } // while (!done)
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        throw e.unwrapIOException();
      } finally {
        onChanged();
      } // finally
      return this;
    }
    private int bitField0_;

    private long dsnpUserId_ ;
    /**
     * <code>uint64 dsnp_user_id = 1;</code>
     * @return The dsnpUserId.
     */
    @java.lang.Override
    public long getDsnpUserId() {
      return dsnpUserId_;
    }
    /**
     * <code>uint64 dsnp_user_id = 1;</code>
     * @param value The dsnpUserId to set.
     * @return This builder for chaining.
     */
    public Builder setDsnpUserId(long value) {
      
      dsnpUserId_ = value;
      bitField0_ |= 0x00000001;
      onChanged();
      return this;
    }
    /**
     * <code>uint64 dsnp_user_id = 1;</code>
     * @return This builder for chaining.
     */
    public Builder clearDsnpUserId() {
      bitField0_ = (bitField0_ & ~0x00000001);
      dsnpUserId_ = 0L;
      onChanged();
      return this;
    }

    private int keysHash_ ;
    /**
     * <code>uint32 keys_hash = 2;</code>
     * @return The keysHash.
     */
    @java.lang.Override
    public int getKeysHash() {
      return keysHash_;
    }
    /**
     * <code>uint32 keys_hash = 2;</code>
     * @param value The keysHash to set.
     * @return This builder for chaining.
     */
    public Builder setKeysHash(int value) {
      
      keysHash_ = value;
      bitField0_ |= 0x00000002;
      onChanged();
      return this;
    }
    /**
     * <code>uint32 keys_hash = 2;</code>
     * @return This builder for chaining.
     */
    public Builder clearKeysHash() {
      bitField0_ = (bitField0_ & ~0x00000002);
      keysHash_ = 0;
      onChanged();
      return this;
    }

    private java.util.List<io.amplica.graphsdk.models.KeyData> keys_ =
      java.util.Collections.emptyList();
    private void ensureKeysIsMutable() {
      if (!((bitField0_ & 0x00000004) != 0)) {
        keys_ = new java.util.ArrayList<io.amplica.graphsdk.models.KeyData>(keys_);
        bitField0_ |= 0x00000004;
       }
    }

    private com.google.protobuf.RepeatedFieldBuilderV3<
        io.amplica.graphsdk.models.KeyData, io.amplica.graphsdk.models.KeyData.Builder, io.amplica.graphsdk.models.KeyDataOrBuilder> keysBuilder_;

    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public java.util.List<io.amplica.graphsdk.models.KeyData> getKeysList() {
      if (keysBuilder_ == null) {
        return java.util.Collections.unmodifiableList(keys_);
      } else {
        return keysBuilder_.getMessageList();
      }
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public int getKeysCount() {
      if (keysBuilder_ == null) {
        return keys_.size();
      } else {
        return keysBuilder_.getCount();
      }
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public io.amplica.graphsdk.models.KeyData getKeys(int index) {
      if (keysBuilder_ == null) {
        return keys_.get(index);
      } else {
        return keysBuilder_.getMessage(index);
      }
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder setKeys(
        int index, io.amplica.graphsdk.models.KeyData value) {
      if (keysBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensureKeysIsMutable();
        keys_.set(index, value);
        onChanged();
      } else {
        keysBuilder_.setMessage(index, value);
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder setKeys(
        int index, io.amplica.graphsdk.models.KeyData.Builder builderForValue) {
      if (keysBuilder_ == null) {
        ensureKeysIsMutable();
        keys_.set(index, builderForValue.build());
        onChanged();
      } else {
        keysBuilder_.setMessage(index, builderForValue.build());
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder addKeys(io.amplica.graphsdk.models.KeyData value) {
      if (keysBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensureKeysIsMutable();
        keys_.add(value);
        onChanged();
      } else {
        keysBuilder_.addMessage(value);
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder addKeys(
        int index, io.amplica.graphsdk.models.KeyData value) {
      if (keysBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensureKeysIsMutable();
        keys_.add(index, value);
        onChanged();
      } else {
        keysBuilder_.addMessage(index, value);
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder addKeys(
        io.amplica.graphsdk.models.KeyData.Builder builderForValue) {
      if (keysBuilder_ == null) {
        ensureKeysIsMutable();
        keys_.add(builderForValue.build());
        onChanged();
      } else {
        keysBuilder_.addMessage(builderForValue.build());
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder addKeys(
        int index, io.amplica.graphsdk.models.KeyData.Builder builderForValue) {
      if (keysBuilder_ == null) {
        ensureKeysIsMutable();
        keys_.add(index, builderForValue.build());
        onChanged();
      } else {
        keysBuilder_.addMessage(index, builderForValue.build());
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder addAllKeys(
        java.lang.Iterable<? extends io.amplica.graphsdk.models.KeyData> values) {
      if (keysBuilder_ == null) {
        ensureKeysIsMutable();
        com.google.protobuf.AbstractMessageLite.Builder.addAll(
            values, keys_);
        onChanged();
      } else {
        keysBuilder_.addAllMessages(values);
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder clearKeys() {
      if (keysBuilder_ == null) {
        keys_ = java.util.Collections.emptyList();
        bitField0_ = (bitField0_ & ~0x00000004);
        onChanged();
      } else {
        keysBuilder_.clear();
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public Builder removeKeys(int index) {
      if (keysBuilder_ == null) {
        ensureKeysIsMutable();
        keys_.remove(index);
        onChanged();
      } else {
        keysBuilder_.remove(index);
      }
      return this;
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public io.amplica.graphsdk.models.KeyData.Builder getKeysBuilder(
        int index) {
      return getKeysFieldBuilder().getBuilder(index);
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public io.amplica.graphsdk.models.KeyDataOrBuilder getKeysOrBuilder(
        int index) {
      if (keysBuilder_ == null) {
        return keys_.get(index);  } else {
        return keysBuilder_.getMessageOrBuilder(index);
      }
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public java.util.List<? extends io.amplica.graphsdk.models.KeyDataOrBuilder> 
         getKeysOrBuilderList() {
      if (keysBuilder_ != null) {
        return keysBuilder_.getMessageOrBuilderList();
      } else {
        return java.util.Collections.unmodifiableList(keys_);
      }
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public io.amplica.graphsdk.models.KeyData.Builder addKeysBuilder() {
      return getKeysFieldBuilder().addBuilder(
          io.amplica.graphsdk.models.KeyData.getDefaultInstance());
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public io.amplica.graphsdk.models.KeyData.Builder addKeysBuilder(
        int index) {
      return getKeysFieldBuilder().addBuilder(
          index, io.amplica.graphsdk.models.KeyData.getDefaultInstance());
    }
    /**
     * <code>repeated .KeyData keys = 3;</code>
     */
    public java.util.List<io.amplica.graphsdk.models.KeyData.Builder> 
         getKeysBuilderList() {
      return getKeysFieldBuilder().getBuilderList();
    }
    private com.google.protobuf.RepeatedFieldBuilderV3<
        io.amplica.graphsdk.models.KeyData, io.amplica.graphsdk.models.KeyData.Builder, io.amplica.graphsdk.models.KeyDataOrBuilder> 
        getKeysFieldBuilder() {
      if (keysBuilder_ == null) {
        keysBuilder_ = new com.google.protobuf.RepeatedFieldBuilderV3<
            io.amplica.graphsdk.models.KeyData, io.amplica.graphsdk.models.KeyData.Builder, io.amplica.graphsdk.models.KeyDataOrBuilder>(
                keys_,
                ((bitField0_ & 0x00000004) != 0),
                getParentForChildren(),
                isClean());
        keys_ = null;
      }
      return keysBuilder_;
    }
    @java.lang.Override
    public final Builder setUnknownFields(
        final com.google.protobuf.UnknownFieldSet unknownFields) {
      return super.setUnknownFields(unknownFields);
    }

    @java.lang.Override
    public final Builder mergeUnknownFields(
        final com.google.protobuf.UnknownFieldSet unknownFields) {
      return super.mergeUnknownFields(unknownFields);
    }


    // @@protoc_insertion_point(builder_scope:DsnpKeys)
  }

  // @@protoc_insertion_point(class_scope:DsnpKeys)
  private static final io.amplica.graphsdk.models.DsnpKeys DEFAULT_INSTANCE;
  static {
    DEFAULT_INSTANCE = new io.amplica.graphsdk.models.DsnpKeys();
  }

  public static io.amplica.graphsdk.models.DsnpKeys getDefaultInstance() {
    return DEFAULT_INSTANCE;
  }

  private static final com.google.protobuf.Parser<DsnpKeys>
      PARSER = new com.google.protobuf.AbstractParser<DsnpKeys>() {
    @java.lang.Override
    public DsnpKeys parsePartialFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws com.google.protobuf.InvalidProtocolBufferException {
      Builder builder = newBuilder();
      try {
        builder.mergeFrom(input, extensionRegistry);
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        throw e.setUnfinishedMessage(builder.buildPartial());
      } catch (com.google.protobuf.UninitializedMessageException e) {
        throw e.asInvalidProtocolBufferException().setUnfinishedMessage(builder.buildPartial());
      } catch (java.io.IOException e) {
        throw new com.google.protobuf.InvalidProtocolBufferException(e)
            .setUnfinishedMessage(builder.buildPartial());
      }
      return builder.buildPartial();
    }
  };

  public static com.google.protobuf.Parser<DsnpKeys> parser() {
    return PARSER;
  }

  @java.lang.Override
  public com.google.protobuf.Parser<DsnpKeys> getParserForType() {
    return PARSER;
  }

  @java.lang.Override
  public io.amplica.graphsdk.models.DsnpKeys getDefaultInstanceForType() {
    return DEFAULT_INSTANCE;
  }

}

