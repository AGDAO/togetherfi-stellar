//! Native integration tests: real escrow, two real pools and the built-in SAC.
//! Authorization is scoped to exact invocations; no blanket auth is used.
extern crate std;

use super::*;
use contributor_pool::{ContributorPool, ContributorPoolClient, DataKey as PoolKey};
use soroban_sdk::{
    testutils::{Address as _, IssuerFlags, Ledger, MockAuth, MockAuthInvoke},
    token::{StellarAssetClient, TokenClient},
};
use std::vec::Vec as NativeVec;

fn authorize(env: &Env, contract: &Address, name: &str, args: Vec<Val>, signers: &[&Address]) {
    let invoke = MockAuthInvoke {
        contract,
        fn_name: name,
        args,
        sub_invokes: &[],
    };
    let auth: NativeVec<_> = signers
        .iter()
        .map(|address| MockAuth { address, invoke: &invoke })
        .collect();
    env.mock_auths(&auth);
}

struct Rig {
    env: Env,
    escrow: Address,
    pools: [Address; 2],
    token: Address,
    admin: Address,
    deployer: Address,
    governance: Address,
    eligibility: Address,
    settlement: Address,
    sponsor: Address,
    creator: Address,
    service: Address,
    referrer: Address,
    outsider: Address,
}

impl Rig {
    fn new() -> Self {
        let env = Env::default();
        env.ledger().with_mut(|l| l.timestamp = 1_000);
        let admin = Address::generate(&env);
        let deployer = Address::generate(&env);
        let governance = Address::generate(&env);
        let eligibility = Address::generate(&env);
        let settlement = Address::generate(&env);
        let sponsor = Address::generate(&env);
        let creator = Address::generate(&env);
        let service = Address::generate(&env);
        let referrer = Address::generate(&env);
        let outsider = Address::generate(&env);
        let asset = env.register_stellar_asset_contract_v2(admin.clone());
        // These issuer capabilities allow genuine nested SAC failures, without
        // changing contract storage or substituting a failing mock contract.
        asset.issuer().set_flag(IssuerFlags::AuthRevocableFlag);
        asset.issuer().set_flag(IssuerFlags::AuthClawbackEnabledFlag);
        let token = asset.address();
        let escrow_salt = BytesN::from_array(&env, &[71; 32]);
        let escrow = env.deployer()
            .with_address(deployer.clone(), escrow_salt.clone()).deployed_address();
        env.register_contract(Some(&escrow), CampaignEscrow);
        let salts = [
            BytesN::from_array(&env, &[72; 32]),
            BytesN::from_array(&env, &[73; 32]),
        ];
        let pools = salts.clone().map(|salt| {
            let id = env.deployer().with_address(deployer.clone(), salt.clone()).deployed_address();
            env.register_contract(Some(&id), ContributorPool);
            authorize(
                &env, &id, "initialize",
                (governance.clone(), eligibility.clone(), token.clone(), escrow.clone(),
                 deployer.clone(), salt.clone()).into_val(&env),
                &[&deployer, &governance, &eligibility],
            );
            ContributorPoolClient::new(&env, &id).initialize(
                &governance, &eligibility, &token, &escrow, &deployer, &salt,
            );
            id
        });
        authorize(
            &env, &escrow, "initialize",
            (governance.clone(), settlement.clone(), token.clone(), service.clone(),
             pools[0].clone(), pools[1].clone(), deployer.clone(), escrow_salt.clone()).into_val(&env),
            &[&deployer, &governance],
        );
        CampaignEscrowClient::new(&env, &escrow).initialize(
            &governance, &settlement, &token, &service,
            &pools[0], &pools[1], &deployer, &escrow_salt,
        );
        let rig = Self {
            env, escrow, pools, token, admin, deployer, governance, eligibility,
            settlement, sponsor, creator, service, referrer, outsider,
        };
        rig.mint(&rig.sponsor, 1_000_000);
        rig.env.mock_auths(&[]);
        rig
    }

