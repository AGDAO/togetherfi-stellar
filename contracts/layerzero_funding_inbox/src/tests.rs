extern crate std;

use super::*;
use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Ledger,
    },
    token::{Client as TokenClient, StellarAssetClient},
    Address, Bytes, BytesN, Env,
};

#[contract]
struct MockEndpoint;

#[contractimpl]
impl MockEndpoint {
    pub fn queue(
        env: Env,
        from: Address,
        to: Address,
        guid: BytesN<32>,
        index: u32,
        message: Bytes,
    ) {
        env.storage()
            .instance()
            .set(&symbol_short!("queued"), &(from, to, guid, index, message));
    }

    pub fn clear_compose(
        env: Env,
        composer: Address,
        from: Address,
        guid: BytesN<32>,
        index: u32,
        message: Bytes,
    ) {
        composer.require_auth();
        let expected: (Address, Address, BytesN<32>, u32, Bytes) = env
            .storage()
            .instance()
            .get(&symbol_short!("queued"))
            .unwrap();
        assert_eq!(expected.0, from);
        assert_eq!(expected.1, composer);
        assert_eq!(expected.2, guid);
        assert_eq!(expected.3, index);
        assert_eq!(expected.4, message);
        env.storage()
            .instance()
            .set(&symbol_short!("cleared"), &true);
    }

    pub fn was_cleared(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&symbol_short!("cleared"))
            .unwrap_or(false)
    }
}

fn setup(
    env: &Env,
) -> (
    FundingInboxClient<'_>,
    Address,
    Address,
    Address,
    BytesN<32>,
    BytesN<32>,
    BytesN<32>,
) {
    let endpoint = env.register(MockEndpoint, ());
    let oft = Address::generate(env);
    let admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let peer = BytesN::from_array(env, &[7; 32]);
    let asset_domain = BytesN::from_array(env, &[8; 32]);
    let escrow_domain = BytesN::from_array(env, &[9; 32]);
    let inbox = env.register(
        FundingInbox,
        (
            &admin,
            &endpoint,
            &oft,
            &token,
            &301u32,
            &peer,
            &asset_domain,
            &escrow_domain,
        ),
    );
    (
        FundingInboxClient::new(env, &inbox),
        inbox,
        endpoint,
        oft,
        peer,
        asset_domain,
        escrow_domain,
    )
}

fn instruction(
    env: &Env,
    sponsor: &Address,
    asset_domain: &BytesN<32>,
    escrow_domain: &BytesN<32>,
    campaign_id: u64,
) -> FundingInstruction {
    FundingInstruction {
        campaign_id,
        operation_version: 1,
        sponsor: sponsor.clone(),
        sponsor_reference: Bytes::from_slice(env, b"source-sponsor"),
        minimum_destination_amount: 90,
        deadline: 100,
        asset_domain: asset_domain.clone(),
        escrow_asset_domain: escrow_domain.clone(),
    }
}

fn wrapped(
    _env: &Env,
    client: &FundingInboxClient,
    instruction: &FundingInstruction,
    peer: &BytesN<32>,
    nonce: u64,
    amount: i128,
) -> Bytes {
    let custom = client.encode_instruction(instruction);
    client.encode_oft_compose_message(&nonce, &301, &amount, peer, &custom)
}

fn queue(
    env: &Env,
    endpoint: &Address,
    oft: &Address,
    inbox: &Address,
    guid: &BytesN<32>,
    message: &Bytes,
) {
    MockEndpointClient::new(env, endpoint).queue(oft, inbox, guid, &0, message);
}

