export interface PersistPageUpdate {
    type: "PersistPage";
    ownerDsnpUserId: string;
    schemaId: number;
    pageId: number;
    prevHash: number;
    payload: Uint8Array;
}
  
export interface DeletePageUpdate {
    type: "DeletePage";
    ownerDsnpUserId: string;
    schemaId: number;
    pageId: number;
    prevHash: number;
}
  
export interface AddKeyUpdate {
    type: "AddKey";
    ownerDsnpUserId: string;
    prevHash: number;
    payload: Uint8Array;
}

export type Update = PersistPageUpdate | DeletePageUpdate | AddKeyUpdate;
