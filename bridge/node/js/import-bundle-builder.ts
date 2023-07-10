import { GraphKeyType, GraphKeyPair, DsnpKeys, ImportBundle, PageData } from "./models";

export interface IImportBundleBuilder {
  dsnpUserId?: string;
  schemaId?: number;
  keyPairs?: GraphKeyPair[];
  dsnpKeys?: DsnpKeys;
  pages?: PageData[];
}

export class ImportBundleBuilder {
  private values: IImportBundleBuilder = {};

  constructor(values?: IImportBundleBuilder) {
    if (values !== undefined) {
      Object.assign(this.values, values);
    }
  }

  public withDsnpUserId(dsnpUserId: string): ImportBundleBuilder {
    return new ImportBundleBuilder({ ...this.values, dsnpUserId });
  }

  public withSchemaId(schemaId: number): ImportBundleBuilder {
    return new ImportBundleBuilder({ ...this.values, schemaId });
  }

  public withGraphKeyPair(keyType: GraphKeyType, publicKey: Uint8Array, secretKey: Uint8Array): ImportBundleBuilder {
    const keyPairs = this.values.keyPairs ? [...this.values.keyPairs] : [];
    keyPairs.push({ keyType, publicKey, secretKey });
    return new ImportBundleBuilder({ ...this.values, keyPairs });
  }

  public withDsnpKeys(dsnpKeys: DsnpKeys): ImportBundleBuilder {
    return new ImportBundleBuilder({ ...this.values, dsnpKeys });
  }

  public withPageData(pageId: number, content: Uint8Array, contentHash: number): ImportBundleBuilder {
    const pages = this.values.pages ? [...this.values.pages] : [];
    pages.push({ pageId, content, contentHash });
    return new ImportBundleBuilder({ ...this.values, pages });
  }

  public build(): ImportBundle {
    const importBundle: ImportBundle = {
      dsnpUserId: this.values.dsnpUserId || '',
      schemaId: this.values.schemaId || 0,
      keyPairs: this.values.keyPairs || [],
      dsnpKeys: this.values.dsnpKeys || { dsnpUserId: '', keysHash: 0, keys: [] },
      pages: this.values.pages || [],
    };

    return importBundle;
  }
}
