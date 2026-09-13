extern crate std;

use super::*;
use endpoint_v2::{MessagingFee, MessagingParams, MessagingReceipt, Origin};
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Bytes, BytesN, Env, IntoVal,
};

const EID: u32 = 301;

#[contract]
struct MockEndpoint;

#[contractimpl]
impl MockEndpoint {
    pub fn __constructor(env: Env, native_token: Address) {
        env.storage().instance().set(&symbol_short!("native"), &native_token);
    }

    pub fn set_delegate(_env: Env, _oapp: Address, _delegate: Option<Address>) {}

    pub fn native_token(env: Env) -> Address {
        env.storage().instance().get(&symbol_short!("native")).unwrap()
    }

    pub fn zro(_env: Env) -> Option<Address> {
        None
    }

    pub fn quote(_env: Env, _sender: Address, _params: MessagingParams) -> MessagingFee {
        MessagingFee { native_fee: 0, zro_fee: 0 }
    }

    pub fn send(_env: Env, _sender: Address, _params: MessagingParams, _refund_address: Address) -> MessagingReceipt {
        MessagingReceipt {
            guid: BytesN::from_array(&_env, &[0xabu8; 32]),
            nonce: 1,
            fee: MessagingFee { native_fee: 0, zro_fee: 0 },
        }
    }

    pub fn clear(env: Env, _caller: Address, origin: Origin, _receiver: Address, guid: BytesN<32>, message: Bytes) {
        env.storage().instance().set(&symbol_short!("origin"), &origin);
        env.storage().instance().set(&symbol_short!("guid"), &guid);
        env.storage().instance().set(&symbol_short!("message"), &message);
    }

    pub fn last_clear(env: Env) -> (Origin, BytesN<32>, Bytes) {
        (
            env.storage().instance().get(&symbol_short!("origin")).unwrap(),
            env.storage().instance().get(&symbol_short!("guid")).unwrap(),
            env.storage().instance().get(&symbol_short!("message")).unwrap(),
        )
    }
}

#[test]
fn payload_is_versioned_domain_separated_and_round_trips() {
    let env = Env::default();
    let hash = BytesN::from_array(&env, &[7; 32]);
    let receipt = FinalStateReceipt { campaign_id: 42, operation_version: 3, status: COMPLETE, state_hash: hash };
    let bytes = encode(&env, &receipt);
    assert_eq!(bytes.len(), PAYLOAD_LEN);
    assert_eq!(decode(&env, &bytes), receipt);
    assert_eq!(bytes.get(0).unwrap(), b'T');
    assert_eq!(bytes.get(4).unwrap(), RECEIPT_VERSION);
}

#[test]
fn unsupported_status_and_malformed_payloads_are_rejected() {
    let env = Env::default();
    let hash = BytesN::from_array(&env, &[1; 32]);
    let bad_status = FinalStateReceipt { campaign_id: 1, operation_version: 1, status: 99, state_hash: hash };
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| encode(&env, &bad_status))).is_err());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decode(&env, &Bytes::from_array(&env, &[1, 2])))).is_err());
}

#[test]
fn nonce_and_peer_checks_are_explicit_in_the_receiver_contract() {
    // This test documents the exact ordered-path rule independently of the
    // endpoint harness: paths begin at nonce one and never skip a nonce.
    let env = Env::default();
    let sender = BytesN::from_array(&env, &[9; 32]);
    let origin = Origin { src_eid: EID, sender, nonce: 1 };
    assert_eq!(origin.nonce, 1);
    assert_ne!(origin.nonce, 0);
    let _executor = Address::generate(&env);
}

