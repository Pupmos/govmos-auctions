use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint64;
use cw721::Cw721ReceiveMsg;

use crate::auction::{Auction, Bid, DynamicBiddingExtension};

#[cw_serde]
pub struct FeeUnverified {
    pub bps: Uint64,
    pub address: String,
    pub label: String,
}

#[cw_serde]
pub struct NftUnverified {
    pub token_id: String,
    pub owner: String,
    pub contract_addr: String,
}

/// All times are in seconds since epoch

#[cw_serde]
pub struct InstantiateMsg {
    pub nft: NftUnverified,
    pub start_time: Uint64,
    pub end_time: Uint64,
    pub reserve_price: Uint64,
    pub dynamic_bidding_extension: DynamicBiddingExtension,
    pub denom: String,
    pub fees: Vec<FeeUnverified>,
    pub payout_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    PlaceBid {},
    EndAuction {},
    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current state of the auction
    #[returns(AuctionInfoResponse)]
    AuctionInfo {},
    /// Returns the bid history of the auction
    #[returns(BidHistoryResponse)]
    BidHistory {
        /// The number of bids to return
        limit: Option<u32>,
        /// The offset to start from
        offset: Option<u32>,
    },
}

#[cw_serde]
pub struct AuctionInfoResponse {
    pub auction_info: Auction,
}

#[cw_serde]
pub struct BidHistoryResponse {
    pub bid_history: Vec<Bid>,
}
