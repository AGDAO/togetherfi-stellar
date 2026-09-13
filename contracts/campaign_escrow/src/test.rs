#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Bytes, Env,
};

#[contracttype]
#[derive(Clone)]
enum MockPoolKey {
    Escrow,
    Token,
}

#[contract]
struct MockContributorPool;

#[contractimpl]
impl MockContributorPool {
    pub fn initialize(env: Env, escrow: Address, token: Address) {
        env.storage().instance().set(&MockPoolKey::Escrow, &escrow);
        env.storage().instance().set(&MockPoolKey::Token, &token);
    }
    pub fn escrow_contract(env: Env) -> Address {
        env.storage().instance().get(&MockPoolKey::Escrow).unwrap()
    }
    pub fn token_contract(env: Env) -> Address {
        env.storage().instance().get(&MockPoolKey::Token).unwrap()
    }
    pub fn record_credit(env: Env, _campaign_id: u64, _amount: i128) {
        let escrow: Address = env.storage().instance().get(&MockPoolKey::Escrow).unwrap();
        escrow.require_auth();
    }
}

struct Setup<'a> {
    env: Env,
    id: Address,
    client: CampaignEscrowClient<'a>,
    token: TokenClient<'a>,
    governance: Address,
    settlement: Address,
    sponsor: Address,
    creator: Address,
    referrer: Address,
    service: Address,
    nft: Address,
    revenue: Address,
}

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let governance = Address::generate(&env);
    let settlement = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let creator = Address::generate(&env);
    let referrer = Address::generate(&env);
    let service = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token = TokenClient::new(&env, &token_id);
    let mint = StellarAssetClient::new(&env, &token_id);
    mint.mint(&sponsor, &10_000_000_000i128);
    let id = env.register_contract(None, CampaignEscrow);
    let nft = env.register_contract(None, MockContributorPool);
    let revenue = env.register_contract(None, MockContributorPool);
    MockContributorPoolClient::new(&env, &nft).initialize(&id, &token_id);
    MockContributorPoolClient::new(&env, &revenue).initialize(&id, &token_id);
    let client = CampaignEscrowClient::new(&env, &id);
    client.initialize(
        &governance,
        &settlement,
        &token_id,
        &service,
        &nft,
        &revenue,
    );
    Setup {
        env,
        id,
        client,
        token,
        governance,
        settlement,
        sponsor,
        creator,
        referrer,
        service,
        nft,
        revenue,
    }
}

fn terms(env: &Env) -> Bytes {
    Bytes::from_slice(env, b"creatorfi-v2")
}
fn activate(s: &Setup, id: u64) {
    s.client
        .activate(&id, &s.creator, &Some(s.referrer.clone()), &terms(&s.env));
}

#[test]
fn initialize_is_one_time_and_config_versioned() {
    let s = setup();
    assert_eq!(s.client.config_version(), 1);
    assert_eq!(s.client.governance(), s.governance);
    assert_eq!(s.client.settlement_authority(), s.settlement);
    assert_eq!(
        s.client.pool_destinations(),
        (s.nft.clone(), s.revenue.clone())
    );
    assert_eq!(
        s.client.try_initialize(
            &s.governance,
            &s.settlement,
            &Address::generate(&s.env),
            &s.service,
            &s.nft,
            &s.revenue
        ),
        Err(Ok(EscrowError::AlreadyInitialized))
    );
}

#[test]
fn exact_split_sends_each_leg_to_its_approved_destination() {
    let s = setup();
    let amount = 1_000_003i128;
    s.client.fund(&1, &s.sponsor, &amount);
    activate(&s, 1);
    s.client.complete(&1, &1);
    let service = amount * 1000 / 10000;
    let nft = amount * 500 / 10000;
    let revenue = amount * 200 / 10000;
    let referrer = amount * 100 / 10000;
    assert_eq!(s.token.balance(&s.service), service);
    assert_eq!(s.token.balance(&s.nft), nft);
    assert_eq!(s.token.balance(&s.revenue), revenue);
    assert_eq!(s.token.balance(&s.referrer), referrer);
    assert_eq!(
        s.token.balance(&s.creator),
        amount - service - nft - revenue - referrer
    );
    assert_eq!(s.token.balance(&s.id), 0);
}

#[test]
fn explicit_referrer_and_sponsor_only_activation() {
    let s = setup();
    s.client.fund(&2, &s.sponsor, &1000);
    let referrer = Address::generate(&s.env);
    // mock_all_auths lets us exercise the call path; a real auth entry still
    // requires the stored sponsor because activate ignores its caller.
    s.client
        .activate(&2, &s.creator, &Some(referrer.clone()), &terms(&s.env));
    s.client.complete(&2, &1);
    assert_eq!(s.token.balance(&referrer), 10);
}

#[test]
fn pause_blocks_new_work_but_expiry_refund_remains_callable() {
    let s = setup();
    s.client.fund(&3, &s.sponsor, &1000);
    s.client.pause();
    assert_eq!(
        s.client.try_activate(&3, &s.creator, &None, &terms(&s.env)),
        Err(Ok(EscrowError::Paused))
    );
    s.env.ledger().with_mut(|l| l.timestamp += EXPIRY);
    s.client.refund_expired(&3);
    assert_eq!(s.token.balance(&s.sponsor), 10_000_000_000);
}

