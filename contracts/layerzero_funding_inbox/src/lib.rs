#![no_std]

//! Isolated, value-receiving LayerZero compose adapter.
//!
//! This contract deliberately does not call CampaignEscrow.  A compose
//! delivery can prove that the configured OFT and source path authorized an
//! instruction, but it cannot prove that a person presenting a Soroban
//! address is the sponsor who funded the source-chain operation.  Funds are
//! therefore held as a liability and may only leave through the sponsor
//! authorized `claim`, `cancel`, or permissionless expired release to that
//! same sponsor. The
//! sponsor can then call CampaignEscrow::fund in a separate native Soroban
//! transaction.

use common_macros::contract_impl;
use endpoint_v2::{ILayerZeroComposer, MessagingComposerClient};
use soroban_sdk::{
    assert_with_error, contracterror, contractevent, contractimpl, contracttype, token, Address,
    Bytes, BytesN, Env, TryFromVal, Val,
};
use utils::{buffer_reader::BufferReader, buffer_writer::BufferWriter};

pub const DOMAIN: &[u8; 4] = b"TFZF";
pub const VERSION: u8 = 1;
pub const MSG_TYPE_FUNDING: u8 = 1;
pub const MAX_SPONSOR_REFERENCE: u32 = 256;
// At the conservative five-second ledger cadence this is about 40 days,
// enough to cover the normal funding deadline window. A sponsor/keeper can
// restore and touch an archived intent before claiming or releasing it.
pub const TTL: u32 = 700_000;

