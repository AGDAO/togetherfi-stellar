#![no_std]

//! Immutable contributor-reward pool.
//!
//! Deploy one instance for the MOFO contributor allocation and one for the
//! community contributor allocation. Campaign Escrow sends each completed
//! campaign's allocation to its configured pool. A pool never chooses a
//! recipient: a manifest must be approved by both governance and a separate
//! eligibility authority, and each stored recipient claims their own exact
//! amount.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env,
    Vec,
};

const MAX_ALLOCATIONS: u32 = 64;
const DAY: u64 = 86_400;
const MAX_CLAIM_WINDOW: u64 = 30 * DAY;
// At a conservative five-second ledger cadence this is roughly 40 days.
const TTL: u32 = 700_000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Governance,
    PendingGovernance,
    EligibilityAuthority,
    PendingEligibilityAuthority,
    Token,
    Escrow,
    Paused,
    TotalLiability,
    TotalCredited,
    TotalPaid,
    LatestManifestId,
    CreditedCampaign(u64),
    Manifest(u64),
    Allocation(u64, u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ManifestStatus {
    Active,
    Completed,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Manifest {
    pub manifest_hash: BytesN<32>,
    pub policy_hash: BytesN<32>,
    pub total_amount: i128,
    pub remaining_amount: i128,
    pub allocation_count: u32,
    pub expires_at: u64,
    pub status: ManifestStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Allocation {
    pub recipient: Address,
    pub amount: i128,
    pub claimed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RoleProposal {
    pub role: u32,
    pub account: Address,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PoolError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ManifestAlreadyExists = 3,
    ManifestNotFound = 4,
    InvalidManifest = 5,
    InvalidAllocation = 6,
    InvalidState = 7,
    NotRecipient = 8,
    AlreadyClaimed = 9,
    ManifestExpired = 10,
    NotExpired = 11,
    InsufficientAvailableBalance = 12,
    Deficit = 13,
    Paused = 14,
    NoPendingRole = 15,
    RolesMustDiffer = 16,
    CampaignAlreadyCredited = 17,
}

#[contract]
pub struct ContributorPool;

#[contractimpl]
impl ContributorPool {
    /// The pool accepts allocatable funding only through the configured
    /// Campaign Escrow contract. Direct token transfers are intentionally not
    /// considered credited campaign funding.
    pub fn initialize(
        env: Env,
        governance: Address,
        eligibility_authority: Address,
        token: Address,
        escrow: Address,
    ) -> Result<(), PoolError> {
        if env.storage().instance().has(&DataKey::Governance) {
            return Err(PoolError::AlreadyInitialized);
        }
        if governance == eligibility_authority {
            return Err(PoolError::RolesMustDiffer);
        }
        governance.require_auth();
        eligibility_authority.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Governance, &governance);
        env.storage()
            .instance()
            .set(&DataKey::EligibilityAuthority, &eligibility_authority);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Escrow, &escrow);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::TotalLiability, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalCredited, &0i128);
        env.storage().instance().set(&DataKey::TotalPaid, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::LatestManifestId, &0u64);
        env.storage().instance().extend_ttl(TTL, TTL);
        env.events().publish(
            (symbol_short!("init"),),
            (governance, eligibility_authority, token, escrow),
        );
        Ok(())
    }

    /// Called only as a nested invocation from Campaign Escrow after that
    /// contract has transferred this campaign's fixed pool leg. The campaign
    /// key makes crediting replay-safe, while separate credited accounting
    /// makes unsolicited direct transfers permanently non-allocatable.
    pub fn record_credit(env: Env, campaign_id: u64, amount: i128) -> Result<(), PoolError> {
        Self::init(&env)?;
        if amount <= 0 {
            return Err(PoolError::InvalidAllocation);
        }
        let escrow = Self::escrow(&env)?;
        escrow.require_auth();
        let campaign_key = DataKey::CreditedCampaign(campaign_id);
        if env.storage().persistent().has(&campaign_key) {
            return Err(PoolError::CampaignAlreadyCredited);
        }
        let credited = Self::total_credited(&env)?;
        let paid = Self::total_paid(&env)?;
        let next_credited = credited
            .checked_add(amount)
            .ok_or(PoolError::InvalidAllocation)?;
        let outstanding = next_credited.checked_sub(paid).ok_or(PoolError::Deficit)?;
        if Self::balance(&env) < outstanding {
            return Err(PoolError::Deficit);
        }
        env.storage().persistent().set(&campaign_key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&campaign_key, TTL, TTL);
        env.storage()
            .instance()
            .set(&DataKey::TotalCredited, &next_credited);
        env.events()
            .publish((symbol_short!("credited"), campaign_id), amount);
        Ok(())
    }

    /// Publishes an immutable, exact-recipient allocation manifest. Both
    /// governance and the separately controlled eligibility authority must
    /// authorize the same invocation, so a backend settlement key cannot
    /// create an arbitrary recipient list.
    pub fn publish_manifest(
        env: Env,
        manifest_id: u64,
        manifest_hash: BytesN<32>,
        policy_hash: BytesN<32>,
        expires_at: u64,
        recipients: Vec<Address>,
        amounts: Vec<i128>,
    ) -> Result<(), PoolError> {
        Self::init(&env)?;
        if Self::paused(&env) {
            return Err(PoolError::Paused);
        }
        let governance = Self::current_governance(&env)?;
        let eligibility = Self::current_eligibility_authority(&env)?;
        governance.require_auth();
        eligibility.require_auth();

        let count = recipients.len();
        if count == 0 || count != amounts.len() || count > MAX_ALLOCATIONS {
            return Err(PoolError::InvalidManifest);
        }
        if expires_at <= env.ledger().timestamp() {
            return Err(PoolError::ManifestExpired);
        }
        if expires_at - env.ledger().timestamp() > MAX_CLAIM_WINDOW {
            return Err(PoolError::InvalidManifest);
        }
        let manifest_key = DataKey::Manifest(manifest_id);
        if env.storage().persistent().has(&manifest_key) {
            return Err(PoolError::ManifestAlreadyExists);
        }
        let latest_manifest_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LatestManifestId)
            .unwrap_or(0);
        if manifest_id <= latest_manifest_id {
            return Err(PoolError::ManifestAlreadyExists);
        }

        let mut total = 0i128;
        for index in 0..count {
            let recipient = recipients.get(index).ok_or(PoolError::InvalidManifest)?;
            let amount = amounts.get(index).ok_or(PoolError::InvalidManifest)?;
            if amount <= 0 {
                return Err(PoolError::InvalidAllocation);
            }
            for prior_index in 0..index {
                if recipients
                    .get(prior_index)
                    .ok_or(PoolError::InvalidManifest)?
                    == recipient
                {
                    return Err(PoolError::InvalidAllocation);
                }
            }
            total = total
                .checked_add(amount)
                .ok_or(PoolError::InvalidAllocation)?;
        }

        let available = Self::available_balance(env.clone())?;
        if available < total {
            return Err(PoolError::InsufficientAvailableBalance);
        }
        let liability = Self::stored_total_liability(&env)?;
        let required = liability
            .checked_add(total)
            .ok_or(PoolError::InvalidAllocation)?;

        for index in 0..count {
            let allocation = Allocation {
                recipient: recipients.get(index).ok_or(PoolError::InvalidManifest)?,
                amount: amounts.get(index).ok_or(PoolError::InvalidManifest)?,
                claimed: false,
            };
            let key = DataKey::Allocation(manifest_id, index);
            env.storage().persistent().set(&key, &allocation);
            env.storage().persistent().extend_ttl(&key, TTL, TTL);
        }
        let manifest = Manifest {
            manifest_hash,
            policy_hash,
            total_amount: total,
            remaining_amount: total,
            allocation_count: count,
            expires_at,
            status: ManifestStatus::Active,
        };
        env.storage().persistent().set(&manifest_key, &manifest);
        env.storage()
            .persistent()
            .extend_ttl(&manifest_key, TTL, TTL);
        env.storage()
            .instance()
            .set(&DataKey::TotalLiability, &required);
        env.storage()
            .instance()
            .set(&DataKey::LatestManifestId, &manifest_id);
        env.events().publish(
            (symbol_short!("manifest"), manifest_id),
            (
                manifest.total_amount,
                manifest.allocation_count,
                manifest.expires_at,
            ),
        );
        Ok(())
    }

    /// Claims an exact, pre-approved allocation. Neither recipient nor amount
    /// is caller supplied, and an allocation can only be claimed once.
    pub fn claim(env: Env, manifest_id: u64, allocation_index: u32) -> Result<(), PoolError> {
        Self::init(&env)?;
        if Self::paused(&env) {
            return Err(PoolError::Paused);
        }
        let manifest_key = DataKey::Manifest(manifest_id);
        let mut manifest = Self::manifest(&env, manifest_id)?;
        if manifest.status != ManifestStatus::Active {
            return Err(PoolError::InvalidState);
        }
        if env.ledger().timestamp() >= manifest.expires_at {
            return Err(PoolError::ManifestExpired);
        }
        if allocation_index >= manifest.allocation_count {
            return Err(PoolError::InvalidAllocation);
        }
        let allocation_key = DataKey::Allocation(manifest_id, allocation_index);
        let mut allocation: Allocation = env
            .storage()
            .persistent()
            .get(&allocation_key)
            .ok_or(PoolError::InvalidAllocation)?;
        if allocation.claimed {
            return Err(PoolError::AlreadyClaimed);
        }
        allocation.recipient.require_auth();

        let liability = Self::stored_total_liability(&env)?;
        if liability < allocation.amount || Self::balance(&env) < liability {
            return Err(PoolError::Deficit);
        }

        allocation.claimed = true;
        manifest.remaining_amount = manifest
            .remaining_amount
            .checked_sub(allocation.amount)
            .ok_or(PoolError::Deficit)?;
        if manifest.remaining_amount == 0 {
            manifest.status = ManifestStatus::Completed;
        }
        env.storage().persistent().set(&allocation_key, &allocation);
        env.storage()
            .persistent()
            .extend_ttl(&allocation_key, TTL, TTL);
        env.storage().persistent().set(&manifest_key, &manifest);
        env.storage()
            .persistent()
            .extend_ttl(&manifest_key, TTL, TTL);
        env.storage()
            .instance()
            .set(&DataKey::TotalLiability, &(liability - allocation.amount));
        let paid = Self::total_paid(&env)?;
        env.storage().instance().set(
            &DataKey::TotalPaid,
            &paid
                .checked_add(allocation.amount)
                .ok_or(PoolError::Deficit)?,
        );
        let token = Self::token(&env)?;
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &allocation.recipient,
            &allocation.amount,
        );
        env.events().publish(
            (symbol_short!("claimed"), manifest_id, allocation_index),
            (
                allocation.recipient,
                allocation.amount,
                manifest.remaining_amount,
            ),
        );
        Ok(())
    }

    /// After a published claim window ends, any unclaimed amount becomes
    /// available to a later cycle. This performs no transfer and cannot send
    /// funds to governance, an operator, or an arbitrary destination.
    pub fn expire_manifest(env: Env, manifest_id: u64) -> Result<(), PoolError> {
        Self::init(&env)?;
        let key = DataKey::Manifest(manifest_id);
        let mut manifest = Self::manifest(&env, manifest_id)?;
        if manifest.status != ManifestStatus::Active {
            return Err(PoolError::InvalidState);
        }
        if env.ledger().timestamp() < manifest.expires_at {
            return Err(PoolError::NotExpired);
        }
        let liability = Self::stored_total_liability(&env)?;
        if liability < manifest.remaining_amount {
            return Err(PoolError::Deficit);
        }
        let released = manifest.remaining_amount;
        manifest.remaining_amount = 0;
        manifest.status = ManifestStatus::Expired;
        env.storage().persistent().set(&key, &manifest);
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        env.storage()
            .instance()
            .set(&DataKey::TotalLiability, &(liability - released));
        env.events()
            .publish((symbol_short!("expired"), manifest_id), released);
        Ok(())
    }

    pub fn propose_governance(env: Env, account: Address) -> Result<(), PoolError> {
        Self::propose_role(&env, 1, account)
    }

    pub fn accept_governance(env: Env) -> Result<(), PoolError> {
        Self::accept_role(&env, 1)
    }

    pub fn propose_eligibility_authority(env: Env, account: Address) -> Result<(), PoolError> {
        Self::propose_role(&env, 2, account)
    }

    pub fn accept_eligibility_authority(env: Env) -> Result<(), PoolError> {
        Self::accept_role(&env, 2)
    }

    pub fn pause(env: Env) -> Result<(), PoolError> {
        Self::init(&env)?;
        Self::current_governance(&env)?.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
        env.storage().instance().extend_ttl(TTL, TTL);
        env.events().publish((symbol_short!("paused"),), ());
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), PoolError> {
        Self::init(&env)?;
        Self::current_governance(&env)?.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().extend_ttl(TTL, TTL);
        env.events().publish((symbol_short!("unpaused"),), ());
        Ok(())
    }

    pub fn get_manifest(env: Env, manifest_id: u64) -> Result<Manifest, PoolError> {
        let key = DataKey::Manifest(manifest_id);
        let manifest = Self::manifest(&env, manifest_id)?;
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        Ok(manifest)
    }

    pub fn get_allocation(
        env: Env,
        manifest_id: u64,
        allocation_index: u32,
    ) -> Result<Allocation, PoolError> {
        let key = DataKey::Allocation(manifest_id, allocation_index);
        let allocation = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(PoolError::InvalidAllocation)?;
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        Ok(allocation)
    }

    pub fn total_liability(env: Env) -> Result<i128, PoolError> {
        Self::init(&env)?;
        Self::stored_total_liability(&env)
    }

    pub fn available_balance(env: Env) -> Result<i128, PoolError> {
        Self::init(&env)?;
        let liability = Self::stored_total_liability(&env)?;
        let outstanding = Self::total_credited(&env)?
            .checked_sub(Self::total_paid(&env)?)
            .ok_or(PoolError::Deficit)?;
        if outstanding < liability || Self::balance(&env) < outstanding {
            return Err(PoolError::Deficit);
        }
        Ok(outstanding - liability)
    }

    pub fn is_paused(env: Env) -> bool {
        Self::paused(&env)
    }

    pub fn governance(env: Env) -> Result<Address, PoolError> {
        Self::current_governance(&env)
    }

    pub fn eligibility_authority(env: Env) -> Result<Address, PoolError> {
        Self::current_eligibility_authority(&env)
    }

    pub fn escrow_contract(env: Env) -> Result<Address, PoolError> {
        Self::escrow(&env)
    }

    pub fn token_contract(env: Env) -> Result<Address, PoolError> {
        Self::token(&env)
    }

    pub fn extend_instance_ttl(env: Env) -> Result<(), PoolError> {
        Self::init(&env)?;
        env.storage().instance().extend_ttl(TTL, TTL);
        Ok(())
    }

    pub fn extend_manifest_ttl(env: Env, manifest_id: u64) -> Result<(), PoolError> {
        let key = DataKey::Manifest(manifest_id);
        let manifest = Self::manifest(&env, manifest_id)?;
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        for index in 0..manifest.allocation_count {
            let allocation_key = DataKey::Allocation(manifest_id, index);
            if !env.storage().persistent().has(&allocation_key) {
                return Err(PoolError::InvalidAllocation);
            }
            env.storage()
                .persistent()
                .extend_ttl(&allocation_key, TTL, TTL);
        }
        Ok(())
    }

    fn init(env: &Env) -> Result<(), PoolError> {
        if env.storage().instance().has(&DataKey::Governance) {
            env.storage().instance().extend_ttl(TTL, TTL);
            Ok(())
        } else {
            Err(PoolError::NotInitialized)
        }
    }

    fn token(env: &Env) -> Result<Address, PoolError> {
        env.storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(PoolError::NotInitialized)
    }

    fn escrow(env: &Env) -> Result<Address, PoolError> {
        env.storage()
            .instance()
            .get(&DataKey::Escrow)
            .ok_or(PoolError::NotInitialized)
    }

    fn current_governance(env: &Env) -> Result<Address, PoolError> {
        env.storage()
            .instance()
            .get(&DataKey::Governance)
            .ok_or(PoolError::NotInitialized)
    }

    fn current_eligibility_authority(env: &Env) -> Result<Address, PoolError> {
        env.storage()
            .instance()
            .get(&DataKey::EligibilityAuthority)
            .ok_or(PoolError::NotInitialized)
    }

    fn manifest(env: &Env, manifest_id: u64) -> Result<Manifest, PoolError> {
        env.storage()
            .persistent()
            .get(&DataKey::Manifest(manifest_id))
            .ok_or(PoolError::ManifestNotFound)
    }

    fn stored_total_liability(env: &Env) -> Result<i128, PoolError> {
        Self::init(env)?;
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::TotalLiability)
            .unwrap_or(0))
    }

    fn total_credited(env: &Env) -> Result<i128, PoolError> {
        Self::init(env)?;
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::TotalCredited)
            .unwrap_or(0))
    }

    fn total_paid(env: &Env) -> Result<i128, PoolError> {
        Self::init(env)?;
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::TotalPaid)
            .unwrap_or(0))
    }

    fn balance(env: &Env) -> i128 {
        let token = Self::token(env).unwrap();
        token::Client::new(env, &token).balance(&env.current_contract_address())
    }

    fn paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    fn propose_role(env: &Env, role: u32, account: Address) -> Result<(), PoolError> {
        Self::init(env)?;
        Self::current_governance(env)?.require_auth();
        let key = if role == 1 {
            DataKey::PendingGovernance
        } else {
            DataKey::PendingEligibilityAuthority
        };
        env.storage().instance().set(&key, &account);
        env.events()
            .publish((symbol_short!("roleprop"), role), account);
        Ok(())
    }

    fn accept_role(env: &Env, role: u32) -> Result<(), PoolError> {
        Self::init(env)?;
        let pending_key = if role == 1 {
            DataKey::PendingGovernance
        } else {
            DataKey::PendingEligibilityAuthority
        };
        let account: Address = env
            .storage()
            .instance()
            .get(&pending_key)
            .ok_or(PoolError::NoPendingRole)?;
        account.require_auth();
        let other_role = if role == 1 {
            Self::current_eligibility_authority(env)?
        } else {
            Self::current_governance(env)?
        };
        if account == other_role {
            return Err(PoolError::RolesMustDiffer);
        }
        let target = if role == 1 {
            DataKey::Governance
        } else {
            DataKey::EligibilityAuthority
        };
        env.storage().instance().set(&target, &account);
        env.storage().instance().remove(&pending_key);
        env.events()
            .publish((symbol_short!("roleacc"), role), account);
        Ok(())
    }
}

mod test;
// Contract tests are compiled only for the native test target.
