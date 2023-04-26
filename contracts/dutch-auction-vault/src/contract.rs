#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, Uint128, Uint64,
};
use cw2::set_contract_version;
use cw_utils::one_coin;

use crate::error::ContractError;
use crate::fungible::{CheckedFungibleAsset, FungibleAsset};
use crate::market::{Cost, Market};
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MarketStateResponse, MigrateMsg, QueryMsg, ReceiveMsg,
    ReceiveNftMsg,
};
use crate::state::{
    CheckedNft, MarketConfig, MARKET_CONFIG, MARKET_STATE, TOTAL_WITHDRAWN, UNIX_START,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:dutch-auction-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let market_config = MarketConfig {
        seller_address: msg
            .seller_address
            .map(|addr| deps.api.addr_validate(&addr))
            .transpose()?,
        seller_nft: msg
            .seller_nft
            .map(|unchecked_nft| -> StdResult<CheckedNft> {
                let contract_addr = deps.api.addr_validate(&unchecked_nft.contract_addr)?;
                Ok(CheckedNft {
                    contract_addr,
                    token_id: unchecked_nft.token_id,
                })
            })
            .transpose()?,
        spend_asset: msg.spend_asset.into_checked(&deps.as_ref())?,
        unit_asset: msg.unit_asset.into_checked(&deps.as_ref())?,
        start_price: msg.start_price,
        min_price: msg.min_price,
        target_duration: msg.target_duration,
    };

    market_config.validate()?;

    MARKET_CONFIG.save(deps.storage, &market_config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handling contract migration
/// To make a contract migratable, you need
/// - this entry_point implemented
/// - only contract admin can migrate, so admin has to be set at contract initiation time
/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender,
            amount,
            msg: recv_action,
        }) => {
            let sender = deps.api.addr_validate(&sender)?;
            let received_amount = amount;
            let received_asset = CheckedFungibleAsset::Cw20 {
                contract_addr: info.sender,
            };
            handle_recv_action(
                deps,
                env,
                sender,
                received_asset,
                received_amount,
                from_binary(&recv_action)?,
            )
        }
        ExecuteMsg::Fund(recv_action) => {
            let received_coin = one_coin(&info)?;
            let received_asset = FungibleAsset::Native {
                denom: received_coin.denom,
            }
            .into_checked(&deps.as_ref())?;
            let received_amount = received_coin.amount;
            handle_recv_action(
                deps,
                env,
                info.sender,
                received_asset,
                received_amount,
                recv_action,
            )
            // Add other ExecuteMsg variants as needed
        }
        ExecuteMsg::ReceiveNft(cw721::Cw721ReceiveMsg {
            sender,
            token_id,
            msg,
        }) => {
            let market_config = MARKET_CONFIG.load(deps.storage)?;
            if let Some(seller_nft) = market_config.seller_nft {
                if info.sender != seller_nft.contract_addr {
                    return Err(ContractError::Unauthorized {});
                }
                if token_id != seller_nft.token_id {
                    return Err(ContractError::Unauthorized {});
                }
            } else {
                return Err(ContractError::Unauthorized {});
            }
            match from_binary(&msg)? {
                ReceiveNftMsg::ScheduleAuction { start_time_unix } => {
                    let sender = deps.api.addr_validate(&sender)?;
                    let mut market_config = MARKET_CONFIG.load(deps.storage)?;
                    market_config.seller_address = Some(sender.clone());
                    MARKET_CONFIG.save(deps.storage, &market_config)?;
                    handle_schedule_auction(deps, env, sender, start_time_unix)
                }
            }
        }
        ExecuteMsg::ScheduleAuction { start_time_unix } => {
            handle_schedule_auction(deps, env, info.sender, start_time_unix)
        }
        ExecuteMsg::WithdrawEarnings {} => withdraw_earnings(deps, env, info),
    }
}

pub fn handle_schedule_auction(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    unix_start_draft: Uint64,
) -> Result<Response, ContractError> {
    let market_config = MARKET_CONFIG.load(deps.storage)?;

    if let Some(seller_addr) = market_config.seller_address {
        if seller_addr != sender {
            return Err(ContractError::Unauthorized {});
        }
    } else {
        return Err(ContractError::Unauthorized {});
    }
    let total_units = market_config
        .unit_asset
        .query_balance(&deps.querier, &env.contract.address)?;

    let market = Market::new(
        market_config.start_price,
        market_config.min_price,
        Uint128::zero(),
        market_config.target_duration,
        Uint128::zero(),
        total_units,
    );

    MARKET_STATE.save(deps.storage, &market)?;

    let unix_start = UNIX_START.may_load(deps.storage)?;

    if unix_start.is_some() {
        return Err(ContractError::AuctionAlreadyScheduled {});
    }
    if unix_start_draft.u64() < env.block.time.seconds() {
        return Err(ContractError::AuctionStartTimeInThePast {});
    }

    UNIX_START.save(deps.storage, &unix_start_draft)?;

    Ok(Response::new()
        .add_attribute("method", "schedule_auction")
        .add_attribute("start_time_unix", unix_start_draft))
}

