import { Connection } from "./connection";
import { DsnpKeys } from "./import_bundle";

export interface ActionOptions {
  ignoreExistingConnections?: boolean;
  ignoreMissingConnections?: boolean;
}

export interface ConnectAction {
  type: "Connect";
  ownerDsnpUserId: string;
  connection: Connection;
  dsnpKeys?: DsnpKeys;
}

export interface DisconnectAction {
  type: "Disconnect";
  ownerDsnpUserId: string;
  connection: Connection;
}

export interface AddGraphKeyAction {
  type: "AddGraphKey";
  ownerDsnpUserId: string;
  newPublicKey: Uint8Array;
}

export type Action = ConnectAction | DisconnectAction | AddGraphKeyAction;
