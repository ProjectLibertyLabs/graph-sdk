import { Connection } from "./connection";
import { DsnpKeys } from "./import_bundle";

export interface ConnectAction {
    type: "Connect";
    owner_dsnp_user_id: string;
    connection: Connection;
    dsnp_keys?: DsnpKeys;
}
  
export interface DisconnectAction {
  type: "Disconnect";
  owner_dsnp_user_id: string;
  connection: Connection;
}

export interface AddGraphKeyAction {
  type: "AddGraphKey";
  owner_dsnp_user_id: string;
  new_public_key: Uint8Array;
}

export type Action = ConnectAction | DisconnectAction | AddGraphKeyAction;
  