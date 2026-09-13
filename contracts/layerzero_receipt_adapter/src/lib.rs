#![no_std]

extern crate alloc;

use endpoint_v2::{MessagingFee, MessagingReceipt, Origin};
use oapp::{
    oapp_core::{endpoint_client, get_peer_or_panic},
    oapp_receiver::{LzReceiveInternal, OAppReceiver},
    oapp_sender::{FeePayer, OAppSenderInternal},
};
use oapp_macros::oapp;
use soroban_sdk::{
    assert_with_error, contracterror, contractevent, contractimpl, contracttype, Address, Bytes,
    BytesN, Env,
};
use utils::{buffer_reader::BufferReader, buffer_writer::BufferWriter, ownable::enforce_owner_auth};

/// The protocol/domain separator.  It prevents a receipt from this adapter
/// being accepted as an application command by a different protocol.
pub const DOMAIN: &[u8; 4] = b"TFZR";
pub const RECEIPT_VERSION: u8 = 1;
pub const MSG_TYPE_FINAL_STATE: u32 = 1;
pub const COMPLETE: u32 = 1;
pub const REFUNDED: u32 = 2;
pub const ANCHORED: u32 = 3;

const PAYLOAD_LEN: u32 = 4 + 1 + 1 + 8 + 4 + 32;

#[contracterror]
#[repr(u32)]
pub enum ReceiptError {
    Malformed = 1,
    UnsupportedStatus = 2,
    DuplicateCampaignOperation = 3,
    InvalidOperationVersion = 4,
    InvalidInboundNonce = 5,
    NonZeroInboundValue = 6,
    OnlyEndpoint = 7,
    OnlyOwner = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalStateReceipt {
    pub campaign_id: u64,
    pub operation_version: u32,
    pub status: u32,
    pub state_hash: BytesN<32>,
}

#[contracttype]
enum DataKey {
    Published(u64, u32),
    Received(u64, u32),
    InboundNonce(u32, BytesN<32>),
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalStatePublished {
    pub campaign_id: u64,
    pub operation_version: u32,
    pub status: u32,
    pub guid: BytesN<32>,
    pub nonce: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalStateReceived {
    pub src_eid: u32,
    pub campaign_id: u64,
    pub operation_version: u32,
    pub status: u32,
    pub guid: BytesN<32>,
    pub nonce: u64,
}

/// A receipt-only LayerZero OApp.  It never references the escrow, token,
/// payout, refund, contributor, or allocation contracts.
#[oapp(custom = [receiver])]
#[common_macros::lz_contract]
pub struct ReceiptAdapter;

fn assert_supported_status(env: &Env, status: u32) {
    assert_with_error!(
        env,
        status == COMPLETE || status == REFUNDED || status == ANCHORED,
        ReceiptError::UnsupportedStatus
    );
}

fn encode(env: &Env, receipt: &FinalStateReceipt) -> Bytes {
    assert_supported_status(env, receipt.status);
    assert_with_error!(env, receipt.operation_version > 0, ReceiptError::InvalidOperationVersion);
    let mut writer = BufferWriter::new(env);
    assert_with_error!(env, receipt.status <= u8::MAX as u32, ReceiptError::UnsupportedStatus);
    writer.write_array(DOMAIN).write_u8(RECEIPT_VERSION).write_u8(receipt.status as u8);
    writer.write_u64(receipt.campaign_id).write_u32(receipt.operation_version);
    writer.write_bytes_n(&receipt.state_hash);
    writer.to_bytes()
}

fn decode(env: &Env, message: &Bytes) -> FinalStateReceipt {
    assert_with_error!(env, message.len() == PAYLOAD_LEN, ReceiptError::Malformed);
    let mut reader = BufferReader::new(message);
    assert_with_error!(
        env,
        reader.read_bytes(4) == Bytes::from_array(env, DOMAIN),
        ReceiptError::Malformed
    );
    assert_with_error!(env, reader.read_u8() == RECEIPT_VERSION, ReceiptError::Malformed);
    let status = reader.read_u8() as u32;
    assert_supported_status(env, status);
    let receipt = FinalStateReceipt {
        campaign_id: reader.read_u64(),
        operation_version: reader.read_u32(),
        status,
        state_hash: reader.read_bytes_n(),
    };
    assert_with_error!(env, receipt.operation_version > 0, ReceiptError::InvalidOperationVersion);
    receipt
}

fn published_key(receipt: &FinalStateReceipt) -> DataKey {
    DataKey::Published(receipt.campaign_id, receipt.operation_version)
}

fn received_key(receipt: &FinalStateReceipt) -> DataKey {
    DataKey::Received(receipt.campaign_id, receipt.operation_version)
}

impl LzReceiveInternal for ReceiptAdapter {
    fn __lz_receive(
        env: &Env,
        origin: &Origin,
        guid: &BytesN<32>,
        message: &Bytes,
        _extra_data: &Bytes,
        _executor: &Address,
        _value: i128,
    ) {
        let receipt = decode(env, message);
        let nonce_key = DataKey::InboundNonce(origin.src_eid, origin.sender.clone());
        let prior_nonce: u64 = env.storage().persistent().get(&nonce_key).unwrap_or(0);
        assert_with_error!(env, origin.nonce == prior_nonce + 1, ReceiptError::InvalidInboundNonce);
        assert_with_error!(env, !env.storage().persistent().has(&received_key(&receipt)), ReceiptError::DuplicateCampaignOperation);

        // The message is informational only.  No inbound value is accepted,
        // and the sole state transition is receipt bookkeeping.
        env.storage().persistent().set(&nonce_key, &origin.nonce);
        env.storage().persistent().set(&received_key(&receipt), &true);
        FinalStateReceived {
            src_eid: origin.src_eid,
            campaign_id: receipt.campaign_id,
            operation_version: receipt.operation_version,
            status: receipt.status,
            guid: guid.clone(),
            nonce: origin.nonce,
        }
        .publish(env);
    }
}

#[contractimpl(contracttrait)]
impl OAppReceiver for ReceiptAdapter {
    fn lz_receive(
        env: &Env,
        executor: &Address,
        origin: &Origin,
        guid: &BytesN<32>,
        message: &Bytes,
        extra_data: &Bytes,
        value: i128,
    ) {
        // The official OApp implementation authenticates the executor and
        // clears through the configured endpoint.  This strict variant keeps
        // those guarantees while rejecting value before any transfer can occur.
        executor.require_auth();
        assert_with_error!(env, value == 0, ReceiptError::NonZeroInboundValue);
        assert_with_error!(
            env,
            get_peer_or_panic::<Self>(env, origin.src_eid) == origin.sender,
            oapp::OAppError::OnlyPeer
        );
        let prior_nonce: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::InboundNonce(origin.src_eid, origin.sender.clone()))
            .unwrap_or(0);
        assert_with_error!(env, origin.nonce == prior_nonce + 1, ReceiptError::InvalidInboundNonce);
        endpoint_client::<Self>(env).clear(
            &env.current_contract_address(),
            origin,
            &env.current_contract_address(),
            guid,
            message,
        );
        Self::__lz_receive(env, origin, guid, message, extra_data, executor, value);
    }

    fn next_nonce(env: &Env, src_eid: u32, sender: &BytesN<32>) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::InboundNonce(src_eid, sender.clone()))
            .unwrap_or(0)
            + 1
    }
}