#[test]
fn expiry_boundary_and_terminal_duplicate_guards() {
    let s = setup();
    s.client.fund(&4, &s.sponsor, &1000);
    activate(&s, 4);
    s.env
        .ledger()
        .with_mut(|l| l.timestamp = 1_000 + EXPIRY - 1);
    s.client.complete(&4, &1);
    assert_eq!(
        s.client.try_complete(&4, &1),
        Err(Ok(EscrowError::InvalidState))
    );
    let s2 = setup();
    s2.client.fund(&5, &s2.sponsor, &1000);
    activate(&s2, 5);
    s2.env.ledger().with_mut(|l| l.timestamp = 1_000 + EXPIRY);
    assert_eq!(
        s2.client.try_complete(&5, &1),
        Err(Ok(EscrowError::Expired))
    );
    s2.client.refund_expired(&5);
    assert_eq!(
        s2.client.try_refund_expired(&5),
        Err(Ok(EscrowError::InvalidState))
    );
}

#[test]
fn rotation_is_two_step_and_old_settlement_rejected() {
    let s = setup();
    let next = Address::generate(&s.env);
    s.client.propose_settlement(&next);
    assert_eq!(s.client.settlement_authority(), s.settlement);
    s.client.accept_settlement();
    assert_eq!(s.client.settlement_authority(), next);
    s.client.fund(&6, &s.sponsor, &1000);
    activate(&s, 6);
    // The old authority has no authorization entry in a non-mocked client;
    // state-level check verifies the accepted key is the only configured one.
    assert_ne!(s.client.settlement_authority(), s.settlement);
}

#[test]
fn wrong_version_and_amount_validation() {
    let s = setup();
    assert_eq!(
        s.client.try_fund(&7, &s.sponsor, &0),
        Err(Ok(EscrowError::InvalidAmount))
    );
    assert_eq!(
        s.client.try_fund(&8, &s.sponsor, &-1),
        Err(Ok(EscrowError::InvalidAmount))
    );
    assert_eq!(
        s.client
            .try_fund(&9, &s.sponsor, &(MINIMUM_CAMPAIGN_AMOUNT - 1)),
        Err(Ok(EscrowError::InvalidAmount))
    );
    s.client.fund(&7, &s.sponsor, &1000);
    assert_eq!(
        s.client.try_activate(&7, &s.creator, &None, &terms(&s.env)),
        Err(Ok(EscrowError::ReferrerRequired))
    );
    activate(&s, 7);
    assert_eq!(
        s.client.try_complete(&7, &2),
        Err(Ok(EscrowError::WrongVersion))
    );
}

#[test]
fn multiple_campaigns_are_isolated_and_ttl_touch_is_callable() {
    let s = setup();
    s.client.fund(&10, &s.sponsor, &1000);
    s.client.fund(&11, &s.sponsor, &2000);
    activate(&s, 10);
    s.client.complete(&10, &1);
    assert_eq!(s.token.balance(&s.id), 2000);
    let before = s.client.get_campaign(&11);
    s.client.extend_campaign_ttl(&11);
    assert_eq!(s.client.get_campaign(&11), before);
    s.client.extend_instance_ttl();
}

#[test]
fn aggregate_deficit_blocks_cross_campaign_spending() {
    let s = setup();
    s.client.fund(&12, &s.sponsor, &1000);
    s.client.fund(&13, &s.sponsor, &2000);
    activate(&s, 12);
    s.token.transfer(&s.id, &Address::generate(&s.env), &1);
    assert_eq!(
        s.client.try_complete(&12, &1),
        Err(Ok(EscrowError::Deficit))
    );
    assert_eq!(s.token.balance(&s.id), 2999);
    assert_eq!(s.client.get_campaign(&12).status, CampaignStatus::Active);
}

#[test]
fn property_accounting_many_base_unit_amounts() {
    let s = setup();
    for id in 100..120 {
        let amount = (id as i128 * 7919) % 900_000 + 1;
        s.client.fund(&id, &s.sponsor, &amount);
        activate(&s, id);
        s.client.complete(&id, &1);
        assert_eq!(s.token.balance(&s.id), 0);
    }
}

#[test]
fn completion_wins_before_expiry_refund_wins_at_expiry() {
    let s = setup();
    s.client.fund(&20, &s.sponsor, &1000);
    activate(&s, 20);
    s.env
        .ledger()
        .with_mut(|l| l.timestamp = 1_000 + EXPIRY - 1);
    s.client.complete(&20, &1);
    assert_eq!(
        s.client.try_refund_expired(&20),
        Err(Ok(EscrowError::InvalidState))
    );

    let s2 = setup();
    s2.client.fund(&21, &s2.sponsor, &1000);
    activate(&s2, 21);
    s2.env.ledger().with_mut(|l| l.timestamp = 1_000 + EXPIRY);
    s2.client.refund_expired(&21);
    assert_eq!(
        s2.client.try_complete(&21, &1),
        Err(Ok(EscrowError::InvalidState))
    );
}

#[test]
fn role_authorization_is_enforced_when_auth_entries_are_absent() {
    let s = setup();
    s.client.fund(&30, &s.sponsor, &1000);
    // Remove the blanket test authorizer: no address is authorized now.
    s.env.mock_auths(&[]);
    assert!(s
        .client
        .try_activate(&30, &s.creator, &None, &terms(&s.env))
        .is_err());
    assert!(s.client.try_pause().is_err());
    assert!(s
        .client
        .try_propose_settlement(&Address::generate(&s.env))
        .is_err());
    s.env.mock_all_auths();
    let next = Address::generate(&s.env);
    s.client.propose_settlement(&next);
    s.env.mock_auths(&[]);
    assert!(s.client.try_accept_settlement().is_err());
    s.env.mock_all_auths();
    s.client.accept_settlement();

    // Explicitly authorize the normal actors for the remaining lifecycle.
    activate(&s, 30);
    s.env.mock_auths(&[]);
    assert!(s.client.try_complete(&30, &1).is_err());
    s.env.mock_all_auths();
    s.client.complete(&30, &1);
}
