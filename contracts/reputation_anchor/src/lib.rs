//! TogetherFi Reputation Anchor Soroban smart contract.
//!
//! Stores creator TogetherScores natively on Stellar. Each creator's score is
//! keyed by `profile_id` (the TogetherFi database id) and linked back to the
//! corresponding Arbitrum on-chain anchor via `arbitrum_tx_hash`, providing
//! cross-chain identity proof.
//!
//! This replaces the classic Stellar `manageData` account-anchoring approach,
//! which had a critical limitation: every call overwrote the same key on the
//! same platform account with no per-creator uniqueness. This contract stores
//! one independent entry per `profile_id` in Soroban persistent storage.
//!
//! Lifecycle:
//!   1. `initialize`: one-time setup that sets the admin signer.
//!   2. `anchor_score`: admin-only score upsert for a profile_id.
//!                             Emits `ScoreAnchored` event. Updates on re-anchor.
//!   3. `get_score`: public read that returns the full ScoreRecord.
//!   4. `get_scores_batch`: public batch read for the verifiability dashboard.
//!
//! Admin is the TogetherFi backend signer (same keypair used by the campaign
//! escrow contract). Only the admin can write scores; reads are open to all.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short,
    Address, Env, String, Symbol, Vec,
};

// ── Storage keys ──────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Score(u64), // one entry per profile_id
}

// ── Data types ────────────────────────────────────────────────────────────────

/// Full score record stored per creator. Every field is preserved across
/// re-anchors so the dashboard can show the full cross-chain proof.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ScoreRecord {
    /// TogetherFi database profile id. Matches profiles.id on the backend.
    pub profile_id: u64,
    /// Creator's Stellar G-key. Stored for display and verification.
    pub creator: Address,
    /// TogetherScore value, 0-1000.
    pub score: u32,
    /// Human-readable tier: "bronze", "silver", or "gold".
    pub tier: Symbol,
    /// Arbitrum One tx hash from ScoreAnchor.anchor(), used as cross-chain proof.
    /// Full 66-char "0x..." string. Empty string when anchored Stellar-only.
    pub arbitrum_tx_hash: String,
    /// Unix timestamp (seconds) at the ledger when this anchor was written.
    pub anchored_at: u64,
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RepAnchorError {
    AlreadyInitialized = 1,
    NotInitialized     = 2,
    ScoreNotFound      = 3,
    InvalidScore       = 4,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ReputationAnchor;

#[contractimpl]
impl ReputationAnchor {
    /// One-time setup. Sets `admin` as the only address allowed to call
    /// `anchor_score`. Cannot be called a second time.
    pub fn initialize(env: Env, admin: Address) -> Result<(), RepAnchorError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(RepAnchorError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        log!(&env, "reputation_anchor initialized, admin={}", admin);
        Ok(())
    }

    /// Admin-only. Writes or overwrites the TogetherScore for `profile_id`.
    ///
    /// `arbitrum_tx_hash` is the Arbitrum One ScoreAnchor transaction hash
    /// for this anchor, providing cross-chain proof. Pass an empty string
    /// when anchoring Stellar-only (no Arbitrum counterpart).
    ///
    /// `score` must be in range 0..=1000. Values outside this range are
    /// rejected with `InvalidScore`.
    ///
    /// Emits `ScoreAnchored` event on every successful call.
    pub fn anchor_score(
        env: Env,
        profile_id: u64,
        creator_stellar_address: Address,
        score: u32,
        tier: Symbol,
        arbitrum_tx_hash: String,
    ) -> Result<(), RepAnchorError> {
        let admin = Self::require_initialized(&env)?;
        admin.require_auth();

        if score > 1000 {
            return Err(RepAnchorError::InvalidScore);
        }

        let anchored_at = env.ledger().timestamp();

        let record = ScoreRecord {
            profile_id,
            creator: creator_stellar_address,
            score,
            tier: tier.clone(),
            arbitrum_tx_hash: arbitrum_tx_hash.clone(),
            anchored_at,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Score(profile_id), &record);

        // ScoreAnchored event, indexed by (profile_id, score, tier).
        // The dashboard listener and the grant verifier both watch for this.
        env.events().publish(
            (symbol_short!("rep"), symbol_short!("anchored")),
            (profile_id, score, tier, anchored_at, arbitrum_tx_hash),
        );

        log!(
            &env,
            "score anchored: profile_id={} score={} at={}",
            profile_id,
            score,
            anchored_at,
        );
        Ok(())
    }

    /// Public read. Returns the full ScoreRecord for `profile_id`.
    /// Returns `ScoreNotFound` if no score has been anchored for this profile.
    pub fn get_score(env: Env, profile_id: u64) -> Result<ScoreRecord, RepAnchorError> {
        env.storage()
            .persistent()
            .get(&DataKey::Score(profile_id))
            .ok_or(RepAnchorError::ScoreNotFound)
    }

    /// Public batch read. Returns all found ScoreRecords for the supplied
    /// `profile_ids`. Missing entries are silently skipped. Each returned
    /// ScoreRecord carries its own `profile_id` so callers can match results
    /// back to their request list.
    ///
    /// Designed for the TogetherFi verifiability dashboard, which needs to
    /// display multiple creator scores in a single contract call.
    pub fn get_scores_batch(env: Env, profile_ids: Vec<u64>) -> Vec<ScoreRecord> {
        let mut results: Vec<ScoreRecord> = Vec::new(&env);
        for profile_id in profile_ids.iter() {
            if let Some(record) = env
                .storage()
                .persistent()
                .get::<DataKey, ScoreRecord>(&DataKey::Score(profile_id))
            {
                results.push_back(record);
            }
        }
        results
    }

    // ── Internal helpers ───────────────────────────────────────────────────────

    fn require_initialized(env: &Env) -> Result<Address, RepAnchorError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RepAnchorError::NotInitialized)
    }
}

mod test;
