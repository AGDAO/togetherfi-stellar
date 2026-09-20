#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env, IntoVal, String, Symbol, Vec,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

struct TestSetup<'a> {
    env: Env,
    contract_id: Address,
    contract: ReputationAnchorClient<'a>,
    admin: Address,
    creator: Address,
}

fn setup<'a>() -> TestSetup<'a> {
    let env = Env::default();
    env.mock_all_auths();

    // Fix ledger timestamp so assertions are deterministic
    env.ledger().with_mut(|li| {
        li.timestamp = 1_720_000_000;
    });

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    let contract_id = env.register_contract(None, ReputationAnchor);
    let contract = ReputationAnchorClient::new(&env, &contract_id);
    contract.initialize(&admin);

    TestSetup {
        env,
        contract_id,
        contract,
        admin,
        creator,
    }
}

// Simulated Arbitrum tx hashes (66-char 0x-prefixed hex strings)
fn arb_hash_a(env: &Env) -> String {
    String::from_str(
        env,
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
    )
}

fn arb_hash_b(env: &Env) -> String {
    String::from_str(
        env,
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
    )
}

fn tier(env: &Env, name: &str) -> Symbol {
    Symbol::new(env, name)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Basic happy path: anchor a score, read it back, verify every field.
#[test]
fn test_anchor_and_retrieve_score() {
    let s = setup();
    let profile_id: u64 = 42;

    s.contract.anchor_score(
        &profile_id,
        &s.creator,
        &750_u32,
        &tier(&s.env, "gold"),
        &arb_hash_a(&s.env),
    );

    let record = s.contract.get_score(&profile_id);
    assert_eq!(record.profile_id, profile_id);
    assert_eq!(record.score, 750_u32);
    assert_eq!(record.tier, tier(&s.env, "gold"));
    assert_eq!(record.creator, s.creator);
    assert_eq!(record.arbitrum_tx_hash, arb_hash_a(&s.env));
    assert_eq!(record.anchored_at, 1_720_000_000_u64);
}

/// Re-anchoring the same profile_id overwrites the old record entirely.
/// The updated score, tier, and timestamp should all reflect the new call.
#[test]
fn test_update_existing_score() {
    let s = setup();
    let profile_id: u64 = 1;

    // First anchor: score 400, silver
    s.contract.anchor_score(
        &profile_id,
        &s.creator,
        &400_u32,
        &tier(&s.env, "silver"),
        &arb_hash_a(&s.env),
    );

    let first = s.contract.get_score(&profile_id);
    assert_eq!(first.score, 400_u32);
    assert_eq!(first.tier, tier(&s.env, "silver"));

    // Advance time and re-anchor with higher score
    s.env.ledger().with_mut(|li| {
        li.timestamp = 1_720_100_000;
    });

    s.contract.anchor_score(
        &profile_id,
        &s.creator,
        &820_u32,
        &tier(&s.env, "gold"),
        &arb_hash_b(&s.env),
    );

    let updated = s.contract.get_score(&profile_id);
    assert_eq!(updated.profile_id, profile_id); // unchanged
    assert_eq!(updated.score, 820_u32);         // updated
    assert_eq!(updated.tier, tier(&s.env, "gold")); // updated
    assert_eq!(updated.anchored_at, 1_720_100_000_u64); // updated
    assert_eq!(updated.arbitrum_tx_hash, arb_hash_b(&s.env)); // updated
}

/// Batch read returns only the profiles that exist. Missing ones are skipped.
/// Each returned record carries its profile_id for caller matching.
#[test]
fn test_batch_read() {
    let s = setup();

    // Anchor three profiles
    for (id, score, t) in [(1u64, 300u32, "bronze"), (3, 650, "silver"), (7, 920, "gold")] {
        s.contract.anchor_score(
            &id,
            &s.creator,
            &score,
            &tier(&s.env, t),
            &arb_hash_a(&s.env),
        );
    }

    // Batch includes two existing profiles and one that was never anchored
    let mut ids: Vec<u64> = Vec::new(&s.env);
    ids.push_back(1_u64);
    ids.push_back(7_u64);
    ids.push_back(99_u64); // not anchored, so it is silently skipped

    let batch = s.contract.get_scores_batch(&ids);

    // Only 2 results returned (99 skipped)
    assert_eq!(batch.len(), 2);

    let r0 = batch.get(0).unwrap();
    assert_eq!(r0.profile_id, 1_u64);
    assert_eq!(r0.score, 300_u32);

    let r1 = batch.get(1).unwrap();
    assert_eq!(r1.profile_id, 7_u64);
    assert_eq!(r1.score, 920_u32);
}

/// An empty batch request returns an empty vector without error.
#[test]
fn test_batch_read_empty_input() {
    let s = setup();
    let ids: Vec<u64> = Vec::new(&s.env);
    let batch = s.contract.get_scores_batch(&ids);
    assert_eq!(batch.len(), 0);
}

/// Attempting to anchor without the admin's auth is rejected.
/// Uses a fresh Env without mock_all_auths so auth checks are enforced.
#[test]
fn test_unauthorized_anchor_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    // Initialize with admin auth mocked for that one call only
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ReputationAnchor);
    let contract = ReputationAnchorClient::new(&env, &contract_id);
    contract.initialize(&admin);

    // Construct a fresh env with NO auth mocking. Re-register so state is clean.
    let env2 = Env::default();
    // No mock_all_auths. require_auth() checks are now strict.
    let contract_id2 = env2.register_contract(None, ReputationAnchor);
    let contract2 = ReputationAnchorClient::new(&env2, &contract_id2);
    // Objects cannot cross Env instances. Generate the creator inside env2.
    let creator_env2 = Address::generate(&env2);

    // Calling anchor_score on an uninitialized contract (no initialize called)
    // fails with NotInitialized before auth is even checked.
    let result = contract2.try_anchor_score(
        &1_u64,
        &creator_env2,
        &500_u32,
        &Symbol::new(&env2, "silver"),
        &String::from_str(&env2, "0xabc"),
    );
    assert!(result.is_err(), "should fail on uninitialized contract");

    // Now initialize the env2 contract, but DON'T mock admin's auth for the
    // subsequent anchor_score call. The contract's stored admin is `admin2`,
    // and without mocking admin2's auth the require_auth() will fail.
    let admin2 = Address::generate(&env2);
    env2.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin2,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id2,
            fn_name: "initialize",
            args: (admin2.clone(),).into_val(&env2),
            sub_invokes: &[],
        },
    }]);
    contract2.initialize(&admin2);

    // Now try anchor_score with NO auth mocked. This must fail.
    let creator2 = Address::generate(&env2);
    let result2 = contract2.try_anchor_score(
        &1_u64,
        &creator2,
        &500_u32,
        &Symbol::new(&env2, "silver"),
        &String::from_str(&env2, "0xabcdef"),
    );
    assert!(result2.is_err(), "anchor_score must be rejected without admin auth");
}

