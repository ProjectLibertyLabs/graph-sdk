export interface PersistPageUpdate {
    type: "PersistPage";
    ownerDsnpUserId: number;
    schemaId: number;
    pageId: number;
    prevHash: Uint8Array;
    payload: Uint8Array;
}
  
export interface DeletePageUpdate {
    type: "DeletePage";
    ownerDsnpUserId: number;
    schemaId: number;
    pageId: number;
    prevHash: Uint8Array;
}
  
export interface AddKeyUpdate {
    type: "AddKey";
    ownerDsnpUserId: number;
    prevHash: Uint8Array;
    payload: Uint8Array;
}

export type Update = PersistPageUpdate | DeletePageUpdate | AddKeyUpdate;
