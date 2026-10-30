#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env, IntoVal, Vec,
};

struct Setup<'a> {
    env: Env,
    id: Address,
    client: ContributorPoolClient<'a>,
    token: TokenClient<'a>,
    governance: Address,
    eligibility: Address,
    funder: Address,
    alice: Address,
    bob: Address,
}

fn hash(env: &Env, fill: u8) -> BytesN<32> {
    BytesN::from_array(env, &[fill; 32])
}

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|ledger| ledger.timestamp = 1_000);
    let governance = Address::generate(&env);
    let eligibility = Address::generate(&env);
    let escrow = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token = TokenClient::new(&env, &token_id);
    StellarAssetClient::new(&env, &token_id).mint(&funder, &10_000_000);
    let deployment_salt = BytesN::from_array(&env, &[1; 32]);
    let id = env.deployer().with_address(governance.clone(), deployment_salt.clone()).deployed_address();
    env.register_contract(Some(&id), ContributorPool);
    let client = ContributorPoolClient::new(&env, &id);
    client.initialize(&governance, &eligibility, &token_id, &escrow, &governance, &deployment_salt);
    Setup {
        env,
        id,
        client,
        token,
        governance,
        eligibility,
        funder,
        alice,
        bob,
    }
}

fn fund(s: &Setup, campaign_id: u64, amount: i128) {
    s.token.transfer(&s.funder, &s.id, &amount);
    s.client.record_credit(&campaign_id, &amount);
}

fn recipients(s: &Setup) -> Vec<Address> {
    let mut recipients = Vec::new(&s.env);
    recipients.push_back(s.alice.clone());
    recipients.push_back(s.bob.clone());
    recipients
}

fn amounts(s: &Setup, first: i128, second: i128) -> Vec<i128> {
    let mut amounts = Vec::new(&s.env);
    amounts.push_back(first);
    amounts.push_back(second);
    amounts
}

fn publish(s: &Setup, id: u64, expires_at: u64) {
    s.client.publish_manifest(
        &id,
        &hash(&s.env, 1),
        &hash(&s.env, 2),
        &expires_at,
        &recipients(s),
        &amounts(s, 400, 600),
    );
}

#[test]
fn manifest_is_exact_immutable_and_claims_are_recipient_authorized() {
    let s = setup();
    fund(&s, 1, 1_000);
    publish(&s, 1, 2_000);
    let manifest = s.client.get_manifest(&1);
    assert_eq!(manifest.total_amount, 1_000);
    assert_eq!(manifest.remaining_amount, 1_000);
    assert_eq!(s.client.available_balance(), 0);
    assert_eq!(
        s.client.try_publish_manifest(
            &1,
            &hash(&s.env, 9),
            &hash(&s.env, 8),
            &2_500,
            &recipients(&s),
            &amounts(&s, 1, 999),
        ),
        Err(Ok(PoolError::ManifestAlreadyExists))
    );
    s.client.claim(&1, &0);
    assert_eq!(s.token.balance(&s.alice), 400);
    assert_eq!(s.client.get_manifest(&1).remaining_amount, 600);
    assert_eq!(
        s.client.try_claim(&1, &0),
        Err(Ok(PoolError::AlreadyClaimed))
    );
    s.client.claim(&1, &1);
    assert_eq!(s.token.balance(&s.bob), 600);
    assert_eq!(s.client.get_manifest(&1).status, ManifestStatus::Completed);
    assert_eq!(s.client.total_liability(), 0);
}

#[test]
fn independent_authorities_cannot_be_collapsed_to_one_signer() {
    let env = Env::default();
    env.mock_all_auths();
    let governance = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin);
    let deployment_salt = BytesN::from_array(&env, &[1; 32]);
    let id = env.deployer().with_address(governance.clone(), deployment_salt.clone()).deployed_address();
    env.register_contract(Some(&id), ContributorPool);
    let client = ContributorPoolClient::new(&env, &id);
    assert_eq!(
        client.try_initialize(
            &governance,
            &governance,
            &token_id,
            &Address::generate(&env),
            &governance,
            &deployment_salt,
        ),
        Err(Ok(PoolError::RolesMustDiffer))
    );

    let s = setup();
    s.client.propose_eligibility_authority(&s.governance);
    assert_eq!(
        s.client.try_accept_eligibility_authority(),
        Err(Ok(PoolError::RolesMustDiffer))
    );
    assert_eq!(s.client.eligibility_authority(), s.eligibility);
}

