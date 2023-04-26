
use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Payment Error")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    // FeeBpsTooHigh
    #[error("Fee basis points are too high")]
    InvalidFeeBps { total_bps: u64 },

    // FeeBpsTooHigh
    #[error("Fee basis points are too high")]
    FeeBpsTooHigh {},

    // FeeLabelTooLong
    #[error("Fee label is too long")]
    FeeLabelTooLong {},

    // FeeLabelTooShort
    #[error("Fee label is too short")]
    FeeLabelTooShort {},

    // FeeLabelInvalidCharacters
    #[error("Fee label contains invalid characters")]
    FeeLabelInvalidCharacters {},

    // FeeLabelNotLowerCase
    #[error("Fee label is not lower case")]
    FeeLabelNotLowerCase {},

    #[error("NFT already received")]
    NftAlreadyReceived {},

    #[error("Received NFT token ID doesn't match the auction token ID")]
    NftTokenIdMismatch {},

    #[error("Received NFT owner doesn't match the auction owner")]
    NftOwnerMismatch {},

    #[error("Received NFT contract address doesn't match the auction contract address")]
    NftContractAddrMismatch {},

    #[error("Auction hasn't started yet")]
    AuctionNotStarted {},

    #[error("Auction hasn't ended yet")]
    AuctionNotEnded {},

    #[error("Auction has ended")]
    AuctionEnded {},

    #[error("Bid amount is below the reserve price")]
    BidBelowReservePrice {},

    #[error("NFT hasn't been received yet")]
    NftNotReceived {},

    #[error("Bid amount must be higher than the current bid")]
    BidAmountTooLow {},

    #[error("Bid amount must be higher than the current bid by the minimum bid increase")]
    BidAmountBelowMinIncrease {},

    #[error("Invalid dynamic bidding extension configuration")]
    InvalidDynamicBiddingExtension {},

    // AuctionEndTimeBeforeStartTime
    #[error("Auction end time is before the start time")]
    AuctionEndTimeBeforeStartTime {},

    // MinBidIncreaseZero
    #[error("Minimum bid increase is zero")]
    MinBidIncreaseZero {},

    // TooManyFees
    #[error("Too many fees")]
    TooManyFees {},

    // TotalFeeBpsTooHigh
    #[error("Total fee basis points are too high")]
    TotalFeeBpsTooHigh {},

    #[error("Invalid timestamp")]
    InvalidTimestamp {},

    #[error("Invalid Uint128")]
    InvalidUint128 {},

    #[error("Invalid Uint64")]
    InvalidUint64 {},
}
