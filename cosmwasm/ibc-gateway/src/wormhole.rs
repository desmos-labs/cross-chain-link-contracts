///! this file represents required wormhole contract methods from
///! https://github.com/wormhole-foundation/wormhole/tree/main/terra/contracts/wormhole

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, StdError, Storage, StdResult};
use cw_storage_plus::Map;
use thiserror::Error;

type HumanAddr = String;

#[cw_serde]
pub enum WormholeExecuteMsg {
    SubmitVAA { vaa: Binary },
    PostMessage { message: Binary, nonce: u32 },
}

#[cw_serde]
pub enum WormholeQueryMsg {
    GuardianSetInfo {},
    VerifyVAA { vaa: Binary, block_time: u64 },
    GetState {},
    QueryAddressHex { address: HumanAddr },
}

// ------------------------------------------------------------------------------------------------------------------

#[cw_serde]
pub struct ParsedVAA {
    pub version: u8,
    pub guardian_set_index: u32,
    pub timestamp: u32,
    pub nonce: u32,
    pub len_signers: u8,

    pub emitter_chain: u16,
    pub emitter_address: Vec<u8>,
    pub sequence: u64,
    pub consistency_level: u8,
    pub payload: Vec<u8>,

    pub hash: Vec<u8>,
}

pub static GUARDIAN_SET_KEY: &[u8] = b"guardian_set";
pub const GUARDIAN_SET : Map<&[u8], bool> = Map::new("guardian_set");

pub fn vaa_archive_add(storage: &mut dyn Storage, hash: &[u8]) -> StdResult<()> {
    GUARDIAN_SET.save(storage, hash, &true)
}


pub fn vaa_archive_check(storage: &dyn Storage, hash: &[u8]) -> bool {
    GUARDIAN_SET.
        load(storage, hash).
        unwrap_or(false)
}

// ------------------------------------------------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum WormholeContractError {
    /// Invalid VAA version
    #[error("InvalidVersion")]
    InvalidVersion,

    /// Guardian set with this index does not exist
    #[error("InvalidGuardianSetIndex")]
    InvalidGuardianSetIndex,

    /// Guardian set expiration date is zero or in the past
    #[error("GuardianSetExpired")]
    GuardianSetExpired,

    /// Not enough signers on the VAA
    #[error("NoQuorum")]
    NoQuorum,

    /// Wrong guardian index order, order must be ascending
    #[error("WrongGuardianIndexOrder")]
    WrongGuardianIndexOrder,

    /// Some problem with signature decoding from bytes
    #[error("CannotDecodeSignature")]
    CannotDecodeSignature,

    /// Some problem with public key recovery from the signature
    #[error("CannotRecoverKey")]
    CannotRecoverKey,

    /// Recovered pubkey from signature does not match guardian address
    #[error("GuardianSignatureError")]
    GuardianSignatureError,

    /// VAA action code not recognized
    #[error("InvalidVAAAction")]
    InvalidVAAAction,

    /// VAA guardian set is not current
    #[error("NotCurrentGuardianSet")]
    NotCurrentGuardianSet,

    /// Only 128-bit amounts are supported
    #[error("AmountTooHigh")]
    AmountTooHigh,

    /// Amount should be higher than zero
    #[error("AmountTooLow")]
    AmountTooLow,

    /// Source and target chain ids must be different
    #[error("SameSourceAndTarget")]
    SameSourceAndTarget,

    /// Target chain id must be the same as the current CHAIN_ID
    #[error("WrongTargetChain")]
    WrongTargetChain,

    /// Wrapped asset init hook sent twice for the same asset id
    #[error("AssetAlreadyRegistered")]
    AssetAlreadyRegistered,

    /// Guardian set must increase in steps of 1
    #[error("GuardianSetIndexIncreaseError")]
    GuardianSetIndexIncreaseError,

    /// VAA was already executed
    #[error("VaaAlreadyExecuted")]
    VaaAlreadyExecuted,

    /// Message sender not permitted to execute this operation
    #[error("PermissionDenied")]
    PermissionDenied,

    /// Could not decode target address from canonical to human-readable form
    #[error("WrongTargetAddressFormat")]
    WrongTargetAddressFormat,

    /// More signatures than active guardians found
    #[error("TooManySignatures")]
    TooManySignatures,

    /// Wrapped asset not found in the registry
    #[error("AssetNotFound")]
    AssetNotFound,

    /// Generic error when there is a problem with VAA structure
    #[error("InvalidVAA")]
    InvalidVAA,

    /// Thrown when fee is enabled for the action, but was not sent with the transaction
    #[error("FeeTooLow")]
    FeeTooLow,

    /// Registering asset outside of the wormhole
    #[error("RegistrationForbidden")]
    RegistrationForbidden,
}

impl WormholeContractError {
    pub fn std(&self) -> StdError {
        StdError::GenericErr {
            msg: format!("{self}"),
        }
    }

    pub fn std_err<T>(&self) -> Result<T, StdError> {
        Err(self.std())
    }
}