#[contracterror]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FundingError {
    Malformed = 1,
    WrongDomain = 2,
    UnsupportedVersion = 3,
    UnsupportedMessageType = 4,
    WrongOft = 5,
    WrongSourceEid = 6,
    WrongPeer = 7,
    WrongAssetDomain = 8,
    WrongEscrowDomain = 9,
    UnsupportedAsset = 10,
    InvalidAmount = 11,
    Expired = 12,
    InsufficientReceived = 13,
    DuplicateMessage = 14,
    DuplicateIntent = 15,
    BalanceDeficit = 16,
    NotFound = 17,
    InvalidState = 18,
    NotSponsor = 19,
    NotRecoverable = 20,
    InvalidReference = 21,
    Paused = 22,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct FundingInstruction {
    pub campaign_id: u64,
    pub operation_version: u32,
    pub sponsor: Address,
    pub sponsor_reference: Bytes,
    pub minimum_destination_amount: i128,
    pub deadline: u64,
    pub asset_domain: BytesN<32>,
    pub escrow_asset_domain: BytesN<32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum IntentStatus {
    Pending,
    Claimed,
    Cancelled,
    ExpiredReleased,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct FundingIntent {
    pub instruction: FundingInstruction,
    pub received_amount: i128,
    pub status: IntentStatus,
    pub guid: BytesN<32>,
    pub compose_index: u32,
    pub source_nonce: u64,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Endpoint,
    Oft,
    Token,
    SourceEid,
    SourcePeer,
    AssetDomain,
    EscrowAssetDomain,
    HeldBalance,
    Admin,
    Paused,
    Intent(u64, u32),
    Message(BytesN<32>, u32),
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundingReceived {
    pub campaign_id: u64,
    pub operation_version: u32,
    pub amount: i128,
    pub guid: BytesN<32>,
    pub source_nonce: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundingReleased {
    pub campaign_id: u64,
    pub operation_version: u32,
    pub amount: i128,
    pub recipient: Address,
    pub status: IntentStatus,
}

/// A compose-only inbox.  It is not an OFT and has no settlement authority.
#[soroban_sdk::contract]
pub struct FundingInbox;

fn instruction_key(instruction: &FundingInstruction) -> DataKey {
    DataKey::Intent(instruction.campaign_id, instruction.operation_version)
}

/// Encodes only the custom OFT compose payload. The OFT envelope is produced
/// by the official OFT and is intentionally not repeated here.
fn encode(env: &Env, instruction: &FundingInstruction) -> Bytes {
    assert_with_error!(
        env,
        instruction.sponsor_reference.len() <= MAX_SPONSOR_REFERENCE,
        FundingError::InvalidReference
    );
    assert_with_error!(
        env,
        instruction.minimum_destination_amount > 0,
        FundingError::InvalidAmount
    );
    assert_with_error!(
        env,
        instruction.operation_version > 0,
        FundingError::InvalidAmount
    );
    let mut writer = BufferWriter::new(env);
    writer
        .write_array(DOMAIN)
        .write_u8(VERSION)
        .write_u8(MSG_TYPE_FUNDING)
        .write_u64(instruction.campaign_id)
        .write_u32(instruction.operation_version)
        .write_address(&instruction.sponsor)
        .write_u32(instruction.sponsor_reference.len())
        .write_bytes(&instruction.sponsor_reference)
        .write_i128(instruction.minimum_destination_amount)
        .write_u64(instruction.deadline)
        .write_bytes_n(&instruction.asset_domain)
        .write_bytes_n(&instruction.escrow_asset_domain);
    writer.to_bytes()
}

fn decode(env: &Env, message: &Bytes) -> FundingInstruction {
    let mut reader = BufferReader::new(message);
    assert_with_error!(
        env,
        reader.read_bytes(4) == Bytes::from_array(env, DOMAIN),
        FundingError::WrongDomain
    );
    assert_with_error!(
        env,
        reader.read_u8() == VERSION,
        FundingError::UnsupportedVersion
    );
    assert_with_error!(
        env,
        reader.read_u8() == MSG_TYPE_FUNDING,
        FundingError::UnsupportedMessageType
    );
    let campaign_id = reader.read_u64();
    let operation_version = reader.read_u32();
    let sponsor = reader.read_address();
    let reference_len = reader.read_u32();
    assert_with_error!(
        env,
        reference_len <= MAX_SPONSOR_REFERENCE,
        FundingError::InvalidReference
    );
    let sponsor_reference = reader.read_bytes(reference_len);
    let minimum_destination_amount = reader.read_i128();
    let deadline = reader.read_u64();
    let asset_domain = reader.read_bytes_n();
    let escrow_asset_domain = reader.read_bytes_n();
    assert_with_error!(
        env,
        reader.position() == message.len(),
        FundingError::Malformed
    );
    let instruction = FundingInstruction {
        campaign_id,
        operation_version,
        sponsor,
        sponsor_reference,
        minimum_destination_amount,
        deadline,
        asset_domain,
        escrow_asset_domain,
    };
    assert_with_error!(
        env,
        instruction.minimum_destination_amount > 0,
        FundingError::InvalidAmount
    );
    assert_with_error!(
        env,
        instruction.operation_version > 0,
        FundingError::InvalidAmount
    );
    instruction
}

/// Official OFTComposeMsg encoding: nonce:u64, srcEid:u32, amountLD:i128,
/// composeFrom:bytes32, followed by the application compose message. All
/// integer fields are big-endian.
fn decode_oft_compose(env: &Env, message: &Bytes) -> (u64, u32, i128, BytesN<32>, Bytes) {
    let mut reader = BufferReader::new(message);
    let nonce = reader.read_u64();
    let src_eid = reader.read_u32();
    let amount_ld = reader.read_i128();
    let compose_from = reader.read_bytes_n();
    let compose_msg = reader.read_bytes_until_end();
    assert_with_error!(env, !compose_msg.is_empty(), FundingError::Malformed);
    (nonce, src_eid, amount_ld, compose_from, compose_msg)
}

fn encode_oft_compose(
    env: &Env,
    nonce: u64,
    src_eid: u32,
    amount_ld: i128,
    compose_from: &BytesN<32>,
    compose_msg: &Bytes,
) -> Bytes {
    let mut writer = BufferWriter::new(env);
    writer
        .write_u64(nonce)
        .write_u32(src_eid)
        .write_i128(amount_ld)
        .write_bytes_n(compose_from)
        .write_bytes(compose_msg);
    writer.to_bytes()
}

fn get<V: TryFromVal<Env, Val>>(env: &Env, key: &DataKey) -> V {
    env.storage().instance().get(key).unwrap()
}

impl FundingInbox {
    fn bump_instance_ttl(env: &Env) {
        env.storage().instance().extend_ttl(TTL, TTL);
    }

    fn extend_persistent_ttl(env: &Env, key: &DataKey) {
        env.storage().persistent().extend_ttl(key, TTL, TTL);
    }

    fn is_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    fn intent(
        env: &Env,
        campaign_id: u64,
        operation_version: u32,
    ) -> Result<FundingIntent, FundingError> {
        let key = DataKey::Intent(campaign_id, operation_version);
        let intent = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(FundingError::NotFound)?;
        Self::extend_persistent_ttl(env, &key);
        Ok(intent)
    }

    fn release(
        env: &Env,
        mut intent: FundingIntent,
        recipient: &Address,
        status: IntentStatus,
    ) -> Result<(), FundingError> {
        let amount = intent.received_amount;
        let held: i128 = env
            .storage()
            .instance()
            .get(&DataKey::HeldBalance)
            .unwrap_or(0);
        if held < amount {
            return Err(FundingError::BalanceDeficit);
        }
        let token_address: Address = get(env, &DataKey::Token);
        let token_client = token::Client::new(env, &token_address);
        let inbox = env.current_contract_address();
        if token_client.balance(&inbox) < held {
            return Err(FundingError::BalanceDeficit);
        }
        intent.status = status.clone();
        let intent_key = instruction_key(&intent.instruction);
        env.storage().persistent().set(&intent_key, &intent);
        Self::extend_persistent_ttl(env, &intent_key);
        env.storage()
            .instance()
            .set(&DataKey::HeldBalance, &(held - amount));
        token_client.transfer(&inbox, recipient, &amount);
        FundingReleased {
            campaign_id: intent.instruction.campaign_id,
            operation_version: intent.instruction.operation_version,
            amount,
            recipient: recipient.clone(),
            status,
        }
        .publish(env);
        Ok(())
    }
}

#[contract_impl]
impl ILayerZeroComposer for FundingInbox {
    /// The official LayerZero composer interface. The executor is authenticated
    /// as the actual executor (not as the endpoint). The endpoint compose
    /// queue is cleared only after route, envelope, amount, and replay checks.
    fn lz_compose(
        env: &Env,
        executor: &Address,
        from: &Address,
        guid: &BytesN<32>,
        index: u32,
        message: &Bytes,
        _extra_data: &Bytes,
        value: i128,
    ) {
        Self::bump_instance_ttl(env);
        executor.require_auth();
        assert_with_error!(env, !Self::is_paused(env), FundingError::Paused);
        assert_with_error!(env, value == 0, FundingError::InvalidAmount);
        let oft: Address = get(env, &DataKey::Oft);
        assert_with_error!(env, *from == oft, FundingError::WrongOft);

        let (nonce, src_eid, amount_ld, compose_from, compose_msg) =
            decode_oft_compose(env, message);
        let configured_eid: u32 = get(env, &DataKey::SourceEid);
        let configured_peer: BytesN<32> = get(env, &DataKey::SourcePeer);
        let configured_asset: BytesN<32> = get(env, &DataKey::AssetDomain);
        let configured_escrow_asset: BytesN<32> = get(env, &DataKey::EscrowAssetDomain);
        assert_with_error!(env, src_eid == configured_eid, FundingError::WrongSourceEid);
        assert_with_error!(
            env,
            compose_from == configured_peer,
            FundingError::WrongPeer
        );
        let instruction = decode(env, &compose_msg);
        assert_with_error!(
            env,
            instruction.asset_domain == configured_asset,
            FundingError::WrongAssetDomain
        );
        assert_with_error!(
            env,
            instruction.escrow_asset_domain == configured_escrow_asset,
            FundingError::WrongEscrowDomain
        );
        let message_key = DataKey::Message(guid.clone(), index);
        let message_seen = env.storage().persistent().has(&message_key);
        if message_seen {
            Self::extend_persistent_ttl(env, &message_key);
        }
        assert_with_error!(env, !message_seen, FundingError::DuplicateMessage);
        let intent_key = instruction_key(&instruction);
        let intent_seen = env.storage().persistent().has(&intent_key);
        if intent_seen {
            Self::extend_persistent_ttl(env, &intent_key);
        }
        assert_with_error!(env, !intent_seen, FundingError::DuplicateIntent);
        assert_with_error!(
            env,
            env.ledger().timestamp() <= instruction.deadline,
            FundingError::Expired
        );

        // The configured OFT, the authenticated executor, the exact source
        // peer, and the Endpoint compose queue are the delivery provenance
        // trust root.  The official Stellar composer API exposes no
        // per-GUID token credit or transfer hook, so a fungible-token balance
        // cannot distinguish an OFT transfer from an ambient transfer.
        let token_address: Address = get(env, &DataKey::Token);
        let token_client = token::Client::new(env, &token_address);
        let inbox = env.current_contract_address();
        let balance = token_client.balance(&inbox);
        assert_with_error!(env, amount_ld > 0, FundingError::InvalidAmount);
        let held: i128 = env
            .storage()
            .instance()
            .get(&DataKey::HeldBalance)
            .unwrap_or(0);
        let required = held.checked_add(amount_ld);
        assert_with_error!(env, required.is_some(), FundingError::InvalidAmount);
        let required = required.unwrap();
        // Defense in depth: authenticated OFT accounting cannot create a
        // liability that exceeds the configured token balance.
        assert_with_error!(env, balance >= required, FundingError::InsufficientReceived);
        assert_with_error!(
            env,
            amount_ld >= instruction.minimum_destination_amount,
            FundingError::InsufficientReceived
        );
        let next_held = required;

        let intent = FundingIntent {
            instruction: instruction.clone(),
            received_amount: amount_ld,
            status: IntentStatus::Pending,
            guid: guid.clone(),
            compose_index: index,
            source_nonce: nonce,
        };
        // Official pull-mode acknowledgement. This verifies the endpoint's
        // queued hash using the full wrapped envelope and marks it consumed.
        let endpoint: Address = get(env, &DataKey::Endpoint);
        MessagingComposerClient::new(env, &endpoint).clear_compose(
            &env.current_contract_address(),
            from,
            guid,
            &index,
            message,
        );
        env.storage().persistent().set(&message_key, &true);
        Self::extend_persistent_ttl(env, &message_key);
        env.storage().persistent().set(&intent_key, &intent);
        Self::extend_persistent_ttl(env, &intent_key);
        env.storage()
            .instance()
            .set(&DataKey::HeldBalance, &next_held);
        FundingReceived {
            campaign_id: instruction.campaign_id,
            operation_version: instruction.operation_version,
            amount: amount_ld,
            guid: guid.clone(),
            source_nonce: nonce,
        }
        .publish(env);
    }
}

#[contractimpl]
impl FundingInbox {
    pub fn __constructor(
        env: &Env,
        admin: &Address,
        endpoint: &Address,
        oft: &Address,
        token: &Address,
        source_eid: u32,
        source_peer: &BytesN<32>,
        asset_domain: &BytesN<32>,
        escrow_asset_domain: &BytesN<32>,
    ) {
        Self::bump_instance_ttl(env);
        env.storage().instance().set(&DataKey::Admin, admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::Endpoint, endpoint);
        env.storage().instance().set(&DataKey::Oft, oft);
        env.storage().instance().set(&DataKey::Token, token);
        env.storage()
            .instance()
            .set(&DataKey::SourceEid, &source_eid);
        env.storage()
            .instance()
            .set(&DataKey::SourcePeer, source_peer);
        env.storage()
            .instance()
            .set(&DataKey::AssetDomain, asset_domain);
        env.storage()
            .instance()
            .set(&DataKey::EscrowAssetDomain, escrow_asset_domain);
        env.storage().instance().set(&DataKey::HeldBalance, &0i128);
        Self::bump_instance_ttl(env);
    }

    pub fn encode_instruction(env: &Env, instruction: &FundingInstruction) -> Bytes {
        Self::bump_instance_ttl(env);
        encode(env, instruction)
    }

    /// Test/tooling helper for the official OFTComposeMsg envelope. Production
    /// callers should pass the envelope emitted by the configured OFT.
    pub fn encode_oft_compose_message(
        env: &Env,
        nonce: u64,
        src_eid: u32,
        amount_ld: i128,
        compose_from: &BytesN<32>,
        compose_msg: &Bytes,
    ) -> Bytes {
        Self::bump_instance_ttl(env);
        encode_oft_compose(env, nonce, src_eid, amount_ld, compose_from, compose_msg)
    }

    pub fn decode_instruction(env: &Env, message: &Bytes) -> FundingInstruction {
        Self::bump_instance_ttl(env);
        decode(env, message)
    }

    /// Releases a pending bridge liability to the exact sponsor in the
    /// authenticated instruction.  The reference is repeated to make an
    /// accidental claim by a different operation impossible.
    pub fn claim(
        env: &Env,
        campaign_id: u64,
        operation_version: u32,
        sponsor: &Address,
        sponsor_reference: &Bytes,
    ) -> Result<(), FundingError> {
        Self::bump_instance_ttl(env);
        let intent = Self::intent(env, campaign_id, operation_version)?;
        if Self::is_paused(env) {
            return Err(FundingError::Paused);
        }
        if intent.status != IntentStatus::Pending {
            return Err(FundingError::InvalidState);
        }
        if env.ledger().timestamp() > intent.instruction.deadline {
            return Err(FundingError::Expired);
        }
        if intent.instruction.sponsor != *sponsor
            || intent.instruction.sponsor_reference != *sponsor_reference
        {
            return Err(FundingError::NotSponsor);
        }
        sponsor.require_auth();
        Self::release(env, intent, sponsor, IntentStatus::Claimed)
    }

    /// A sponsor may cancel before expiry and recover only that intent's
    /// unconsumed locally held amount.  This is never an escrow refund.
    pub fn cancel(
        env: &Env,
        campaign_id: u64,
        operation_version: u32,
        sponsor: &Address,
        sponsor_reference: &Bytes,
    ) -> Result<(), FundingError> {
        Self::bump_instance_ttl(env);
        let intent = Self::intent(env, campaign_id, operation_version)?;
        if intent.status != IntentStatus::Pending {
            return Err(FundingError::InvalidState);
        }
        if intent.instruction.sponsor != *sponsor
            || intent.instruction.sponsor_reference != *sponsor_reference
        {
            return Err(FundingError::NotSponsor);
        }
        sponsor.require_auth();
        Self::release(env, intent, sponsor, IntentStatus::Cancelled)
    }

    /// After an intent expires, anyone may release its remaining amount to the
    /// exact sponsor committed in that intent. No owner or rescue destination
    /// can confiscate sponsor liabilities.
    pub fn release_expired(
        env: &Env,
        campaign_id: u64,
        operation_version: u32,
    ) -> Result<(), FundingError> {
        Self::bump_instance_ttl(env);
        let intent = Self::intent(env, campaign_id, operation_version)?;
        if intent.status != IntentStatus::Pending {
            return Err(FundingError::InvalidState);
        }
        if env.ledger().timestamp() <= intent.instruction.deadline {
            return Err(FundingError::NotRecoverable);
        }
        let sponsor = intent.instruction.sponsor.clone();
        Self::release(env, intent, &sponsor, IntentStatus::ExpiredReleased)
    }

    pub fn get_intent(
        env: &Env,
        campaign_id: u64,
        operation_version: u32,
    ) -> Result<FundingIntent, FundingError> {
        Self::bump_instance_ttl(env);
        Self::intent(env, campaign_id, operation_version)
    }

    /// Restores and refreshes an archived intent for its committed sponsor.
    ///
    /// If an intent has fallen below the network archival threshold, the
    /// sponsor must submit a transaction with that intent key in the
    /// RestoreFootprint before calling this method. This call then verifies
    /// the sponsor and reference and extends the persistent entry, after
    /// which `claim`, `cancel`, or `release_expired` can be used normally.
    pub fn restore_intent(
        env: &Env,
        campaign_id: u64,
        operation_version: u32,
        sponsor: &Address,
        sponsor_reference: &Bytes,
    ) -> Result<FundingIntent, FundingError> {
        Self::bump_instance_ttl(env);
        sponsor.require_auth();
        let key = DataKey::Intent(campaign_id, operation_version);
        let intent: FundingIntent = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(FundingError::NotFound)?;
        if intent.instruction.sponsor != *sponsor
            || intent.instruction.sponsor_reference != *sponsor_reference
        {
            return Err(FundingError::NotSponsor);
        }
        Self::extend_persistent_ttl(env, &key);
        Ok(intent)
    }

    pub fn held_balance(env: &Env) -> i128 {
        Self::bump_instance_ttl(env);
        env.storage()
            .instance()
            .get(&DataKey::HeldBalance)
            .unwrap_or(0)
    }

    pub fn paused(env: &Env) -> bool {
        Self::bump_instance_ttl(env);
        Self::is_paused(env)
    }

    pub fn admin(env: &Env) -> Address {
        Self::bump_instance_ttl(env);
        get(env, &DataKey::Admin)
    }

    pub fn pause(env: &Env) {
        Self::bump_instance_ttl(env);
        let admin: Address = get(env, &DataKey::Admin);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
    }

    pub fn unpause(env: &Env) {
        Self::bump_instance_ttl(env);
        let admin: Address = get(env, &DataKey::Admin);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    pub fn configured_token(env: &Env) -> Address {
        Self::bump_instance_ttl(env);
        get(env, &DataKey::Token)
    }
}

#[cfg(test)]
mod tests;