/// Reading a score that was never anchored returns ScoreNotFound.
#[test]
fn test_score_not_found() {
    let s = setup();
    let result = s.contract.try_get_score(&9999_u64);
    assert!(result.is_err());
    // Error should be ScoreNotFound (code 3)
    assert_eq!(
        result.err().unwrap().unwrap(),
        RepAnchorError::ScoreNotFound,
    );
}

/// A second call to initialize is rejected with AlreadyInitialized.
#[test]
fn test_double_initialize_fails() {
    let s = setup();
    let second_admin = Address::generate(&s.env);
    let result = s.contract.try_initialize(&second_admin);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().unwrap(),
        RepAnchorError::AlreadyInitialized,
    );
}

/// Score values above 1000 are rejected with InvalidScore.
#[test]
fn test_invalid_score_out_of_range() {
    let s = setup();
    let result = s.contract.try_anchor_score(
        &1_u64,
        &s.creator,
        &1001_u32, // invalid: max is 1000
        &tier(&s.env, "gold"),
        &arb_hash_a(&s.env),
    );
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().unwrap(),
        RepAnchorError::InvalidScore,
    );
}

/// Score = 1000 is the boundary maximum and must be accepted.
#[test]
fn test_max_score_accepted() {
    let s = setup();
    s.contract.anchor_score(
        &1_u64,
        &s.creator,
        &1000_u32,
        &tier(&s.env, "gold"),
        &arb_hash_a(&s.env),
    );
    let record = s.contract.get_score(&1_u64);
    assert_eq!(record.score, 1000_u32);
}

/// Score = 0 is a valid value (unscored / new creator).
#[test]
fn test_zero_score_accepted() {
    let s = setup();
    s.contract.anchor_score(
        &2_u64,
        &s.creator,
        &0_u32,
        &tier(&s.env, "bronze"),
        &arb_hash_a(&s.env),
    );
    let record = s.contract.get_score(&2_u64);
    assert_eq!(record.score, 0_u32);
}

/// Multiple distinct profile_ids are stored independently; reading one does
/// not affect or return another.
#[test]
fn test_multiple_profiles_independent() {
    let s = setup();
    let creator_a = Address::generate(&s.env);
    let creator_b = Address::generate(&s.env);

    s.contract.anchor_score(&10_u64, &creator_a, &300_u32, &tier(&s.env, "bronze"), &arb_hash_a(&s.env));
    s.contract.anchor_score(&20_u64, &creator_b, &700_u32, &tier(&s.env, "gold"),   &arb_hash_b(&s.env));

    let a = s.contract.get_score(&10_u64);
    let b = s.contract.get_score(&20_u64);

    assert_eq!(a.score, 300_u32);
    assert_eq!(b.score, 700_u32);
    assert_ne!(a.creator, b.creator);
}
