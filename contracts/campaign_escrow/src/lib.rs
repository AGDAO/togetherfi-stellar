//! TogetherFi Campaign Escrow — a real Soroban smart contract.
//!
//! This is NOT `manageData` account anchoring. It is a genuine Soroban contract
//! that takes custody of USDC for a Stellar campaign and releases it
//! automatically on completion with the same split TogetherFi already uses on
//! Arbitrum's RewardEscrowV3:
//!
//!   82.5% -> creator wallet
//!   10.0% -> platform treasury
//!    5.0% -> revenue share pool
//!    2.5% -> referrer wallet (falls back to the revenue share pool if there is no referrer)
//!
//! Lifecycle per campaign:
//!   1. `initialize`  — one-time, sets the admin (TogetherFi backend signer) and
//!                      the four payout wallets (platform treasury, revenue pool,
//!                      default referrer fallback, USDC token contract id).
//!   2. `fund`        — brand deposits USDC into the contract for a campaign_id.
//!                      Requires the brand's signature (`brand.require_auth()`).
//!                      Funds move brand -> contract. Campaign status: Funded.
//!   3. `complete`    — admin-only. Splits and pays out held funds in one atomic
//!                      transaction to creator / platform / revenue pool / referrer.
//!                      Campaign status: Completed. Can only run once per campaign
//!                      (guards against double payout).
//!   4. `refund`      — admin-only escape hatch: if a campaign is cancelled before
//!                      completion, the full held balance returns to the brand.
//!
//! All amounts are in the token's native stroops (USDC on Stellar uses 7 decimals
//! via its Stellar Asset Contract, i.e. amount units are 1e-7 USDC).

#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, log, token, Address, Env};