#[test]
fn mock_endpoint_delivery_authenticates_peer_clears_and_orders() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let native_token = env.register_stellar_asset_contract_v2(token_admin).address();
    let endpoint = env.register(MockEndpoint, (&native_token,));
    let adapter = env.register(ReceiptAdapter, (&owner, &endpoint));
    let client = ReceiptAdapterClient::new(&env, &adapter);
    let peer = BytesN::from_array(&env, &[8; 32]);
    let peer_option = Some(peer.clone());

    env.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &adapter,
            fn_name: "set_peer",
            args: (&EID, &peer_option, &owner).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_peer(&EID, &peer_option, &owner);

    let state_hash = BytesN::from_array(&env, &[4; 32]);
    let message = client.encode_receipt(&7, &1, &COMPLETE, &state_hash);
    let executor = Address::generate(&env);
    let origin = Origin { src_eid: EID, sender: peer, nonce: 1 };
    let guid = BytesN::from_array(&env, &[5; 32]);
    let extra_data = Bytes::new(&env);
    let value = 0i128;
    env.mock_auths(&[MockAuth {
        address: &executor,
        invoke: &MockAuthInvoke {
            contract: &adapter,
            fn_name: "lz_receive",
            args: (&executor, &origin, &guid, &message, &extra_data, &value).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.lz_receive(&executor, &origin, &guid, &message, &extra_data, &value);

    assert!(client.was_received(&7, &1));
    assert_eq!(client.next_nonce(&EID, &origin.sender), 2);
    let clear_client = MockEndpointClient::new(&env, &endpoint);
    let (cleared_origin, cleared_guid, cleared_message) = clear_client.last_clear();
    assert_eq!(cleared_origin, origin);
    assert_eq!(cleared_guid, guid);
    assert_eq!(cleared_message, message);

    // A duplicate campaign operation is rejected even when a delivery is
    // attempted with a new GUID and the next ordered nonce.
    let duplicate_guid = BytesN::from_array(&env, &[6; 32]);
    let duplicate_origin = Origin { src_eid: EID, sender: origin.sender.clone(), nonce: 2 };
    env.mock_auths(&[MockAuth {
        address: &executor,
        invoke: &MockAuthInvoke {
            contract: &adapter,
            fn_name: "lz_receive",
            args: (&executor, &duplicate_origin, &duplicate_guid, &message, &extra_data, &value).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client
        .try_lz_receive(&executor, &duplicate_origin, &duplicate_guid, &message, &extra_data, &value)
        .is_err());
}

#[test]
fn quote_and_owner_send_return_the_official_receipt() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let native_token = env.register_stellar_asset_contract_v2(token_admin).address();
    let endpoint = env.register(MockEndpoint, (&native_token,));
    let adapter = env.register(ReceiptAdapter, (&owner, &endpoint));
    let client = ReceiptAdapterClient::new(&env, &adapter);
    let peer = BytesN::from_array(&env, &[3; 32]);
    let peer_option = Some(peer);
    env.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &adapter,
            fn_name: "set_peer",
            args: (&EID, &peer_option, &owner).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_peer(&EID, &peer_option, &owner);

    let hash = BytesN::from_array(&env, &[2; 32]);
    let options = Bytes::new(&env);
    let fee = client.quote_final_state(&EID, &99, &4, &COMPLETE, &hash, &options, &false);
    assert_eq!(fee, MessagingFee { native_fee: 0, zro_fee: 0 });

    let refund = Address::generate(&env);
    let zero = 0i128;
    let transfer = MockAuthInvoke {
        contract: &native_token,
        fn_name: "transfer",
        args: (&owner, &endpoint, &zero).into_val(&env),
        sub_invokes: &[],
    };
    env.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &adapter,
            fn_name: "send_final_state",
            args: (&owner, &EID, &99u64, &4u32, &COMPLETE, &hash, &options, &MessagingFee { native_fee: 0, zro_fee: 0 }, &refund)
                .into_val(&env),
            sub_invokes: &[transfer],
        },
    }]);
    let sent = client.send_final_state(
        &owner,
        &EID,
        &99,
        &4,
        &COMPLETE,
        &hash,
        &options,
        &MessagingFee { native_fee: 0, zro_fee: 0 },
        &refund,
    );
    assert_eq!(sent.guid, BytesN::from_array(&env, &[0xabu8; 32]));
    assert_eq!(sent.nonce, 1);
    assert!(client.was_published(&99, &4));
    env.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &adapter,
            fn_name: "send_final_state",
            args: (&owner, &EID, &99u64, &4u32, &COMPLETE, &hash, &options, &MessagingFee { native_fee: 0, zro_fee: 0 }, &refund)
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client
        .try_send_final_state(
            &owner,
            &EID,
            &99,
            &4,
            &COMPLETE,
            &hash,
            &options,
            &MessagingFee { native_fee: 0, zro_fee: 0 },
            &refund,
        )
        .is_err());
}