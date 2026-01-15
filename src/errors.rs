
use scale::{Decode, Encode};
use ink::env::Error as EnvError;

/// Lottery error messages
#[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// Attempt to start the lottery when it is already started
    AlreadyStarted,
    /// Error when starting the lottery beyond the starting block
    InvalidBlock,
    /// Standard error if it could not find the record
    NoRecords,    
    /// Standard error if the account is not was is expected
    BadOrigin,
    /// Total draws exceeded the set maximum draws
    TooManyDraws,
    /// Cannot find the draw number
    DrawNotFound,
    /// The draw is still close
    DrawClosed,
    /// The draw is still open
    DrawOpen,
    /// The draw is still being processed
    DrawProcessing,
    /// The draw is not anymore processing
    DrawNotProcessing,
    /// The bet must equal to the set bet amount
    InvalidBetAmount,
    /// Invalid blocks hierarchy
    InvalidBlocksHierarchy,
    /// The draw is not yet closed
    DrawNotClosed,
}

/// Runtime call execution error
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum RuntimeError {
    /// Failed to dispatch a runtime call.
    CallRuntimeFailed,
}

/// Unified contract error type.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    /// Internal errors.
    Internal(Error),
    /// Runtime call errors.
    Runtime(RuntimeError),
}

// Error conversions for convenience.
impl From<Error> for ContractError {
    fn from(err: Error) -> Self {
        Self::Internal(err)
    }
}

impl From<RuntimeError> for ContractError {
    fn from(err: RuntimeError) -> Self {
        Self::Runtime(err)
    }
}

impl From<EnvError> for RuntimeError {
    fn from(e: EnvError) -> Self {
        use ink::env::ReturnErrorCode;
        match e {
            EnvError::ReturnError(ReturnErrorCode::CallRuntimeFailed) => {
                Self::CallRuntimeFailed
            }
            _ => panic!("Unexpected error from pallet_contracts environment"),
        }
    }
}