pub fn handle_recv_action(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    received_asset: CheckedFungibleAsset,
    received_amount: Uint128,
    recv_action: ReceiveMsg,
) -> Result<Response, ContractError> {
    match recv_action {
        ReceiveMsg::BuyUnits { units } => {
            buy_units(deps, env, sender, received_asset, received_amount, units)
        }
        ReceiveMsg::ProvideUnits {} => {
            provide_units(deps, env, sender, received_asset, received_amount)
        }
    }
}

pub fn provide_units(
    deps: DepsMut,
    _env: Env,
    sender: Addr,
    received_asset: CheckedFungibleAsset,
    received_amount: Uint128,
) -> Result<Response, ContractError> {
    // just assert that checked asset is the correct one
    let market_config = MARKET_CONFIG.load(deps.storage)?;
    if market_config.unit_asset != received_asset {
        return Err(ContractError::WrongAsset {});
    }
    Ok(Response::new()
        .add_attribute("method", "provide_units")
        .add_attribute("provider", sender)
        .add_attribute("units", received_amount.to_string()))
}

fn buy_units(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    received_asset: CheckedFungibleAsset,
    received_amount: Uint128,
    units: Uint128,
) -> Result<Response, ContractError> {
    let market_config = MARKET_CONFIG.load(deps.storage)?;
    if market_config.spend_asset != received_asset {
        return Err(ContractError::WrongAsset {});
    }
    let unix_start = UNIX_START.load(deps.storage)?;
    let time_elapsed: Uint128 = Uint64::from(env.block.time.seconds())
        .checked_sub(unix_start)
        .map_err(|_e| ContractError::AuctionHasNotStartedYet {})?
        .into();

    let mut market_state = MARKET_STATE.load(deps.storage)?;
    let Cost(cost) = market_state.buy_units(units, time_elapsed)?;
    MARKET_STATE.save(deps.storage, &market_state)?;

    let remainder = received_amount
        .checked_sub(cost)
        .map_err(|_e| ContractError::InsufficientFunds {})?;

    let change_msgs = if remainder.is_zero() {
        vec![]
    } else {
        vec![received_asset.into_send_message(remainder, &sender)?]
    };

    let payout_msg = market_config.unit_asset.into_send_message(units, &sender)?;

    Ok(Response::new()
        .add_message(payout_msg)
        .add_messages(change_msgs)
        .add_attribute("method", "buy_units")
        .add_attribute("buyer", sender)
        .add_attribute("units", units.to_string())
        .add_attribute("cost", cost.to_string()))
}

fn withdraw_earnings(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let market_config = MARKET_CONFIG.load(deps.storage)?;
    if market_config.seller_address != Some(info.sender.clone()) {
        return Err(ContractError::Unauthorized {});
    }

    let market_state = MARKET_STATE.load(deps.storage)?;
    let total_withdrawn = TOTAL_WITHDRAWN
        .may_load(deps.storage)?
        .unwrap_or(Uint128::zero());

    let to_withdraw = market_state
        .total_spent
        .checked_sub(total_withdrawn)
        .map_err(|_e| ContractError::InsufficientFunds {})?;

    TOTAL_WITHDRAWN.save(deps.storage, &(total_withdrawn + to_withdraw))?;

    let payout_msg = market_config
        .spend_asset
        .into_send_message(to_withdraw, &info.sender)?;

    Ok(Response::new()
        .add_message(payout_msg)
        .add_attribute("method", "withdraw_earnings")
        .add_attribute("seller", info.sender)
        .add_attribute("payout", to_withdraw.to_string()))
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMarketState {} => to_binary(&query_market_state(deps)?), // Add other QueryMsg variants as needed
    }
}

fn query_market_state(deps: Deps) -> Result<MarketStateResponse, StdError> {
    let market = MARKET_STATE.may_load(deps.storage)?;
    let unix_start_time = UNIX_START.may_load(deps.storage)?;
    let config = MARKET_CONFIG.load(deps.storage)?;

    Ok(MarketStateResponse {
        market,
        unix_start_time,
        config,
    })
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages

    Ok(Response::new())
}
