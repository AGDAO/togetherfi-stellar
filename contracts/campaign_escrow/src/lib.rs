#![no_std]

//! CreatorFi-only campaign escrow (v2).
//!
//! The contract deliberately has no relayer or rescue operation.  Settlement
//! is submitted by the settlement authority and all payout destinations are
//! committed by the sponsor before a campaign can be settled.

use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractclient, contracterror, contractimpl, contracttype, symbol_short, token,
    Address, Bytes, Env, IntoVal, Symbol, Val, Vec,
};

const DAY: u64 = 86_400;
const EXPIRY: u64 = 30 * DAY;
const DENOM: i128 = 10_000;
const SERVICE: i128 = 1_000;
const NFT_POOL: i128 = 500;
const REVENUE: i128 = 200;
const REFERRER: i128 = 100;
// At least 100 base units makes every 5% / 2% / 1% leg a positive integer,
// so each contributor pool can receive and authenticate its campaign credit.
const MINIMUM_CAMPAIGN_AMOUNT: i128 = 100;
// Soroban TTLs are ledger counts, not seconds.  At the conservative five
// second ledger cadence this is about 40 days, keeping a funded campaign
// alive through its 30-day timestamp expiry even without a keeper touch.
const TTL: u32 = 700_000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Governance,
    PendingGovernance,
    Settlement,
    PendingSettlement,
    Token,
    ServiceTreasury,
    NftPool,
    RevenuePool,
    Paused,
    ConfigVersion,
    TotalLiability,
    Campaign(u64),
    ConsumedOperation(u64, u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CampaignStatus {
    Funded,
    Active,
    Completed,
    Refunded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Campaign {
    pub sponsor: Address,
    pub amount: i128,
    pub creator: Address,
    pub referrer: Option<Address>,
    pub terms_hash: Bytes,
    pub funded_at: u64,
    pub expires_at: u64,
    pub operation_version: u32,
    pub status: CampaignStatus,
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
pub enum EscrowError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    CampaignAlreadyExists = 3,
    CampaignNotFound = 4,
    InvalidAmount = 5,
    InvalidState = 6,
    Paused = 7,
    Expired = 8,
    NotExpired = 9,
    WrongVersion = 10,
    EmptyTerms = 11,
    NoPendingRole = 12,
    Deficit = 13,
    InvalidContributorPool = 14,
    ReferrerRequired = 15,
}

#[contract]
pub struct CampaignEscrow;

#[contractclient(name = "ContributorPoolClient")]
pub trait ContributorPoolInterface {
    fn escrow_contract(env: Env) -> Address;
    fn token_contract(env: Env) -> Address;
    fn record_credit(env: Env, campaign_id: u64, amount: i128);
}

#[contractimpl]
impl CampaignEscrow {
    pub fn initialize(
        env: Env,
        governance: Address,
        settlement: Address,
        token: Address,
        service_treasury: Address,
        nft_pool: Address,
        revenue_pool: Address,
    ) -> Result<(), EscrowError> {
        if env.storage().instance().has(&DataKey::Governance) {
            return Err(EscrowError::AlreadyInitialized);
        }
        if nft_pool == revenue_pool {
            return Err(EscrowError::InvalidContributorPool);
        }
        let nft_client = ContributorPoolClient::new(&env, &nft_pool);
        let revenue_client = ContributorPoolClient::new(&env, &revenue_pool);
        let this_contract = env.current_contract_address();
        if nft_client.escrow_contract() != this_contract
            || revenue_client.escrow_contract() != this_contract
            || nft_client.token_contract() != token
            || revenue_client.token_contract() != token
        {
            return Err(EscrowError::InvalidContributorPool);
        }
        governance.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Governance, &governance);
        env.storage()
            .instance()
            .set(&DataKey::Settlement, &settlement);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::ServiceTreasury, &service_treasury);
        env.storage().instance().set(&DataKey::NftPool, &nft_pool);
        env.storage()
            .instance()
            .set(&DataKey::RevenuePool, &revenue_pool);
        env.storage().instance().set(&DataKey::ConfigVersion, &1u32);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::TotalLiability, &0i128);
        env.storage().instance().extend_ttl(TTL, TTL);
        Ok(())
    }

    pub fn fund(
        env: Env,
        campaign_id: u64,
        sponsor: Address,
        amount: i128,
    ) -> Result<(), EscrowError> {
        Self::init(&env)?;
        if amount < MINIMUM_CAMPAIGN_AMOUNT || amount > i128::MAX / DENOM {
            return Err(EscrowError::InvalidAmount);
        }
        if Self::paused(&env) {
            return Err(EscrowError::Paused);
        }
        let key = DataKey::Campaign(campaign_id);
        if env.storage().persistent().has(&key) {
            return Err(EscrowError::CampaignAlreadyExists);
        }
        sponsor.require_auth();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        token::Client::new(&env, &token).transfer(
            &sponsor,
            &env.current_contract_address(),
            &amount,
        );
        let liability: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalLiability)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalLiability, &(liability + amount));
        let now = env.ledger().timestamp();
        let c = Campaign {
            sponsor: sponsor.clone(),
            amount,
            creator: env.current_contract_address(),
            referrer: None,
            terms_hash: Bytes::new(&env),
            funded_at: now,
            expires_at: now + EXPIRY,
            operation_version: 0,
            status: CampaignStatus::Funded,
        };
        env.storage().persistent().set(&key, &c);
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        env.events().publish(
            (symbol_short!("funded"), campaign_id),
            (sponsor, amount, now),
        );
        Ok(())
    }

    /// Commits the creator, required referrer, and application terms. The
    /// sponsor is the only party able to activate its campaign.
    pub fn activate(
        env: Env,
        campaign_id: u64,
        creator: Address,
        referrer: Option<Address>,
        terms_hash: Bytes,
    ) -> Result<(), EscrowError> {
        Self::init(&env)?;
        if Self::paused(&env) {
            return Err(EscrowError::Paused);
        }
        if terms_hash.is_empty() {
            return Err(EscrowError::EmptyTerms);
        }
        if referrer.is_none() {
            return Err(EscrowError::ReferrerRequired);
        }
        let key = DataKey::Campaign(campaign_id);
        let mut c = Self::campaign(&env, campaign_id)?;
        if c.status != CampaignStatus::Funded {
            return Err(EscrowError::InvalidState);
        }
        c.sponsor.require_auth();
        c.creator = creator;
        c.referrer = referrer;
        c.terms_hash = terms_hash;
        c.status = CampaignStatus::Active;
        c.operation_version = 1;
        env.storage().persistent().set(&key, &c);
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        env.events().publish(
            (symbol_short!("activated"), campaign_id),
            (c.creator.clone(), c.terms_hash.clone(), c.expires_at),
        );
        Ok(())
    }

    /// Settlement authority submits directly; no relayer role exists.
    pub fn complete(env: Env, campaign_id: u64, operation_version: u32) -> Result<(), EscrowError> {
        Self::init(&env)?;
        if Self::paused(&env) {
            return Err(EscrowError::Paused);
        }
        let settlement: Address = env.storage().instance().get(&DataKey::Settlement).unwrap();
        settlement.require_auth();
        let key = DataKey::Campaign(campaign_id);
        let mut c = Self::campaign(&env, campaign_id)?;
        if c.status != CampaignStatus::Active {
            return Err(EscrowError::InvalidState);
        }
        if operation_version != c.operation_version {
            return Err(EscrowError::WrongVersion);
        }
        if env.ledger().timestamp() >= c.expires_at {
            return Err(EscrowError::Expired);
        }
        let op_key = DataKey::ConsumedOperation(campaign_id, operation_version);
        if env.storage().persistent().has(&op_key) {
            return Err(EscrowError::WrongVersion);
        }
        c.status = CampaignStatus::Completed;
        env.storage().persistent().set(&key, &c);
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        env.storage().persistent().set(&op_key, &true);
        env.storage().persistent().extend_ttl(&op_key, TTL, TTL);
        let total = c.amount;
        let liability: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalLiability)
            .unwrap_or(0);
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let tc = token::Client::new(&env, &token);
        let escrow = env.current_contract_address();
        if liability < total || tc.balance(&escrow) < liability {
            return Err(EscrowError::Deficit);
        }
        env.storage()
            .instance()
            .set(&DataKey::TotalLiability, &(liability - total));
        // Creator receives the explicit integer remainder, making the sum
        // exactly total for every base-unit amount.
        let service = total
            .checked_mul(SERVICE)
            .ok_or(EscrowError::InvalidAmount)?
            / DENOM;
        let nft = total
            .checked_mul(NFT_POOL)
            .ok_or(EscrowError::InvalidAmount)?
            / DENOM;
        let revenue = total
            .checked_mul(REVENUE)
            .ok_or(EscrowError::InvalidAmount)?
            / DENOM;
        let referrer = total
            .checked_mul(REFERRER)
            .ok_or(EscrowError::InvalidAmount)?
            / DENOM;
        let referrer_address = c.referrer.clone().ok_or(EscrowError::ReferrerRequired)?;
        let creator = total - service - nft - revenue - referrer;
        tc.transfer(&escrow, &c.creator, &creator);
        tc.transfer(
            &escrow,
            &env.storage()
                .instance()
                .get(&DataKey::ServiceTreasury)
                .unwrap(),
            &service,
        );
        tc.transfer(
            &escrow,
            &env.storage().instance().get(&DataKey::NftPool).unwrap(),
            &nft,
        );
        let nft_pool: Address = env.storage().instance().get(&DataKey::NftPool).unwrap();
        Self::authorize_pool_credit(&env, &nft_pool, campaign_id, nft);
        ContributorPoolClient::new(&env, &nft_pool).record_credit(&campaign_id, &nft);
        tc.transfer(
            &escrow,
            &env.storage().instance().get(&DataKey::RevenuePool).unwrap(),
            &revenue,
        );
        let revenue_pool: Address = env.storage().instance().get(&DataKey::RevenuePool).unwrap();
        Self::authorize_pool_credit(&env, &revenue_pool, campaign_id, revenue);
        ContributorPoolClient::new(&env, &revenue_pool).record_credit(&campaign_id, &revenue);
        tc.transfer(&escrow, &referrer_address, &referrer);
        env.events().publish(
            (symbol_short!("completed"), campaign_id),
            (c.creator, total, creator, service, nft, revenue, referrer),
        );
        Ok(())
    }

    /// Callable by anyone, including while paused.  Refund destination is
    /// always the original sponsor stored at funding.
    pub fn refund_expired(env: Env, campaign_id: u64) -> Result<(), EscrowError> {
        Self::init(&env)?;
        let key = DataKey::Campaign(campaign_id);
        let mut c = Self::campaign(&env, campaign_id)?;
        if c.status != CampaignStatus::Active && c.status != CampaignStatus::Funded {
            return Err(EscrowError::InvalidState);
        }
        if env.ledger().timestamp() < c.expires_at {
            return Err(EscrowError::NotExpired);
        }
        c.status = CampaignStatus::Refunded;
        let liability: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalLiability)
            .unwrap_or(0);
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let tc = token::Client::new(&env, &token);
        let escrow = env.current_contract_address();
        if liability < c.amount || tc.balance(&escrow) < liability {
            return Err(EscrowError::Deficit);
        }
        env.storage()
            .instance()
            .set(&DataKey::TotalLiability, &(liability - c.amount));
        env.storage().persistent().set(&key, &c);
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        tc.transfer(&escrow, &c.sponsor, &c.amount);
        env.events().publish(
            (symbol_short!("refunded"), campaign_id),
            (c.sponsor, c.amount),
        );
        Ok(())
    }

    pub fn propose_governance(env: Env, account: Address) -> Result<(), EscrowError> {
        Self::propose(&env, 1, account)
    }
    pub fn accept_governance(env: Env) -> Result<(), EscrowError> {
        Self::accept(&env, 1)
    }
    pub fn propose_settlement(env: Env, account: Address) -> Result<(), EscrowError> {
        Self::propose(&env, 2, account)
    }
    pub fn accept_settlement(env: Env) -> Result<(), EscrowError> {
        Self::accept(&env, 2)
    }

    pub fn pause(env: Env) -> Result<(), EscrowError> {
        Self::init(&env)?;
        let g = Self::current_governance(&env)?;
        g.require_auth();
        env.storage().instance().extend_ttl(TTL, TTL);
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events().publish((symbol_short!("paused"),), ());
        Ok(())
    }
    pub fn unpause(env: Env) -> Result<(), EscrowError> {
        Self::init(&env)?;
        let g = Self::current_governance(&env)?;
        g.require_auth();
        env.storage().instance().extend_ttl(TTL, TTL);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events().publish((symbol_short!("unpaused"),), ());
        Ok(())
    }
    pub fn get_campaign(env: Env, id: u64) -> Result<Campaign, EscrowError> {
        let c = Self::campaign(&env, id)?;
        let key = DataKey::Campaign(id);
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        Ok(c)
    }
    pub fn config_version(env: Env) -> Result<u32, EscrowError> {
        Self::init(&env)?;
        env.storage().instance().extend_ttl(TTL, TTL);
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::ConfigVersion)
            .unwrap_or(1))
    }
    /// Immutable destinations committed at initialization. Off-chain
    /// accounting must read this state before treating a completion as a
    /// contributor-pool credit.
    pub fn pool_destinations(env: Env) -> Result<(Address, Address), EscrowError> {
        Self::init(&env)?;
        env.storage().instance().extend_ttl(TTL, TTL);
        Ok((
            env.storage().instance().get(&DataKey::NftPool).unwrap(),
            env.storage().instance().get(&DataKey::RevenuePool).unwrap(),
        ))
    }
    pub fn token_contract(env: Env) -> Result<Address, EscrowError> {
        Self::init(&env)?;
        env.storage().instance().extend_ttl(TTL, TTL);
        Ok(env.storage().instance().get(&DataKey::Token).unwrap())
    }
    pub fn is_paused(env: Env) -> bool {
        Self::paused(&env)
    }
    pub fn governance(env: Env) -> Result<Address, EscrowError> {
        Self::current_governance(&env)
    }
    pub fn settlement_authority(env: Env) -> Result<Address, EscrowError> {
        Self::init(&env)?;
        env.storage().instance().extend_ttl(TTL, TTL);
        Ok(env.storage().instance().get(&DataKey::Settlement).unwrap())
    }
    pub fn extend_instance_ttl(env: Env) -> Result<(), EscrowError> {
        Self::init(&env)?;
        env.storage().instance().extend_ttl(TTL, TTL);
        Ok(())
    }
    pub fn extend_campaign_ttl(env: Env, id: u64) -> Result<(), EscrowError> {
        let key = DataKey::Campaign(id);
        Self::campaign(&env, id)?;
        env.storage().persistent().extend_ttl(&key, TTL, TTL);
        Ok(())
    }

    fn propose(env: &Env, role: u32, account: Address) -> Result<(), EscrowError> {
        Self::init(env)?;
        let g = Self::current_governance(env)?;
        g.require_auth();
        let key = if role == 1 {
            DataKey::PendingGovernance
        } else {
            DataKey::PendingSettlement
        };
        env.storage().instance().set(&key, &account);
        env.events()
            .publish((symbol_short!("role_prop"), role), account);
        Ok(())
    }
    fn accept(env: &Env, role: u32) -> Result<(), EscrowError> {
        Self::init(env)?;
        let key = if role == 1 {
            DataKey::PendingGovernance
        } else {
            DataKey::PendingSettlement
        };
        let next: Address = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(EscrowError::NoPendingRole)?;
        next.require_auth();
        let dest = if role == 1 {
            DataKey::Governance
        } else {
            DataKey::Settlement
        };
        env.storage().instance().set(&dest, &next);
        env.storage().instance().remove(&key);
        env.events()
            .publish((symbol_short!("role_acc"), role), next);
        Ok(())
    }
    fn init(env: &Env) -> Result<(), EscrowError> {
        if env.storage().instance().has(&DataKey::Governance) {
            env.storage().instance().extend_ttl(TTL, TTL);
            Ok(())
        } else {
            Err(EscrowError::NotInitialized)
        }
    }
    fn current_governance(env: &Env) -> Result<Address, EscrowError> {
        env.storage()
            .instance()
            .get(&DataKey::Governance)
            .ok_or(EscrowError::NotInitialized)
    }

    fn authorize_pool_credit(env: &Env, pool: &Address, campaign_id: u64, amount: i128) {
        let mut args = Vec::<Val>::new(env);
        args.push_back(campaign_id.into_val(env));
        args.push_back(amount.into_val(env));
        let mut authorizations = Vec::new(env);
        authorizations.push_back(InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: pool.clone(),
                fn_name: Symbol::new(env, "record_credit"),
                args,
            },
            sub_invocations: Vec::new(env),
        }));
        env.authorize_as_current_contract(authorizations);
    }
    fn campaign(env: &Env, id: u64) -> Result<Campaign, EscrowError> {
        env.storage()
            .persistent()
            .get(&DataKey::Campaign(id))
            .ok_or(EscrowError::CampaignNotFound)
    }
    fn paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }
}

mod test;
