# Rust Examples

### Create and export a new graph
```rust
use dsnp_graph_config::{ConnectionType, Environment, PrivacyType};
use dsnp_graph_core::api::{
    api::{GraphAPI, GraphState},
    api_types::{Action, Connection},
};

fn main() {
    // create graph state with chosen environment
    let mut state = GraphState::new(Environment::Mainnet);

    // graph owner that we want to interact with
    let my_dsnp_user_id = 1000;

    // get that desired graph schema id
    let public_follow_graph_schema_id = Environment::Mainnet
        .get_config()
        .get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Public))
        .unwrap();

    // add a new connection to the owner graph
    let add_connection = Action::Connect {
        connection: Connection { dsnp_user_id: 2000, schema_id: public_follow_graph_schema_id },
        dsnp_keys: None,
        owner_dsnp_user_id: my_dsnp_user_id,
    };

    // add new connection to graph
    let apply_result = state.apply_actions(&vec![add_connection]);
    if let Err(e) = apply_result {
        println!("{:?}", e);
        return
    }

    // get all connections including pending one that we just added
    let connections = state
        .get_connections_for_user_graph(&my_dsnp_user_id, &public_follow_graph_schema_id, true)
        .unwrap();
    println!("Connections {:?}", connections);
    let connections_user_ids: Vec<_> = connections.iter().map(|c| c.user_id).collect();
    assert!(connections_user_ids.contains(&2000));

    // export all updates to the graph
    match state.export_updates() {
        Ok(updates) => {
            println!("Updates {:?}", updates);
            // Update Blockchain using the updates
        },
        Err(e) => {
            println!("{:?}", e);
        },
    }
}
```

### Add a new Graph Key
```rust
use dsnp_graph_config::Environment;
use dsnp_graph_core::api::{
	api::{GraphAPI, GraphState},
	api_types::{Action, Update},
};

fn main() {
	// create graph state with chosen environment
	let mut state = GraphState::new(Environment::Mainnet);

	// graph key owner
	let dsnp_key_owner = 1000;

	// new_key
	let x25519_public_key = vec![
		15u8, 234, 44, 175, 171, 220, 131, 117, 43, 227, 111, 165, 52, 150, 64, 218, 44, 130, 138,
		221, 10, 41, 13, 241, 60, 210, 216, 23, 62, 178, 73, 111,
	];
	let new_key_action = Action::AddGraphKey {
		owner_dsnp_user_id: dsnp_key_owner,
		new_public_key: x25519_public_key,
	};

	// add new key
	let key_result = state.apply_actions(&[new_key_action]);
	if let Err(e) = key_result {
		println!("{:?}", e);
		return
	}

	// export newly add key to publish on chain
	match state.export_updates() {
		Ok(updates) => {
			println!("Updates {:?}", updates);
			let _add_key_updates: Vec<_> =
				updates.iter().filter(|u| matches!(u, Update::AddKey { .. })).collect();
			// publish added key
		},
		Err(e) => {
			println!("{:?}", e);
		},
	}
}
```
### Read and deserialize published graph keys
```rust
use dsnp_graph_core::api::{
	api::{GraphAPI, GraphState},
	api_types::{DsnpKeys, KeyData},
};

fn main() {
	// graph key owner
	let dsnp_key_owner = 1000;

	// published keys blobs fetched from blockchain
	let published_keys_blob = vec![
		64, 15, 234, 44, 175, 171, 220, 131, 117, 43, 227, 111, 165, 52, 150, 64, 218, 44, 130,
		138, 221, 10, 41, 13, 241, 60, 210, 216, 23, 62, 178, 73, 111,
	];
	let dsnp_keys = DsnpKeys {
		keys: vec![KeyData { content: published_keys_blob, index: 0 }],
		dsnp_user_id: dsnp_key_owner,
		keys_hash: 2789, // should get this hash value from blockchain
	};

	// deserialize published keys
	match GraphState::deserialize_dsnp_keys(&dsnp_keys) {
		Ok(keys) => {
			println!("Keys {:?}", keys);
		},
		Err(e) => {
			println!("{:?}", e);
		},
	}
}

```

