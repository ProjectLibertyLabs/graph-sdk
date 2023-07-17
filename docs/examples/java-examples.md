# Java GraphSDK Example

## Setting up package dependencies

The GraphSDK JAR package is hosted in a GitHub Maven repository. The URL for the Maven repo is:
https://maven.pkg.github.com/LibertyDSNP/graph-sdk

and the package is: `io.amplica.graphsdk`

## Prerequisites

Since GraphSDK does not itself interact with the blockchain, for the following examples, we'll make use of a proposed class, `FrequencyService`, that provides an API to Frequency blockchain RPCs and Extrinsics. The structure/syntax for your own blockchain interface may be somewhat different.

## Basic Social Graph Workflow

The following code examples illustrate how to implement various steps in the workflows represented [here](basic-flow.md)

## Code Example

```kotlin
  fun addNewGraphKeyOrReadPublicKeysWorkflow() {
    // Graph keys are x25519 and different from msa control keys which are sr25519 and should not get mixed
    val newKeyPair = Graph.generateKeyPair(GraphKeyType.X25519)
    val msaId = 1L;
    val publicKey = newKeyPair.publicKey

    // ----- SETUP GRAPH ----- //
    // initialize the Graph in memory
    Configuration configuration = Configuration.getMainnet()
    val graph = Graph(configuration)

    // Get the schema ID for the graph keys
    val publicKeySchemaId = configuration
    .getEnvironment()
    .getConfig()
    .getGraphPublicKeySchemaId()

    // Retrieve current public keys from the chain. We need the content
    // hash of the current list of keys in order to send an update
    val currentKeys = frequencyClient.getItemizedStorage(msaId, schemaId.value).join()
    val keyContentHash = currentKeys.contentHash;

    // create the request to add a new key to the graph
    val graphActions = Actions.newBuilder().addActions(
      Actions.Action.newBuilder().setAddKeyAction(
        Actions.Action.AddGraphKey.newBuilder()
          .setOwnerDsnpUserId(msaId.toLong())
          .setNewPublicKey(publicKey)
          .build()
      )
    ).build()

    // add new key to graph
    graph.applyActions(graphActions)

    // export graph updates which in this case is only the added new key
    val updates = graph.exportUpdates()

    // ----- PUBLISH TO BLOCKCHAIN ----- //
    // map exported key into chain request
    val itemizedActions: List<ItemAction> = updates.filter { u -> u.hasAddKey() }.map { update ->
      AddItemAction.from(update.addKey.payload.toByteArray())
    }

    // finally, add the new exported key to blockchain
    // here we are using `createApplyItemActions` for simplicity but in reality this should be done using
    // `createApplyItemActionsWithSignature` since user should sign the payload of adding a new graph key with one of
    // their msa control keys
    val retVal =
      frequencyClient.createApplyItemActions(msaId, schemaId.value, keyContentHash.toBigInteger(), itemizedActions)
        .join()

    // verify the chain update
    Assertions.assertThat(retVal.isRight()).isTrue
    val pageUpdated: ItemizedPageUpdated = retVal.getOrThrow()
    Assertions.assertThat(pageUpdated.schemaId).isEqualTo(schemaId)
    Assertions.assertThat(pageUpdated.msaId).isEqualTo(MessageSourceId(msaId))
    Assertions.assertThat(pageUpdated.previousHash).isEqualTo(PageHash(schemaPageHash.toLong()))
    Assertions.assertThat(pageUpdated.currentHash).isNotEqualTo(PageHash(schemaPageHash.toLong()))

    // retrieve all the published keys to verify existing of new key
    val publishedKeys = frequencyClient.getItemizedStorage(msaId, schemaId.value).join()
    Assertions.assertThat(publishedKeys).isNotNull
    Assertions.assertThat(publishedKeys.items).isNotNull
    Assertions.assertThat(publishedKeys.items.size).isEqualTo(1)
    Assertions.assertThat(publishedKeys.items[0].index).isEqualTo(0)

    // ----- DESERIALIZATION OF PUBLISHED ON-CHAIN KEYS ----- //
    // Deserialize Published Public Graph Keys to raw form
    val mappedKeys = publishedKeys.items.map { i ->
      KeyData.newBuilder()
        .setIndex(i.index)
        .setContent(
          // substring use is to skip the `0x` in front of the payload
          ByteString.fromHex(i.payload.substring(2)))
        .build()
    }
    val dsnpKeys = DsnpKeys.newBuilder()
      .setDsnpUserId(msaId.toLong())
      .setKeysHash(publishedKeys.contentHash)
      .addAllKeys(mappedKeys)
      .build()

    val deserializedKeys = Graph.deserializeDsnpKeys(dsnpKeys)
    Assertions.assertThat(deserializedKeys).isNotNull
    Assertions.assertThat(deserializedKeys.size).isEqualTo(1)
    Assertions.assertThat(deserializedKeys[0].keyId).isEqualTo(0L)
    Assertions.assertThat(deserializedKeys[0].key).isEqualTo(publicKey)
  }

  fun publicGraphUpdateWorkflow() {
    val msaId = 1L // Dummy ID for example

    // ----- FETCH PUBLIC GRAPH FROM CHAIN ----- //
    val configuration = Configuration(Configuration.getMainnet())
    val publicFollowSchemaId = configuration.getSchemaId(ConnectionType.FollowPublic)
    val pages = frequencyClient.getPaginatedStorage(msaId, publicFollowSchemaId).join()

    val bundlePages = pages.map { page ->
      PageData.newBuilder()
        .setPageId(page.page_id)
        .setContentHash(page.content_hash.toInt())
        // substring use is to skip the `0x` in front of the payload
        .setContent(ByteString.fromHex(page.payload.substring(2)))
        .build()
    }
    val bundles = ImportBundles.newBuilder()
      .addBundles(
        ImportBundles.ImportBundle.newBuilder()
          .setDsnpUserId(msaId.toLong())
          .setSchemaId(publicFollowSchemaId)
          .setDsnpKeys(DsnpKeys.newBuilder().setDsnpUserId(msaId.toLong()).build())
          .addAllPages(bundlePages)
          .build()
      ).build()


    // ----- SETUP GRAPH ----- //
    // initialize the Graph in memory
    val graph = Graph(configuration)
    graph.importUserData(bundles)

    // create the request to add new connections to graph
    val newConnectionMsaId = 100L;
    val graphActions = Actions.newBuilder().addActions(
      Actions.Action.newBuilder().setConnectAction(
        Actions.Action.ConnectAction.newBuilder()
          .setOwnerDsnpUserId(msaId.toLong())
          .setConnection(
            Connection.newBuilder().setDsnpUserId(newConnectionMsaId).setSchemaId(publicFollowSchemaId).build()
          ).build()
      )
    ).build()

    // add new connections to the graph
    graph.applyActions(graphActions)

    // export graph updates which in this case is graph page update
    val updates = graph.exportUpdates()

    // ----- PUBLISH TO BLOCKCHAIN ----- //
    for (update in updates) {
      if (update.hasPersist()) {
        val persist = update.persist
        // finally, update the graph on chain with new connections
        val retVal =
          frequencyClient.createUpsertPage(
            msaId,
            publicFollowSchemaId,
            persist.pageId,
            persist.prevHash.toBigInteger(),
            persist.payload.toByteArray()
          )
            .join()

        // verify the chain update
        Assertions.assertThat(retVal.isRight()).isTrue
        val pageUpdated: PaginatedPageUpdated = retVal.getOrThrow()
        Assertions.assertThat(pageUpdated.previousHash).isEqualTo(PageHash(persist.prevHash.toLong()))
        Assertions.assertThat(pageUpdated.currentHash).isNotEqualTo(PageHash(persist.prevHash.toLong()))
      }
    }
  }

  fun privateFollowGraphUpdateWorkflow() : Triple<SchemaId, SchemaId, ImportBundles.ImportBundle.GraphKeyPair> {
    val msaId = 1L
    val newConnectionMsaId = 100L

    // ----- FETCH GRAPH KEYS FROM CHAIN ----- //
    // This logic was shown above; here extracted to a function 'fetchGraphKeys'
    val (deserializedKeys, dsnpKeys) = fetchGraphKeys(keysSchemaId, msaId)
    val keyPairs: ArrayList<ImportBundles.ImportBundle.GraphKeyPair> = ArrayList()

    // Here we must have retrieved the public/private graph keypairs from the wallet
    // or provider cache.
    val graphKeys: ArrayList<GraphKeyPair> = ??
    for(graphKeyPair in graphKeys) {
        keyPairs.add(
          ImportBundles.ImportBundle.GraphKeyPair.newBuilder()
            .setKeyType(GraphKeyType.X25519)
            .setPublicKey(graphKeyPair.publicKey)
            .setSecretKey(ByteString.copyFrom(graphKeyPair.secretKey.toByteArray()))
            .build()
        )
      }
    }

    // ----- FETCH PRIVATE FOLLOW GRAPH FROM CHAIN ----- //
    val configuration = Configuration(Configuration.getMainnet())
    val privateFollowSchemaId = configuration.getSchemaId(ConnectionType.FollowPrivate)
    val pages = frequencyClient.getPaginatedStorage(msaId, privateFollowSchemaId).join()

    val bundlePages = pages.map { page ->
      PageData.newBuilder()
        .setPageId(page.page_id)
        .setContentHash(page.content_hash.toInt())
        // substring use is to skip the `0x` in front of the payload
        .setContent(ByteString.fromHex(page.payload.substring(2)))
        .build()
    }
    val bundles = ImportBundles.newBuilder()
      .addBundles(
        ImportBundles.ImportBundle.newBuilder()
          .setDsnpUserId(msaId.toLong())
          .setSchemaId(privateFollowSchemaId)
          .setDsnpKeys(dsnpKeys)
          .addAllPages(bundlePages)
           // NOTICE that for private graph we have to set private keys in here but that's not required for public graph
          .addAllKeyPairs(keyPairs)
          .build()
      ).build()


    // ----- SETUP GRAPH ----- //
    // initializing the Graph in memory
    val graph = Graph(configuration)
    graph.importUserData(bundles)

    // creating the request to add new connections to graph
    val graphActions = Actions.newBuilder().addActions(
      Actions.Action.newBuilder().setConnectAction(
        Actions.Action.ConnectAction.newBuilder()
          .setOwnerDsnpUserId(msaId.toLong())
          .setConnection(
            Connection.newBuilder().setDsnpUserId(newConnectionMsaId).setSchemaId(privateFollowSchemaId).build()
          ).build()
      )
    ).build()

    // adding new connections to the graph
    graph.applyActions(graphActions)

    // exporting graph updates which in this case is graph page update
    val updates = graph.exportUpdates()

    // ----- PUBLISH TO BLOCKCHAIN ----- //
    for (update in updates) {
      if (update.hasPersist()) {
        val persist = update.persist
        // finally, updating the graph on chain with new connections
        val retVal =
          frequencyClient.createUpsertPage(
            msaId,
            privateFollowSchemaId,
            persist.pageId,
            persist.prevHash.toBigInteger(),
            persist.payload.toByteArray()
          )
            .join()

        // verifying the chain update
        Assertions.assertThat(retVal.isRight()).isTrue
        val pageUpdated: PaginatedPageUpdated = retVal.getOrThrow()
        Assertions.assertThat(pageUpdated.previousHash).isEqualTo(PageHash(persist.prevHash.toLong()))
        Assertions.assertThat(pageUpdated.currentHash).isNotEqualTo(PageHash(persist.prevHash.toLong()))
      }
    }
    return Triple(schemaId, keysSchemaId, newKeyPair)
  }

  @Test
  fun privateFriendshipGraphUpdateWorkflow() {
    // --- setup other msa -----//
    val msaId = 1L
    val newConnectionMsaId = 100L
    val ownerKeyPair = ?? // owner's Graph encryption keys,

    // ----- FETCH GRAPH KEYS FROM CHAIN ----- //
    // This logic was shown above; here extracted to a function 'fetchGraphKeys'
    val (deserializedKeys, dsnpKeys) = fetchGraphKeys(keysSchemaId, msaId)

    // Here: different from PrivateFollow, for PrivateFriendship we also require
    // the other user's public grap key
    val (_, newConnectionDsnpKeys) = fetchGraphKeys(keysSchemaId, newConnectionMsaId);

    val keyPairs: ArrayList<ImportBundles.ImportBundle.GraphKeyPair> = ArrayList()

    // Here we must have retrieved the public/private graph keypairs from the wallet
    // or provider cache.
    val graphKeys: ArrayList<GraphKeyPair> = ??
    for(graphKeyPair in graphKeys) {
        keyPairs.add(
          ImportBundles.ImportBundle.GraphKeyPair.newBuilder()
            .setKeyType(GraphKeyType.X25519)
            .setPublicKey(graphKeyPair.publicKey)
            .setSecretKey(ByteString.copyFrom(graphKeyPair.secretKey.toByteArray()))
            .build()
        )
      }
    }

    val (connectionDeserializedKeys, connectionDsnpKeys) = fetchGraphKeys(keysSchemaId, newConnectionMsaId1.value)

    // ----- FETCH PRIVATE FRIENDSHIP GRAPH FROM CHAIN ----- //
    val configuration = Configuration(Configuration.getMainnet())
    val privateFriendshipSchemaId = configuration.getSchemaId(ConnectionType.FriendshipPrivate)

    // Prepare bundle input
    val ownerPages = frequencyClient.getPaginatedStorage(msaId, privateFriendshipSchemaId).join()
    val bundlePages = ownerPages.map { page ->
      PageData.newBuilder()
        .setPageId(page.page_id)
        .setContentHash(page.content_hash.toInt())
        // substring use is to skip the `0x` in front of the payload
        .setContent(ByteString.fromHex(page.payload.substring(2)))
        .build()
    }
    val bundles = ImportBundles.newBuilder()
      .addBundles(
        ImportBundles.ImportBundle.newBuilder()
          .setDsnpUserId(msaId.toLong())
          .setSchemaId(privateFriendshipSchemaId)
          .setDsnpKeys(ownerDsnpKeys)
          .addAllPages(bundlePages)
          // NOTICE that for private graph we have to set private keys in here but that's not required for public graph
          .addAllKeyPairs(ownerKeyPairs)
          .build()
      .addBundles(
        ImportBundles.ImportBundle.newBuilder()
          .setDsnpUserId(newConnectionMsaId)
          .setDsnpKeys(connectionDsnpKeys)
          .build()
      ).build()

    // ----- SETUP GRAPH ----- //
    // initializing the Graph in memory
    val graph = Graph(configuration)
    graph.importUserData(bundles)

    // creating the request to add new connections to graph
    val graphActions = Actions.newBuilder().addActions(
      Actions.Action.newBuilder().setConnectAction(
        Actions.Action.ConnectAction.newBuilder()
          .setOwnerDsnpUserId(msaId.toLong())
          .setConnection(
            Connection.newBuilder().setDsnpUserId(newConnectionMsaId1.value.toLong()).setSchemaId(privateFriendshipSchemaId).build()
          )
          // importing the published keys of the connection so the PRID can get calculated
          .setDsnpKeys(connectionDsnpKeys)
          .build()
      )
    ).build()

    // adding new connections to the graph
    graph.applyActions(graphActions)

    // exporting graph updates which in this case is graph page update
    val updates = graph.exportUpdates()

    // ----- PUBLISH TO BLOCKCHAIN ----- //
    for (update in updates) {
      if (update.hasPersist()) {
        val persist = update.persist
        // finally, updating the graph on chain with new connections
        val retVal =
          frequencyClient.createUpsertPage(
            msaId,
            privateFriendshipSchemaId,
            persist.pageId,
            persist.prevHash.toBigInteger(),
            persist.payload.toByteArray()
          )
            .join()

        // verifying the chain update
        Assertions.assertThat(retVal.isRight()).isTrue
        val pageUpdated: PaginatedPageUpdated = retVal.getOrThrow()
        Assertions.assertThat(pageUpdated.previousHash).isEqualTo(PageHash(persist.prevHash.toLong()))
        Assertions.assertThat(pageUpdated.currentHash).isNotEqualTo(PageHash(persist.prevHash.toLong()))
      }
    }
  }
```