#[test]
fn first_initialization_requires_the_actual_deployment_authority() {
    let env = Env::default();
    let deployer = Address::generate(&env);
    let attacker = Address::generate(&env);
    let governance = Address::generate(&env);
    let eligibility = Address::generate(&env);
    let token = Address::generate(&env);
    let escrow = Address::generate(&env);
    let salt = BytesN::from_array(&env, &[7; 32]);
    let id = env.deployer().with_address(deployer.clone(), salt.clone()).deployed_address();
    env.register_contract(Some(&id), ContributorPool);
    let client = ContributorPoolClient::new(&env, &id);
    assert_eq!(client.try_initialize(&attacker, &eligibility, &token, &escrow, &attacker, &salt),
        Err(Ok(PoolError::InvalidDeployment)));
    assert_eq!(client.try_initialize(&governance, &eligibility, &token, &escrow, &deployer, &BytesN::from_array(&env, &[8; 32])),
        Err(Ok(PoolError::InvalidDeployment)));
    assert!(client.try_initialize(&attacker, &eligibility, &token, &escrow, &deployer, &salt).is_err());
    let args = (governance.clone(), eligibility.clone(), token.clone(), escrow.clone(), deployer.clone(), salt.clone()).into_val(&env);
    let invoke = MockAuthInvoke { contract: &id, fn_name: "initialize", args, sub_invokes: &[] };
    env.mock_auths(&[MockAuth { address: &deployer, invoke: &invoke }]);
    assert!(client.try_initialize(&governance, &eligibility, &token, &escrow, &deployer, &salt).is_err());
    env.mock_auths(&[
        MockAuth { address: &deployer, invoke: &invoke },
        MockAuth { address: &governance, invoke: &invoke },
    ]);
    assert!(client.try_initialize(&governance, &eligibility, &token, &escrow, &deployer, &salt).is_err());
    env.mock_auths(&[
        MockAuth { address: &deployer, invoke: &invoke },
        MockAuth { address: &governance, invoke: &invoke },
        MockAuth { address: &eligibility, invoke: &invoke },
    ]);
    client.initialize(&governance, &eligibility, &token, &escrow, &deployer, &salt);
    assert_eq!(client.governance(), governance);
    assert_eq!(client.eligibility_authority(), eligibility);
    // Deployment uses the deployer as the fixture eligibility authority.
    // One authorization must satisfy both appearances of that same address.
    let salt2 = BytesN::from_array(&env, &[9; 32]);
    let id2 = env.deployer().with_address(deployer.clone(), salt2.clone()).deployed_address();
    env.register_contract(Some(&id2), ContributorPool);
    let args2 = (governance.clone(), deployer.clone(), token.clone(), escrow.clone(), deployer.clone(), salt2.clone()).into_val(&env);
    let invoke2 = MockAuthInvoke { contract: &id2, fn_name: "initialize", args: args2, sub_invokes: &[] };
    env.mock_auths(&[
        MockAuth { address: &deployer, invoke: &invoke2 },
        MockAuth { address: &governance, invoke: &invoke2 },
    ]);
    ContributorPoolClient::new(&env, &id2).initialize(&governance, &deployer, &token, &escrow, &deployer, &salt2);
}

#[test]
fn insufficient_funds_and_duplicate_recipients_fail_closed() {
    let s = setup();
    fund(&s, 1, 999);
    assert_eq!(
        s.client.try_publish_manifest(
            &1,
            &hash(&s.env, 1),
            &hash(&s.env, 2),
            &2_000,
            &recipients(&s),
            &amounts(&s, 400, 600),
        ),
        Err(Ok(PoolError::InsufficientAvailableBalance))
    );
    fund(&s, 2, 1);
    let mut duplicate_recipients = Vec::new(&s.env);
    duplicate_recipients.push_back(s.alice.clone());
    duplicate_recipients.push_back(s.alice.clone());
    assert_eq!(
        s.client.try_publish_manifest(
            &2,
            &hash(&s.env, 1),
            &hash(&s.env, 2),
            &2_000,
            &duplicate_recipients,
            &amounts(&s, 400, 600),
        ),
        Err(Ok(PoolError::InvalidAllocation))
    );
}