### Update a Private Follow graph
```rust
use dsnp_graph_config::{ConnectionType, Environment, PrivacyType};
use dsnp_graph_core::api::{
	api::{GraphAPI, GraphState},
	api_types::{Action, Connection, DsnpKeys, ImportBundle},
};

fn main() {
	// create graph state with chosen environment
	let mut state = GraphState::new(Environment::Mainnet);

	// graph owner that we want to interact with
	let my_dsnp_user_id = 1000;

	// get that desired graph schema id
	let private_follow_graph_schema_id = Environment::Mainnet
		.get_config()
		.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
		.unwrap();

	// import existing published graph and keys fetched from blockchain
	let import_bundle = ImportBundle {
		dsnp_keys: DsnpKeys {
			keys_hash: 123, // get from blockchain
			keys: vec![/* published keys got from blockchain */],
			dsnp_user_id: my_dsnp_user_id,
		},
		dsnp_user_id: my_dsnp_user_id,
		schema_id: private_follow_graph_schema_id,
		pages: vec![/* published graph pages got from blockchain */],
		key_pairs: vec![ /* get key-pairs associated with the my_dsnp_user_id user from wallet */],
	};

	if let Err(e) = state.import_users_data(&vec![import_bundle]) {
		println!("{:?}", e);
		return
	}

	// add a new connection to the owner graph
	let add_connection = Action::Connect {
		connection: Connection { dsnp_user_id: 3000, schema_id: private_follow_graph_schema_id },
		owner_dsnp_user_id: my_dsnp_user_id,
		dsnp_keys: None,
	};

	// add new connection to graph
	let apply_result = state.apply_actions(&vec![add_connection]);
	if let Err(e) = apply_result {
		println!("{:?}", e);
		return
	}

	// export all updates to the graph
	match state.export_updates() {
		Ok(updates) => {
			println!("Updates {:?}", updates);
			// Update Blockchain using the updates
		},
		Err(e) => {
			println!("{:?}", e);
		},
	}
}
```

### Update a Private Friendship graph
```rust
use dsnp_graph_config::{ConnectionType, Environment, PrivacyType};
use dsnp_graph_core::api::{
	api::{GraphAPI, GraphState},
	api_types::{Action, Connection, DsnpKeys, ImportBundle},
};

fn main() {
	// create graph state with chosen environment
	let mut state = GraphState::new(Environment::Mainnet);

	// graph owner that we want to interact with
	let my_dsnp_user_id = 1000;

	// get that desired graph schema id
	let private_friendship_graph_schema_id = Environment::Mainnet
		.get_config()
		.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Private))
		.unwrap();

	// import existing published graph and keys fetched from blockchain
	let import_bundle = ImportBundle {
		dsnp_keys: DsnpKeys {
			keys_hash: 123, // get from blockchain
			keys: vec![/* published keys got from blockchain */],
			dsnp_user_id: my_dsnp_user_id,
		},
		dsnp_user_id: my_dsnp_user_id,
		schema_id: private_friendship_graph_schema_id,
		pages: vec![/* published graph pages got from blockchain */],
		key_pairs: vec![ /* get key-pairs associated with the my_dsnp_user_id user from wallet */],
	};

	if let Err(e) = state.import_users_data(&vec![import_bundle]) {
		println!("{:?}", e);
		return
	}

	// get all associated user without keys so we can fetch and import keys for them
	let user_without_keys = state.get_connections_without_keys().unwrap();
	let mut users_import_bundles: Vec<ImportBundle> = vec![];
	for _user in user_without_keys {
		// let user_dsnp_keys = DsnpKeys {..}  // fetch published DsnpKeys for user
		// let user_pages = .. // fetch published private friendship pages for the user
		// let user_import_bundle = {
		// 	dsnp_keys: user_dsnp_keys,
		// 	dsnp_user_id: user,
		// 	schema_id: private_friendship_graph_schema_id,
		// 	pages: user_pages,
		// 	key_pairs: vec![], // empty key pairs for user since we don't know and need their secret key
		// }
		// users_import_bundle.push(user_import_bundle);
	}

	// import these fetched keys and pages into state
	if let Err(e) = state.import_users_data(&users_import_bundles) {
		println!("{:?}", e);
		return
	}

	// add a new connection to the owner graph
	let add_connection = Action::Connect {
		connection: Connection { dsnp_user_id: 400, schema_id: private_friendship_graph_schema_id },
		owner_dsnp_user_id: my_dsnp_user_id,
		dsnp_keys: Some(DsnpKeys {
			dsnp_user_id: 400,
			keys_hash: 2982, // get keys hash from chain
			keys: vec![/* get keys from chain for user 400 */],
		}),
	};

	// add new connection to graph
	let apply_result = state.apply_actions(&vec![add_connection]);
	if let Err(e) = apply_result {
		println!("{:?}", e);
		return
	}

	// export all updates to the graph
	match state.export_updates() {
		Ok(updates) => {
			println!("Updates {:?}", updates);
			// Update Blockchain using the updates
		},
		Err(e) => {
			println!("{:?}", e);
		},
	}
}

```