#[test]
fn official_oft_envelope_records_authenticated_amount_and_clears_full_message() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    // The fixture balance is sufficient; delivery provenance comes from the
    // configured OFT/envelope/queue trust root, not the fungible balance.
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    let intent = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    // OFT nonces are path-scoped, so the first message for this inbox need
    // not have nonce one.
    let message = wrapped(&env, &client, &intent, &peer, 17, 100);
    let guid = BytesN::from_array(&env, &[3; 32]);
    queue(&env, &endpoint, &oft, &inbox, &guid, &message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &guid,
        &0,
        &message,
        &Bytes::new(&env),
        &0,
    );
    assert_eq!(client.held_balance(), 100);
    assert_eq!(client.get_intent(&42, &1).source_nonce, 17);
    assert_eq!(client.get_intent(&42, &1).received_amount, 100);
    assert_eq!(
        TokenClient::new(&env, &client.configured_token()).balance(&inbox),
        100
    );
    assert!(MockEndpointClient::new(&env, &endpoint).was_cleared());
}

#[test]
fn sponsor_claim_releases_held_balance() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    let intent = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let message = wrapped(&env, &client, &intent, &peer, 1, 100);
    let guid = BytesN::from_array(&env, &[0x21; 32]);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    queue(&env, &endpoint, &oft, &inbox, &guid, &message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &guid,
        &0,
        &message,
        &Bytes::new(&env),
        &0,
    );
    client.claim(&42, &1, &sponsor, &intent.sponsor_reference);
    assert_eq!(
        TokenClient::new(&env, &client.configured_token()).balance(&inbox),
        0
    );
}

#[test]
fn ambient_balance_cannot_be_allocated_by_a_non_oft_compose() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, _oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    let intent = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let message = wrapped(&env, &client, &intent, &peer, 1, 100);
    let guid = BytesN::from_array(&env, &[0x31; 32]);
    // Ambient configured-token balance is not provenance. Even with enough
    // balance, a compose from any non-configured OFT must fail.
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    let arbitrary_oft = Address::generate(&env);
    MockEndpointClient::new(&env, &endpoint).queue(&arbitrary_oft, &inbox, &guid, &0, &message);
    assert!(client
        .try_lz_compose(
            &Address::generate(&env),
            &arbitrary_oft,
            &guid,
            &0,
            &message,
            &Bytes::new(&env),
            &0,
        )
        .is_err());
    assert_eq!(client.held_balance(), 0);
}

#[test]
fn unauthorized_or_unqueued_compose_cannot_credit() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, _endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    let intent = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let message = wrapped(&env, &client, &intent, &peer, 1, 100);
    let guid = BytesN::from_array(&env, &[4; 32]);

    env.set_auths(&[]);
    assert!(client
        .try_lz_compose(
            &Address::generate(&env),
            &oft,
            &guid,
            &0,
            &message,
            &Bytes::new(&env),
            &0,
        )
        .is_err());
    env.mock_all_auths_allowing_non_root_auth();
    assert!(client
        .try_lz_compose(
            &Address::generate(&env),
            &oft,
            &guid,
            &0,
            &message,
            &Bytes::new(&env),
            &0,
        )
        .is_err());
    assert_eq!(client.held_balance(), 0);
}

#[test]
fn envelope_route_nonce_and_amount_must_match() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    let intent = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let guid = BytesN::from_array(&env, &[5; 32]);
    // The configured token balance is 99, while the authenticated amount is 100.
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &99);
    let message = wrapped(&env, &client, &intent, &peer, 1, 100);
    queue(&env, &endpoint, &oft, &inbox, &guid, &message);
    assert!(client
        .try_lz_compose(
            &Address::generate(&env),
            &oft,
            &guid,
            &0,
            &message,
            &Bytes::new(&env),
            &0,
        )
        .is_err());
    assert_eq!(client.held_balance(), 0);

    for (index, amount) in [(6u8, 0i128), (7u8, -1i128), (8u8, i128::MAX)] {
        let boundary_guid = BytesN::from_array(&env, &[index; 32]);
        let boundary_message = wrapped(&env, &client, &intent, &peer, 1, amount);
        queue(
            &env,
            &endpoint,
            &oft,
            &inbox,
            &boundary_guid,
            &boundary_message,
        );
        assert!(client
            .try_lz_compose(
                &Address::generate(&env),
                &oft,
                &boundary_guid,
                &0,
                &boundary_message,
                &Bytes::new(&env),
                &0,
            )
            .is_err());
    }
}