#[test]
fn unclaimed_amount_carries_forward_after_expiry_without_any_transfer() {
    let s = setup();
    fund(&s, 1, 1_500);
    publish(&s, 1, 2_000);
    s.client.claim(&1, &0);
    s.env.ledger().with_mut(|ledger| ledger.timestamp = 2_000);
    s.client.expire_manifest(&1);
    assert_eq!(s.client.get_manifest(&1).status, ManifestStatus::Expired);
    assert_eq!(s.client.total_liability(), 0);
    assert_eq!(s.client.available_balance(), 1_100);
    assert_eq!(s.token.balance(&s.governance), 0);
    assert_eq!(s.token.balance(&s.eligibility), 0);

    s.client.publish_manifest(
        &2,
        &hash(&s.env, 3),
        &hash(&s.env, 4),
        &3_000,
        &recipients(&s),
        &amounts(&s, 500, 600),
    );
    assert_eq!(s.client.total_liability(), 1_100);
}

#[test]
fn claim_window_is_bounded_and_manifest_ids_cannot_be_reused() {
    let s = setup();
    fund(&s, 1, 2_000);
    assert_eq!(
        s.client.try_publish_manifest(
            &1,
            &hash(&s.env, 1),
            &hash(&s.env, 2),
            &(1_000 + MAX_CLAIM_WINDOW + 1),
            &recipients(&s),
            &amounts(&s, 400, 600),
        ),
        Err(Ok(PoolError::InvalidManifest))
    );
    publish(&s, 1, 2_000);
    s.client.extend_manifest_ttl(&1);
    s.env.ledger().with_mut(|ledger| ledger.timestamp = 2_000);
    s.client.expire_manifest(&1);
    assert_eq!(
        s.client.try_publish_manifest(
            &1,
            &hash(&s.env, 3),
            &hash(&s.env, 4),
            &3_000,
            &recipients(&s),
            &amounts(&s, 400, 600),
        ),
        Err(Ok(PoolError::ManifestAlreadyExists))
    );
}

#[test]
fn pause_and_role_rotation_do_not_allow_arbitrary_release() {
    let s = setup();
    fund(&s, 1, 1_000);
    s.client.pause();
    assert_eq!(
        s.client.try_publish_manifest(
            &1,
            &hash(&s.env, 1),
            &hash(&s.env, 2),
            &2_000,
            &recipients(&s),
            &amounts(&s, 400, 600),
        ),
        Err(Ok(PoolError::Paused))
    );
    s.client.unpause();
    let next = Address::generate(&s.env);
    s.client.propose_eligibility_authority(&next);
    assert_eq!(s.client.eligibility_authority(), s.eligibility);
    s.client.accept_eligibility_authority();
    assert_eq!(s.client.eligibility_authority(), next);
}

#[test]
fn role_auth_and_claim_auth_are_enforced_without_blanket_auth() {
    let s = setup();
    fund(&s, 1, 1_000);
    s.env.mock_auths(&[]);
    assert!(s
        .client
        .try_publish_manifest(
            &1,
            &hash(&s.env, 1),
            &hash(&s.env, 2),
            &2_000,
            &recipients(&s),
            &amounts(&s, 400, 600),
        )
        .is_err());
    s.env.mock_all_auths();
    publish(&s, 1, 2_000);
    s.env.mock_auths(&[]);
    assert!(s.client.try_claim(&1, &0).is_err());
}

#[test]
fn direct_token_transfers_never_become_allocatable_campaign_funding() {
    let s = setup();
    s.token.transfer(&s.funder, &s.id, &1_000);
    assert_eq!(s.client.available_balance(), 0);
    assert_eq!(
        s.client.try_publish_manifest(
            &1,
            &hash(&s.env, 1),
            &hash(&s.env, 2),
            &2_000,
            &recipients(&s),
            &amounts(&s, 400, 600),
        ),
        Err(Ok(PoolError::InsufficientAvailableBalance))
    );
    s.client.record_credit(&1, &1_000);
    assert_eq!(s.client.available_balance(), 1_000);
    assert_eq!(
        s.client.try_record_credit(&1, &1_000),
        Err(Ok(PoolError::CampaignAlreadyCredited))
    );
}
// End of contributor-pool test module.