const BPS_DENOM: i128 = 10_000;
const CREATOR_BPS: i128 = 8_250; // 82.5%
const PLATFORM_BPS: i128 = 1_000; // 10.0%
#[allow(dead_code)]
const REVENUE_POOL_BPS: i128 = 500; // 5.0% (kept for documentation; revenue_amount is computed as the remainder so rounding dust never gets stuck)
const REFERRER_BPS: i128 = 250; // 2.5%

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    UsdcToken,
    PlatformTreasury,
    RevenuePool,
    DefaultReferrer,
    Campaign(u64),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CampaignStatus {
    Funded,
    Completed,
    Refunded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Campaign {
    pub brand: Address,
    pub amount: i128,
    pub status: CampaignStatus,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    CampaignAlreadyExists = 3,
    CampaignNotFound = 4,
    CampaignNotFunded = 5,
    InvalidAmount = 6,
}

#[contract]
pub struct CampaignEscrow;

#[contractimpl]
impl CampaignEscrow {
    /// One-time setup. `admin` is the TogetherFi backend signer that is allowed
    /// to call `complete` / `refund`. Wallet addresses are the same fixed
    /// destinations used by RewardEscrowV3 on Arbitrum, just Stellar accounts.
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc_token: Address,
        platform_treasury: Address,
        revenue_pool: Address,
        default_referrer: Address,
    ) -> Result<(), EscrowError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(EscrowError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage()
            .instance()
            .set(&DataKey::PlatformTreasury, &platform_treasury);
        env.storage().instance().set(&DataKey::RevenuePool, &revenue_pool);
        env.storage()
            .instance()
            .set(&DataKey::DefaultReferrer, &default_referrer);

        log!(&env, "campaign_escrow initialized, admin={}", admin);
        Ok(())
    }

    /// Brand funds a campaign. Transfers `amount` of USDC from `brand` to this
    /// contract. Requires the brand's own signature, the contract never moves
    /// funds out of a brand's wallet without their authorization.
    pub fn fund(env: Env, campaign_id: u64, brand: Address, amount: i128) -> Result<(), EscrowError> {
        Self::require_initialized(&env)?;
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Campaign(campaign_id))
        {
            return Err(EscrowError::CampaignAlreadyExists);
        }

        brand.require_auth();

        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&brand, &env.current_contract_address(), &amount);

        let campaign = Campaign {
            brand: brand.clone(),
            amount,
            status: CampaignStatus::Funded,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        log!(&env, "campaign {} funded by {} amount={}", campaign_id, brand, amount);
        Ok(())
    }

    /// Admin-only. Releases the held balance for `campaign_id` in the fixed
    /// 82.5 / 10 / 5 / 2.5 split. If `referrer` is None, the referrer's 2.5%
    /// share is redirected to the revenue share pool (7.5% total to the pool).
    /// Can only be called once per campaign; a second call fails with
    /// `CampaignNotFunded` because the status is no longer `Funded`.
    pub fn complete(
        env: Env,
        campaign_id: u64,
        creator: Address,
        referrer: Option<Address>,
    ) -> Result<(), EscrowError> {
        let admin = Self::require_initialized(&env)?;
        admin.require_auth();

        let mut campaign: Campaign = env
            .storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))
            .ok_or(EscrowError::CampaignNotFound)?;

        if campaign.status != CampaignStatus::Funded {
            return Err(EscrowError::CampaignNotFunded);
        }

        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token_client = token::Client::new(&env, &usdc_token);
        let platform_treasury: Address =
            env.storage().instance().get(&DataKey::PlatformTreasury).unwrap();
        let revenue_pool: Address = env.storage().instance().get(&DataKey::RevenuePool).unwrap();
        let default_referrer: Address =
            env.storage().instance().get(&DataKey::DefaultReferrer).unwrap();

        let total = campaign.amount;
        let creator_amount = total * CREATOR_BPS / BPS_DENOM;
        let platform_amount = total * PLATFORM_BPS / BPS_DENOM;
        let referrer_amount = total * REFERRER_BPS / BPS_DENOM;
        // Revenue pool gets its own 5% plus any dust from integer division, so
        // the four transfers always sum to exactly `total` with no funds stuck.
        let revenue_amount = total - creator_amount - platform_amount - referrer_amount;

        token_client.transfer(&env.current_contract_address(), &creator, &creator_amount);
        token_client.transfer(
            &env.current_contract_address(),
            &platform_treasury,
            &platform_amount,
        );

        let referrer_dest = referrer.unwrap_or(default_referrer);
        if referrer_dest == revenue_pool {
            // No distinct referrer configured: fold the referrer share into the pool.
            token_client.transfer(
                &env.current_contract_address(),
                &revenue_pool,
                &(revenue_amount + referrer_amount),
            );
        } else {
            token_client.transfer(&env.current_contract_address(), &referrer_dest, &referrer_amount);
            token_client.transfer(&env.current_contract_address(), &revenue_pool, &revenue_amount);
        }

        campaign.status = CampaignStatus::Completed;
        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        log!(
            &env,
            "campaign {} completed: creator={} platform={} revenue_pool_leg={} referrer={}",
            campaign_id,
            creator_amount,
            platform_amount,
            revenue_amount,
            referrer_amount
        );
        Ok(())
    }

    /// Admin-only escape hatch: returns the full held balance to the brand if a
    /// campaign is cancelled before completion.
    pub fn refund(env: Env, campaign_id: u64) -> Result<(), EscrowError> {
        let admin = Self::require_initialized(&env)?;
        admin.require_auth();

        let mut campaign: Campaign = env
            .storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))
            .ok_or(EscrowError::CampaignNotFound)?;

        if campaign.status != CampaignStatus::Funded {
            return Err(EscrowError::CampaignNotFunded);
        }

        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&env.current_contract_address(), &campaign.brand, &campaign.amount);

        campaign.status = CampaignStatus::Refunded;
        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        log!(&env, "campaign {} refunded to {}", campaign_id, campaign.brand);
        Ok(())
    }

    /// Read-only campaign lookup, used by the backend to verify state.
    pub fn get_campaign(env: Env, campaign_id: u64) -> Result<Campaign, EscrowError> {
        env.storage()
            .persistent()
            .get(&DataKey::Campaign(campaign_id))
            .ok_or(EscrowError::CampaignNotFound)
    }

    fn require_initialized(env: &Env) -> Result<Address, EscrowError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(EscrowError::NotInitialized)
    }
}

mod test;
