export interface KeyData {
    index: number;
    content: Uint8Array;
}
  
export interface DsnpKeys {
    dsnp_user_id: number;
    keys_hash: number;
    keys: KeyData[];
}

export enum GraphKeyType {
    X25519 = 0,
}

export interface GraphKeyPair {
    key_type: GraphKeyType;
    public_key: Uint8Array;
    secret_key: Uint8Array;
}

export interface PageData {
    page_id: string;
    content: Uint8Array;
    content_hash: Uint8Array;
}
  
export interface ImportBundle {
    dsnp_user_id: number;
    schema_id: number
    key_pairs: GraphKeyPair[];
    dsnp_keys: DsnpKeys;
    pages: PageData[];
}
