import { expect } from '@jest/globals';
import { GraphKeyType, GraphKeyPair, DsnpKeys, PageData, ImportBundle } from './models';
import { ImportBundleBuilder } from './import-bundle-builder';

describe('ImportBundleBuilder', () => {
  it('should build the import bundle correctly', () => {
    const dsnpUserId = '1000';
    const schemaId = 123;
    const keyPairs: GraphKeyPair[] = [
      {
        keyType: GraphKeyType.X25519,
        publicKey: new Uint8Array([1, 2, 3]),
        secretKey: new Uint8Array([4, 5, 6]),
      },
    ];
    const dsnpKeys: DsnpKeys = {
      dsnpUserId: '1000',
      keysHash: 456,
      keys: [{ index: 0, content: new Uint8Array([7, 8, 9]) }],
    };
    const pages: PageData[] = [
      {
        pageId: 1,
        content: new Uint8Array([10, 11, 12]),
        contentHash: 789,
      },
    ];

    const importBundle: ImportBundle = ImportBundleBuilder.setDsnpUserId(dsnpUserId)
      .setSchemaId(schemaId)
      .addGraphKeyPair(GraphKeyType.X25519, new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6]))
      .setDsnpKeys(dsnpKeys)
      .addPageData(1, new Uint8Array([10, 11, 12]), 789)
      .build();

    expect(importBundle).toEqual({
      dsnpUserId,
      schemaId,
      keyPairs,
      dsnpKeys,
      pages,
    });
  });
});