    fn escrow(&self) -> CampaignEscrowClient<'_> {
        CampaignEscrowClient::new(&self.env, &self.escrow)
    }

    fn pool(&self, index: usize) -> ContributorPoolClient<'_> {
        ContributorPoolClient::new(&self.env, &self.pools[index])
    }

    fn tokens(&self) -> TokenClient<'_> {
        TokenClient::new(&self.env, &self.token)
    }

    fn mint(&self, to: &Address, amount: i128) {
        authorize(&self.env, &self.token, "mint",
            (to.clone(), amount).into_val(&self.env), &[&self.admin]);
        StellarAssetClient::new(&self.env, &self.token).mint(to, &amount);
    }

    fn prepare(&self, id: u64, amount: i128) {
        // The sponsor must authorize both the escrow entry point and its
        // nested token transfer; a root-only authorization must not suffice.
        let transfer = MockAuthInvoke {
            contract: &self.token,
            fn_name: "transfer",
            args: (self.sponsor.clone(), self.escrow.clone(), amount).into_val(&self.env),
            sub_invokes: &[],
        };
        let invoke = MockAuthInvoke {
            contract: &self.escrow,
            fn_name: "fund",
            args: (id, self.sponsor.clone(), amount).into_val(&self.env),
            sub_invokes: &[transfer],
        };
        self.env.mock_auths(&[MockAuth { address: &self.sponsor, invoke: &invoke }]);
        self.escrow().fund(&id, &self.sponsor, &amount);
        let terms = Bytes::from_slice(&self.env, b"stellar-native-integration");
        authorize(&self.env, &self.escrow, "activate",
            (id, self.creator.clone(), Some(self.referrer.clone()), terms.clone()).into_val(&self.env),
            &[&self.sponsor]);
        self.escrow().activate(&id, &self.creator, &Some(self.referrer.clone()), &terms);
        self.env.mock_auths(&[]);
    }

    fn authorize_complete(&self, id: u64) {
        authorize(&self.env, &self.escrow, "complete",
            (id, 1_u32).into_val(&self.env), &[&self.settlement]);
    }

    fn complete(&self, id: u64) {
        self.authorize_complete(id);
        self.escrow().complete(&id, &1);
        self.env.mock_auths(&[]);
    }

    fn revoke(&self, who: &Address, revoked: bool) {
        authorize(&self.env, &self.token, "set_authorized",
            (who.clone(), !revoked).into_val(&self.env), &[&self.admin]);
        StellarAssetClient::new(&self.env, &self.token).set_authorized(who, &!revoked);
        self.env.mock_auths(&[]);
    }

    fn seed_reserved_liabilities(&self) {
        self.prepare(1, 10_000);
        self.complete(1);
        for (index, credited) in [(0, 500_i128), (1, 200_i128)] {
            let hash = BytesN::from_array(&self.env, &[81; 32]);
            let policy = BytesN::from_array(&self.env, &[82; 32]);
            let recipients = Vec::from_array(&self.env, [self.creator.clone(), self.referrer.clone()]);
            let amounts = Vec::from_array(&self.env, [credited * 2 / 5, credited * 3 / 5]);
            authorize(&self.env, &self.pools[index], "publish_manifest",
                (1_u64, hash.clone(), policy.clone(), 2_000_u64, recipients.clone(), amounts.clone())
                    .into_val(&self.env),
                &[&self.governance, &self.eligibility]);
            self.pool(index).publish_manifest(&1, &hash, &policy, &2_000, &recipients, &amounts);
            authorize(&self.env, &self.pools[index], "claim",
                (1_u64, 0_u32).into_val(&self.env), &[&self.creator]);
            self.pool(index).claim(&1, &0);
            assert_eq!(self.pool(index).total_liability(), credited * 3 / 5);
            assert_eq!(self.pool(index).available_balance(), 0);
        }
        self.env.mock_auths(&[]);
    }

    fn state(&self, ids: &[u64]) -> State {
        let addresses = [
            &self.escrow, &self.pools[0], &self.pools[1], &self.sponsor, &self.creator,
            &self.service, &self.referrer, &self.admin, &self.deployer,
            &self.governance, &self.eligibility, &self.settlement, &self.outsider,
        ];
        State {
            balances: addresses.iter().map(|a| self.tokens().balance(a)).collect(),
            liability: self.env.as_contract(&self.escrow,
                || self.env.storage().instance().get(&DataKey::TotalLiability).unwrap()),
            campaigns: ids.iter().map(|id| {
                let campaign = self.escrow().get_campaign(id);
                let consumed = self.env.as_contract(&self.escrow, ||
                    self.env.storage().persistent().has(&DataKey::ConsumedOperation(*id, 1)));
                (campaign, consumed)
            }).collect(),
            pools: self.pools.iter().map(|pool| self.env.as_contract(pool, || PoolState {
                liability: self.env.storage().instance().get(&PoolKey::TotalLiability).unwrap(),
                credited: self.env.storage().instance().get(&PoolKey::TotalCredited).unwrap(),
                paid: self.env.storage().instance().get(&PoolKey::TotalPaid).unwrap(),
                credits: ids.iter().map(|id|
                    self.env.storage().persistent().has(&PoolKey::CreditedCampaign(*id))).collect(),
            })).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct State {
    balances: NativeVec<i128>,
    liability: i128,
    campaigns: NativeVec<(Campaign, bool)>,
    pools: NativeVec<PoolState>,
}

#[derive(Debug, PartialEq, Eq)]
struct PoolState {
    liability: i128,
    credited: i128,
    paid: i128,
    credits: NativeVec<bool>,
}

#[test]
fn real_pools_reject_direct_credits_without_escrow_authorization() {
    let rig = Rig::new();
    rig.prepare(1, 10_000);
    rig.complete(1);
    let before = rig.state(&[1]);
    for pool in &rig.pools {
        let client = ContributorPoolClient::new(&rig.env, pool);
        rig.env.mock_auths(&[]);
        assert!(client.try_record_credit(&99, &1).is_err());
        for signer in [&rig.outsider, &rig.governance, &rig.settlement] {
            authorize(&rig.env, pool, "record_credit",
                (99_u64, 1_i128).into_val(&rig.env), &[signer]);
            assert!(client.try_record_credit(&99, &1).is_err());
        }
        assert!(!rig.env.as_contract(pool,
            || rig.env.storage().persistent().has(&PoolKey::CreditedCampaign(99))));
        assert_eq!(rig.state(&[1]), before);
    }
}

#[test]
fn real_pools_credit_each_campaign_once_and_preserve_fixed_rounding() {
    let rig = Rig::new();
    rig.prepare(1, 10_003);
    rig.complete(1);
    assert_eq!(rig.tokens().balance(&rig.creator), 8_203);
    assert_eq!(rig.tokens().balance(&rig.service), 1_000);
    assert_eq!(rig.tokens().balance(&rig.referrer), 100);
    assert_eq!(rig.tokens().balance(&rig.escrow), 0);
    for (index, amount) in [(0, 500_i128), (1, 200_i128)] {
        assert_eq!(rig.tokens().balance(&rig.pools[index]), amount);
        assert_eq!(rig.pool(index).available_balance(), amount);
    }
    let before = rig.state(&[1]);
    assert_eq!(before.liability, 0);
    assert_eq!(before.campaigns[0].0.status, CampaignStatus::Completed);
    assert!(before.campaigns[0].1);
    for pool in &before.pools {
        assert_eq!(pool.credits, std::vec![true]);
    }
    rig.authorize_complete(1);
    assert_eq!(rig.escrow().try_complete(&1, &1), Err(Ok(EscrowError::InvalidState)));
    assert_eq!(rig.state(&[1]), before);
    // A distinct campaign can still credit both pools normally.
    rig.prepare(2, 100);
    rig.complete(2);
    assert_eq!(rig.pool(0).available_balance(), 505);
    assert_eq!(rig.pool(1).available_balance(), 202);
    assert_eq!(rig.state(&[1, 2]).pools[0].credits, std::vec![true, true]);
    assert_eq!(rig.state(&[1, 2]).pools[1].credits, std::vec![true, true]);
}

#[test]
fn real_second_pool_credit_failure_rolls_back_all_earlier_payouts_and_credits() {
    let rig = Rig::new();
    rig.seed_reserved_liabilities();
    rig.prepare(2, 10_000);
    // A real issuer clawback leaves this pool one base unit short of its
    // existing liabilities. Its next record_credit fails with Deficit,
    // after the first pool's transfer AND credit have already executed.
    authorize(&rig.env, &rig.token, "clawback",
        (rig.pools[1].clone(), 1_i128).into_val(&rig.env), &[&rig.admin]);
    StellarAssetClient::new(&rig.env, &rig.token).clawback(&rig.pools[1], &1);
    let before = rig.state(&[1, 2]);
    assert_eq!(before.pools[0].liability, 300);
    assert_eq!(before.pools[1].liability, 120);
    rig.authorize_complete(2);
    assert!(rig.escrow().try_complete(&2, &1).is_err());
    assert_eq!(rig.state(&[1, 2]), before);
    assert_eq!(rig.escrow().get_campaign(&2).status, CampaignStatus::Active);
    // Repair only the original one-unit deficit: no duplicate payments or
    // manual marker resets are needed for the exact same settlement.
    rig.mint(&rig.pools[1], 1);
    rig.complete(2);
    let after = rig.state(&[1, 2]);
    assert_eq!(after.liability, 0);
    assert_eq!(after.pools[0].credited, 1_000);
    assert_eq!(after.pools[1].credited, 400);
    assert_eq!(after.pools[0].paid, 200);
    assert_eq!(after.pools[1].paid, 80);
    assert_eq!(after.pools[0].liability, 300);
    assert_eq!(after.pools[1].liability, 120);
}

#[test]
fn real_final_token_transfer_failure_rolls_back_both_pool_credits_and_all_balances() {
    let rig = Rig::new();
    rig.seed_reserved_liabilities();
    rig.prepare(2, 10_000);
    // The referrer is the final leg, after BOTH pool credits have succeeded.
    // Token issuer authorization, not a mock contract, rejects that transfer.
    rig.revoke(&rig.referrer, true);
    let before = rig.state(&[1, 2]);
    rig.authorize_complete(2);
    assert!(rig.escrow().try_complete(&2, &1).is_err());
    assert_eq!(rig.state(&[1, 2]), before);
    rig.revoke(&rig.referrer, false);
    rig.complete(2);
    let after = rig.state(&[1, 2]);
    for pool in &after.pools {
        assert_eq!(pool.credits, std::vec![true, true]);
    }
    assert_eq!(after.pools[0].credited, 1_000);
    assert_eq!(after.pools[1].credited, 400);
    assert_eq!(after.liability, 0);
}