#[test]
fn sequential_oft_credits_track_each_authenticated_oft_amount() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    let first = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let second = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 43);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    let first_message = wrapped(&env, &client, &first, &peer, 1, 100);
    let second_message = wrapped(&env, &client, &second, &peer, 2, 200);
    let first_guid = BytesN::from_array(&env, &[0x11; 32]);
    let second_guid = BytesN::from_array(&env, &[0x12; 32]);
    queue(&env, &endpoint, &oft, &inbox, &first_guid, &first_message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &first_guid,
        &0,
        &first_message,
        &Bytes::new(&env),
        &0,
    );
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &200);
    queue(&env, &endpoint, &oft, &inbox, &second_guid, &second_message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &second_guid,
        &0,
        &second_message,
        &Bytes::new(&env),
        &0,
    );
    assert_eq!(client.held_balance(), 300);
    assert_eq!(client.get_intent(&42, &1).received_amount, 100);
    assert_eq!(client.get_intent(&43, &1).received_amount, 200);
}

#[test]
fn instance_and_persistent_entries_receive_safe_ttl() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    let intent = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let message = wrapped(&env, &client, &intent, &peer, 9, 100);
    let guid = BytesN::from_array(&env, &[13; 32]);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    queue(&env, &endpoint, &oft, &inbox, &guid, &message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &guid,
        &0,
        &message,
        &Bytes::new(&env),
        &0,
    );

    let (instance_ttl, intent_ttl, message_ttl) = env.as_contract(&inbox, || {
        (
            env.storage().instance().get_ttl(),
            env.storage().persistent().get_ttl(&DataKey::Intent(42, 1)),
            env.storage()
                .persistent()
                .get_ttl(&DataKey::Message(guid.clone(), 0)),
        )
    });
    assert!(instance_ttl >= TTL);
    assert!(intent_ttl >= TTL);
    assert!(message_ttl >= TTL);

    // Reading a live intent is also a keeper touch for the persistent entry.
    env.ledger().set_sequence_number(100);
    let before = env.as_contract(&inbox, || {
        env.storage().persistent().get_ttl(&DataKey::Intent(42, 1))
    });
    client.get_intent(&42, &1);
    let after = env.as_contract(&inbox, || {
        env.storage().persistent().get_ttl(&DataKey::Intent(42, 1))
    });
    assert!(after >= before);
}

#[test]
fn duplicate_guid_and_operation_rejected_but_nonce_gaps_and_reordering_work() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    let intent = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let message = wrapped(&env, &client, &intent, &peer, 1, 100);
    let guid = BytesN::from_array(&env, &[6; 32]);
    queue(&env, &endpoint, &oft, &inbox, &guid, &message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &guid,
        &0,
        &message,
        &Bytes::new(&env),
        &0,
    );
    queue(&env, &endpoint, &oft, &inbox, &guid, &message);
    assert!(client
        .try_lz_compose(
            &Address::generate(&env),
            &oft,
            &guid,
            &0,
            &message,
            &Bytes::new(&env),
            &0,
        )
        .is_err());

    let next_intent = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 43);
    let next_message = wrapped(&env, &client, &next_intent, &peer, 3, 100);
    let next_guid = BytesN::from_array(&env, &[7; 32]);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    queue(&env, &endpoint, &oft, &inbox, &next_guid, &next_message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &next_guid,
        &0,
        &next_message,
        &Bytes::new(&env),
        &0,
    );

    // A later compose can carry an older path nonce, and a gap is valid.
    let out_of_order = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 44);
    let out_of_order_message = wrapped(&env, &client, &out_of_order, &peer, 2, 100);
    let out_of_order_guid = BytesN::from_array(&env, &[10; 32]);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    queue(
        &env,
        &endpoint,
        &oft,
        &inbox,
        &out_of_order_guid,
        &out_of_order_message,
    );
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &out_of_order_guid,
        &0,
        &out_of_order_message,
        &Bytes::new(&env),
        &0,
    );
    assert_eq!(client.held_balance(), 300);
    assert_eq!(client.get_intent(&43, &1).source_nonce, 3);
    assert_eq!(client.get_intent(&44, &1).source_nonce, 2);
}

