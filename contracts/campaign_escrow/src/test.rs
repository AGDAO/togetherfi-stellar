#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Env,
};

fn create_usdc_token<'a>(env: &Env, admin: &Address) -> (Address, TokenClient<'a>, StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract(admin.clone());
    (
        contract_address.clone(),
        TokenClient::new(env, &contract_address),
        StellarAssetClient::new(env, &contract_address),
    )
}

struct TestSetup<'a> {
    env: Env,
    escrow_id: Address,
    escrow: CampaignEscrowClient<'a>,
    usdc: TokenClient<'a>,
    usdc_admin: StellarAssetClient<'a>,
    admin: Address,
    platform_treasury: Address,
    revenue_pool: Address,
    default_referrer: Address,
    brand: Address,
    creator: Address,
}

fn setup<'a>() -> TestSetup<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let platform_treasury = Address::generate(&env);
    let revenue_pool = Address::generate(&env);
    let default_referrer = Address::generate(&env);
    let brand = Address::generate(&env);
    let creator = Address::generate(&env);

    let (usdc_id, usdc, usdc_admin) = create_usdc_token(&env, &token_admin);
    usdc_admin.mint(&brand, &1_000_000_000);

    let escrow_id = env.register_contract(None, CampaignEscrow);
    let escrow = CampaignEscrowClient::new(&env, &escrow_id);
    escrow.initialize(&admin, &usdc_id, &platform_treasury, &revenue_pool, &default_referrer);

    TestSetup {
        env,
        escrow_id,
        escrow,
        usdc,
        usdc_admin,
        admin,
        platform_treasury,
        revenue_pool,
        default_referrer,
        brand,
        creator,
    }
}

#[test]
fn fund_then_complete_splits_correctly() {
    let s = setup();
    let campaign_id: u64 = 1;
    let amount: i128 = 100_000_0000; // 100,000.0000 USDC (7 decimals)

    s.escrow.fund(&campaign_id, &s.brand, &amount);
    assert_eq!(s.usdc.balance(&s.escrow_id), amount);
    assert_eq!(s.usdc.balance(&s.brand), 1_000_000_000 - amount);

    s.escrow.complete(&campaign_id, &s.creator, &None);

    // 82.5% creator, 10% platform, 2.5% referrer folded into pool since no
    // distinct referrer was passed (referrer defaults to default_referrer,
    // which is NOT the pool here, so it should receive its own 2.5% share).
    let expected_creator = amount * 8_250 / 10_000;
    let expected_platform = amount * 1_000 / 10_000;
    let expected_referrer = amount * 250 / 10_000;
    let expected_pool = amount - expected_creator - expected_platform - expected_referrer;

    assert_eq!(s.usdc.balance(&s.creator), expected_creator);
    assert_eq!(s.usdc.balance(&s.platform_treasury), expected_platform);
    assert_eq!(s.usdc.balance(&s.default_referrer), expected_referrer);
    assert_eq!(s.usdc.balance(&s.revenue_pool), expected_pool);
    assert_eq!(s.usdc.balance(&s.escrow_id), 0);

    let campaign = s.escrow.get_campaign(&campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Completed);
}

#[test]
fn complete_with_explicit_referrer_pays_referrer_directly() {
    let s = setup();
    let campaign_id: u64 = 2;
    let amount: i128 = 50_000_0000;
    let referrer = Address::generate(&s.env);

    s.escrow.fund(&campaign_id, &s.brand, &amount);
    s.escrow.complete(&campaign_id, &s.creator, &Some(referrer.clone()));

    let expected_referrer = amount * 250 / 10_000;
    assert_eq!(s.usdc.balance(&referrer), expected_referrer);
    // default_referrer should NOT have received anything in this case.
    assert_eq!(s.usdc.balance(&s.default_referrer), 0);
}

#[test]
#[should_panic]
fn double_complete_fails() {
    let s = setup();
    let campaign_id: u64 = 3;
    let amount: i128 = 10_000_0000;

    s.escrow.fund(&campaign_id, &s.brand, &amount);
    s.escrow.complete(&campaign_id, &s.creator, &None);
    // Second call must panic: campaign status is no longer Funded.
    s.escrow.complete(&campaign_id, &s.creator, &None);
}

#[test]
fn refund_returns_full_amount_to_brand() {
    let s = setup();
    let campaign_id: u64 = 4;
    let amount: i128 = 20_000_0000;

    s.escrow.fund(&campaign_id, &s.brand, &amount);
    let brand_balance_after_fund = s.usdc.balance(&s.brand);

    s.escrow.refund(&campaign_id);

    assert_eq!(s.usdc.balance(&s.brand), brand_balance_after_fund + amount);
    assert_eq!(s.usdc.balance(&s.escrow_id), 0);

    let campaign = s.escrow.get_campaign(&campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Refunded);
}
