import { GraphKeyType, GraphKeyPair, DsnpKeys, ImportBundle, PageData } from "./models";

export class ImportBundleBuilder {
  private dsnpUserId: string;
  private schemaId: number;
  private keyPairs: GraphKeyPair[];
  private dsnpKeys: DsnpKeys;
  private pages: PageData[];

  constructor() {
    this.dsnpUserId = "";
    this.schemaId = 0;
    this.keyPairs = [];
    this.dsnpKeys = { dsnpUserId: "", keysHash: 0, keys: [] };
    this.pages = [];
  }

  setDsnpUserId(dsnpUserId: string): ImportBundleBuilder {
    this.dsnpUserId = dsnpUserId;
    return this;
  }

  setSchemaId(schemaId: number): ImportBundleBuilder {
    this.schemaId = schemaId;
    return this;
  }

  addGraphKeyPair(keyType: GraphKeyType, publicKey: Uint8Array, secretKey: Uint8Array): ImportBundleBuilder {
    this.keyPairs.push({ keyType, publicKey, secretKey });
    return this;
  }

  setDsnpKeys(dsnpKeys: DsnpKeys): ImportBundleBuilder {
    this.dsnpKeys = dsnpKeys;
    return this;
  }

  addPageData(pageId: number, content: Uint8Array, contentHash: number): ImportBundleBuilder {
    this.pages.push({ pageId, content, contentHash });
    return this;
  }

  build(): ImportBundle {
    const importBundle: ImportBundle = {
      dsnpUserId: this.dsnpUserId,
      schemaId: this.schemaId,
      keyPairs: this.keyPairs,
      dsnpKeys: this.dsnpKeys,
      pages: this.pages,
    };

    // Reset the instance properties for the next build
    this.dsnpUserId = "";
    this.schemaId = 0;
    this.keyPairs = [];
    this.dsnpKeys = { dsnpUserId: "", keysHash: 0, keys: [] };
    this.pages = [];

    return importBundle;
  }
}
