export interface KeyData {
    index: number;
    content: Uint8Array;
}
  
export interface DsnpKeys {
    dsnpUserId: number;
    keysHash: Uint8Array;
    keys: KeyData[];
}

export enum GraphKeyType {
    X25519 = 0,
}

export interface GraphKeyPair {
    keyType: GraphKeyType;
    publicKey: Uint8Array;
    secretKey: Uint8Array;
}

export interface PageData {
    page_id: string;
    content: Uint8Array;
    contentHash: Uint8Array;
}
  
export interface ImportBundle {
    dsnpUserId: number;
    schemaId: number;
    keyPairs: GraphKeyPair[];
    dsnpKeys: DsnpKeys;
    pages: PageData[];
}
