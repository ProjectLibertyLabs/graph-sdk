import { Connection } from "./connection";
import { DsnpKeys } from "./import_bundle";

export interface ConnectAction {
    type: "Connect";
    ownerDsnpUserId: number;
    connection: Connection;
    dsnpKeys: DsnpKeys;
}
  
export interface DisconnectAction {
  type: "Disconnect";
  ownerDsnpUserId: number;
  connection: Connection;
}

export interface AddGraphKeyAction {
  type: "AddGraphKey";
  ownerDsnpUserId: number;
  newPublicKey: Uint8Array;
}

export type Action = ConnectAction | DisconnectAction | AddGraphKeyAction;
  