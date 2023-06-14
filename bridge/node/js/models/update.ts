export interface PersistPageUpdate {
    type: "PersistPage";
    owner_dsnp_user_id: string;
    schema_id: string;
    page_id: string;
    prev_hash: Uint8Array;
    payload: Uint8Array;
}
  
export interface DeletePageUpdate {
    type: "DeletePage";
    owner_dsnp_user_id: string;
    schema_id: string;
    page_id: string;
    prev_hash: Uint8Array;
}
  
export interface AddKeyUpdate {
    type: "AddKey";
    owner_dsnp_user_id: string;
    prev_hash: Uint8Array;
    payload: Uint8Array;
}

export type Update = PersistPageUpdate | DeletePageUpdate | AddKeyUpdate;