#[test]
fn sponsor_cancel_and_expired_recovery_release_only_pending_liabilities() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    let token = TokenClient::new(&env, &client.configured_token());
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    let first = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let first_message = wrapped(&env, &client, &first, &peer, 1, 100);
    let first_guid = BytesN::from_array(&env, &[8; 32]);
    queue(&env, &endpoint, &oft, &inbox, &first_guid, &first_message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &first_guid,
        &0,
        &first_message,
        &Bytes::new(&env),
        &0,
    );
    client.cancel(&42, &1, &sponsor, &first.sponsor_reference);
    assert_eq!(token.balance(&sponsor), 100);

    let second = FundingInstruction {
        campaign_id: 43,
        deadline: 1,
        ..first
    };
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    let second_message = wrapped(&env, &client, &second, &peer, 2, 100);
    let second_guid = BytesN::from_array(&env, &[9; 32]);
    queue(&env, &endpoint, &oft, &inbox, &second_guid, &second_message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &second_guid,
        &0,
        &second_message,
        &Bytes::new(&env),
        &0,
    );
    env.ledger().set_timestamp(2);
    client.release_expired(&43, &1);
    assert_eq!(token.balance(&sponsor), 200);
}

#[test]
fn pause_blocks_new_liabilities_and_claims_but_not_recovery() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (client, inbox, endpoint, oft, peer, asset_domain, escrow_domain) = setup(&env);
    let sponsor = Address::generate(&env);
    let first = instruction(&env, &sponsor, &asset_domain, &escrow_domain, 42);
    let first_message = wrapped(&env, &client, &first, &peer, 1, 100);
    let first_guid = BytesN::from_array(&env, &[0x41; 32]);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    queue(&env, &endpoint, &oft, &inbox, &first_guid, &first_message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &first_guid,
        &0,
        &first_message,
        &Bytes::new(&env),
        &0,
    );

    env.set_auths(&[]);
    assert!(client.try_pause().is_err());
    env.mock_all_auths_allowing_non_root_auth();
    client.pause();
    assert!(client.paused());
    // Even a properly queued compose cannot create another liability while
    // paused (the pause gate runs before replay/accounting checks).
    queue(&env, &endpoint, &oft, &inbox, &first_guid, &first_message);
    assert!(client
        .try_lz_compose(
            &Address::generate(&env),
            &oft,
            &first_guid,
            &0,
            &first_message,
            &Bytes::new(&env),
            &0,
        )
        .is_err());
    assert!(client
        .try_claim(&42, &1, &sponsor, &first.sponsor_reference)
        .is_err());
    // A sponsor can still cancel a pending liability during the pause.
    client.cancel(&42, &1, &sponsor, &first.sponsor_reference);

    // A liability created before the pause remains recoverable after expiry.
    let second = FundingInstruction {
        campaign_id: 43,
        deadline: 1,
        ..first
    };
    let second_message = wrapped(&env, &client, &second, &peer, 2, 100);
    let second_guid = BytesN::from_array(&env, &[0x42; 32]);
    StellarAssetClient::new(&env, &client.configured_token()).mint(&inbox, &100);
    client.unpause();
    queue(&env, &endpoint, &oft, &inbox, &second_guid, &second_message);
    client.lz_compose(
        &Address::generate(&env),
        &oft,
        &second_guid,
        &0,
        &second_message,
        &Bytes::new(&env),
        &0,
    );
    client.pause();
    env.ledger().set_timestamp(2);
    client.release_expired(&43, &1);
}
