use cosmwasm_std::{
    coins, entry_point, to_binary, wasm_execute, BankMsg, Binary, Deps, DepsMut, Env, Event,
    MessageInfo, Response, StdError, StdResult, Timestamp, Uint128,
};
use cw2::set_contract_version;
use cw721::Cw721ReceiveMsg;

use crate::auction::{Auction, Bid, DynamicBiddingExtension, Fee, Nft};
use crate::error::ContractError;
use crate::msg::{AuctionInfoResponse, BidHistoryResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{AUCTION_INFO, BID_HISTORY};

const CONTRACT_NAME: &str = "crates.io:nft-auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let nft = Nft::new(
        msg.nft.token_id,
        deps.api.addr_validate(&msg.nft.owner)?,
        deps.api.addr_validate(&msg.nft.contract_addr)?,
    );

    let dynamic_bidding_extension = DynamicBiddingExtension {
        enabled: msg.dynamic_bidding_extension.enabled,
        time_extension_secs: msg.dynamic_bidding_extension.time_extension_secs,
        min_bid_increase: msg.dynamic_bidding_extension.min_bid_increase,
    };

    let auction = Auction::new(
        nft,
        Timestamp::from_seconds(msg.start_time.into()),
        Timestamp::from_seconds(msg.end_time.into()),
        Uint128::from(msg.reserve_price),
        dynamic_bidding_extension,
        msg.denom,
        msg.fees
            .into_iter()
            .map(|fee| {
                Ok(Fee::new(
                    fee.bps,
                    deps.api.addr_validate(&fee.address)?,
                    fee.label,
                ))
            })
            .collect::<Result<Vec<Fee>, StdError>>()?,
        deps.api.addr_validate(&msg.payout_address)?,
    );

    auction.validate()?;

    AUCTION_INFO.save(deps.storage, &auction)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PlaceBid {} => place_bid(deps, env, info),
        ExecuteMsg::EndAuction {} => end_auction(deps, env),
        ExecuteMsg::ReceiveNft(recv_msg) => receive_nft(deps, env, info, recv_msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, StdError> {
    match msg {
        QueryMsg::AuctionInfo {} => query_auction_info(deps),
        QueryMsg::BidHistory { limit, offset } => query_bid_history(deps, limit, offset),
    }
}

fn receive_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut auction: Auction = AUCTION_INFO.load(deps.storage)?;

    let sender = deps.api.addr_validate(&msg.sender)?;
    auction.receive_nft(msg.token_id, sender, info.sender)?;

    AUCTION_INFO.save(deps.storage, &auction)?;

    Ok(Response::default())
}

fn place_bid(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut auction: Auction = AUCTION_INFO.load(deps.storage)?;
    let previous_bid = auction.current_bid.clone();
    let amount = cw_utils::must_pay(&info, &auction.denom)?;

    auction.place_bid(info.sender.clone(), amount, env.block.time)?;
    AUCTION_INFO.save(deps.storage, &auction)?;

    let bid = Bid {
        bidder: info.sender.clone(),
        amount,
        timestamp: env.block.time,
    };

    BID_HISTORY.push_front(deps.storage, &bid)?;

    // bank send previous bid back to previous bidder
    let mut bank_messages: Vec<BankMsg> = vec![];
    if let Some(previous_bid) = previous_bid {
        bank_messages.push(BankMsg::Send {
            to_address: previous_bid.bidder.to_string(),
            amount: coins(previous_bid.amount.u128(), &auction.denom),
        });
    }

    Ok(Response::default()
        .add_messages(bank_messages)
        .add_attribute("action", "place_bid")
        .add_attribute("bidder", info.sender.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn end_auction(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut auction: Auction = AUCTION_INFO.load(deps.storage)?;
    auction.end_auction(env.block.time)?;

    AUCTION_INFO.save(deps.storage, &auction)?;

    let Some(winning_bid) = auction.winning_bid.clone() else {
        return Err(ContractError::AuctionNotEnded {});
    };

    let nft_msg = wasm_execute(
        auction.nft.contract_addr,
        &cw721::Cw721ExecuteMsg::TransferNft {
            recipient: winning_bid.bidder.to_string(),
            token_id: auction.nft.token_id.clone(),
        },
        vec![],
    )?;

    // send fees to fee addresses then send whats left to the vault
    let mut remaining_amount = winning_bid.amount;
    let mut bank_msgs = vec![];
    let mut attrs: Vec<(String, String)> = vec![];
    for fee in auction.fees {
        let fee_amount = Uint128::from(fee.bps) * winning_bid.amount / Uint128::from(10_000_u128);
        bank_msgs.push(BankMsg::Send {
            to_address: fee.address.to_string(),
            amount: coins(fee_amount.u128(), auction.denom.clone()),
        });
        remaining_amount -= fee_amount;
        attrs.push((fee.label, fee_amount.to_string()));
    }
    let fee_dist_event = Event::new("fee_distribution")
        .add_attribute("auction_denom", auction.denom.clone())
        .add_attribute("auction_amount", winning_bid.amount.to_string())
        .add_attributes(attrs);
    bank_msgs.push(BankMsg::Send {
        to_address: auction.payout_address.to_string(),
        amount: coins(remaining_amount.u128(), auction.denom.clone()),
    });
    Ok(Response::default()
        .add_message(nft_msg)
        .add_messages(bank_msgs)
        .add_event(fee_dist_event)
        .add_attribute("action", "end_auction"))
}

fn query_auction_info(deps: Deps) -> Result<Binary, StdError> {
    let auction: Auction = AUCTION_INFO.load(deps.storage)?;
    let response = AuctionInfoResponse {
        auction_info: auction,
    };
    to_binary(&response)
}

fn query_bid_history(
    deps: Deps,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Binary, StdError> {
    let bids = BID_HISTORY
        .iter(deps.storage)?
        .skip(offset.unwrap_or(0) as usize)
        .take(limit.unwrap_or(30) as usize)
        .collect::<StdResult<Vec<Bid>>>()?;

    let response = BidHistoryResponse { bid_history: bids };
    to_binary(&response)
}
