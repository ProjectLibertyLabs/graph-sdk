export interface PersistPageUpdate {
    type: "PersistPage";
    ownerDsnpUserId: number;
    schemaId: number;
    pageId: number;
    prevHash: number;
    payload: Uint8Array;
}
  
export interface DeletePageUpdate {
    type: "DeletePage";
    ownerDsnpUserId: number;
    schemaId: number;
    pageId: number;
    prevHash: number;
}
  
export interface AddKeyUpdate {
    type: "AddKey";
    ownerDsnpUserId: number;
    prevHash: number;
    payload: Uint8Array;
}

export type Update = PersistPageUpdate | DeletePageUpdate | AddKeyUpdate;