#[contractimpl]
impl ReceiptAdapter {
    /// Initializes only OApp ownership and endpoint configuration.
    pub fn __constructor(env: &Env, owner: &Address, endpoint: &Address) {
        oapp::oapp_core::init_ownable_oapp::<Self>(env, owner, endpoint, owner);
    }

    pub fn encode_receipt(
        env: &Env,
        campaign_id: u64,
        operation_version: u32,
        status: u32,
        state_hash: &BytesN<32>,
    ) -> Bytes {
        encode(
            env,
            &FinalStateReceipt { campaign_id, operation_version, status, state_hash: state_hash.clone() },
        )
    }

    pub fn decode_receipt(env: &Env, message: &Bytes) -> FinalStateReceipt {
        decode(env, message)
    }

    pub fn quote_final_state(
        env: &Env,
        dst_eid: u32,
        campaign_id: u64,
        operation_version: u32,
        status: u32,
        state_hash: &BytesN<32>,
        options: &Bytes,
        pay_in_zro: bool,
    ) -> MessagingFee {
        let message = encode(
            env,
            &FinalStateReceipt { campaign_id, operation_version, status, state_hash: state_hash.clone() },
        );
        let options = Self::combine_options(env, dst_eid, MSG_TYPE_FINAL_STATE, options);
        Self::__quote(env, dst_eid, &message, &options, pay_in_zro)
    }

    /// Publishes a final-state receipt.  The owner is the only authority
    /// allowed to attest native Stellar finality; this does not touch escrow.
    pub fn send_final_state(
        env: &Env,
        owner: &Address,
        dst_eid: u32,
        campaign_id: u64,
        operation_version: u32,
        status: u32,
        state_hash: &BytesN<32>,
        options: &Bytes,
        fee: &MessagingFee,
        refund_address: &Address,
    ) -> MessagingReceipt {
        enforce_owner_auth::<Self>(env);
        assert_with_error!(env, *owner == Self::owner(env).unwrap(), ReceiptError::OnlyOwner);
        let receipt =
            FinalStateReceipt { campaign_id, operation_version, status, state_hash: state_hash.clone() };
        let message = encode(env, &receipt);
        assert_with_error!(env, !env.storage().persistent().has(&published_key(&receipt)), ReceiptError::DuplicateCampaignOperation);
        let options = Self::combine_options(env, dst_eid, MSG_TYPE_FINAL_STATE, options);
        let sent = Self::__lz_send(
            env,
            dst_eid,
            &message,
            &options,
            &FeePayer::Verified(owner.clone()),
            fee,
            refund_address,
        );
        env.storage().persistent().set(&published_key(&receipt), &true);
        FinalStatePublished {
            campaign_id,
            operation_version,
            status,
            guid: sent.guid.clone(),
            nonce: sent.nonce,
        }
        .publish(env);
        sent
    }

    pub fn was_published(env: &Env, campaign_id: u64, operation_version: u32) -> bool {
        env.storage().persistent().has(&DataKey::Published(campaign_id, operation_version))
    }

    pub fn was_received(env: &Env, campaign_id: u64, operation_version: u32) -> bool {
        env.storage().persistent().has(&DataKey::Received(campaign_id, operation_version))
    }
}

#[cfg(test)]
mod tests;