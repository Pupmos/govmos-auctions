use cosmwasm_std::StdError;
use cw_utils::PaymentError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("Custom error: {0}")]
    CustomError(String),

    // has not started yet
    #[error("Auction has not started yet")]
    AuctionHasNotStartedYet {},

    // no funds
    #[error("No funds")]
    NoFunds {},

    #[error("Zero tokens")]
    ZeroTokens {},

    // wrong asset
    #[error("Wrong asset")]
    WrongAsset {},

    // PaymentError
    #[error("Payment error")]
    PaymentError(#[from] PaymentError),

    #[error("Cannot set both seller address and nft")]
    CannotSetBothSellerAddressAndNft {},

    #[error("Must set either seller address or nft")]
    MustSetEitherSellerAddressOrNft {},

    // "Epoch duration cannot be zero"
    #[error("Epoch duration cannot be zero")]
    EpochDurationCannotBeZero {},

    #[error("Auction already scheduled")]
    AuctionAlreadyScheduled {},

    #[error("Auction start time in the past")]
    AuctionStartTimeInThePast {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
