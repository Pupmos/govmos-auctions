use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128, Uint64};

use crate::{fungible::FungibleAsset, market::Market, state::MarketConfig};

#[cw_serde]
pub struct UncheckedNft {
    pub contract_addr: String,
    pub token_id: String,
}

#[cw_serde]
pub struct ValsetUpdate {
    pub nonce: Uint128,
    pub power: Uint128,
    #[serde(flatten)]
    pub valset: HashMap<String, Uint128>,
}

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    /// how many spend_asset tokens are required to buy one unit_asset token
    pub start_price: Decimal,
    /// the minimum price for which the auction will sell
    pub min_price: Decimal,
    /// the duration of the auction in seconds
    pub target_duration: Uint128,
    /// the asset to be used for bids
    pub spend_asset: FungibleAsset,
    /// the asset to be bid on and sold
    pub unit_asset: FungibleAsset,
    /// the address of the seller or None if the seller is an NFT owner
    pub seller_address: Option<String>,
    /// the NFT to be sold or None if the seller is an address
    pub seller_nft: Option<UncheckedNft>,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    /// Used to provide cw20 tokens to satisfy a funds promise.
    Receive(cw20::Cw20ReceiveMsg),
    /// Provides native tokens to satisfy a funds promise.
    Fund(ReceiveMsg),
    /// Used to schedule auction when seller is an NFT
    ReceiveNft(cw721::Cw721ReceiveMsg),
    /// Used to schedule auction when seller is an address
    ScheduleAuction { start_time_unix: Uint64 },
    /// Withdraws seller's earnings from the auction
    WithdrawEarnings {},
}

// Receive Action
#[cw_serde]
pub enum ReceiveMsg {
    /// A safe way to provide funds to the contract, while asserting the
    /// asset provided is the unit_asset.
    ProvideUnits {},
    /// Send funds to the contract to buy a requested amount of units.
    /// if more than the requested amount is sent, the remainder is returned.
    BuyUnits { units: Uint128 },
}

#[cw_serde]
pub enum ReceiveNftMsg {
    ScheduleAuction { start_time_unix: Uint64 },
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MarketStateResponse)]
    GetMarketState {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct MarketStateResponse {
    pub market: Option<Market>,
    pub unix_start_time: Option<Uint64>,
    pub config: MarketConfig,
}
