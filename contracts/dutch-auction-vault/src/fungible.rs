// CREDIT TO DAO DAO https://github.com/DA0-DA0/dao-contracts/blob/main/contracts/external/cw-token-swap/src/state.rs

use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, QuerierWrapper, StdError, Uint128, WasmMsg,
};

/// Information about the token being used on one side of the escrow.
#[cw_serde]
pub enum FungibleAsset {
    /// A native token.
    Native { denom: String },
    /// A cw20 token.
    Cw20 { contract_addr: String },
}

#[cw_serde]
pub enum CheckedFungibleAsset {
    Native { denom: String },
    Cw20 { contract_addr: Addr },
}

impl CheckedFungibleAsset {
    pub fn into_send_message(
        self,
        amount: Uint128,
        recipient: &Addr,
    ) -> Result<CosmosMsg, StdError> {
        Ok(match self {
            Self::Native { denom } => BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![Coin { denom, amount }],
            }
            .into(),
            Self::Cw20 { contract_addr } => WasmMsg::Execute {
                contract_addr: contract_addr.into_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: recipient.to_string(),
                    amount,
                })?,
                funds: vec![],
            }
            .into(),
        })
    }

    pub fn query_balance(
        &self,
        querier: &QuerierWrapper,
        address: &Addr,
    ) -> Result<Uint128, StdError> {
        Ok(match self {
            Self::Native { denom } => querier.query_balance(address, denom)?.amount,
            Self::Cw20 { contract_addr } => {
                let balance: cw20::BalanceResponse = querier.query_wasm_smart(
                    contract_addr.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: address.to_string(),
                    },
                )?;
                balance.balance
            }
        })
    }
}

impl FungibleAsset {
    pub fn into_checked(self, deps: &Deps) -> Result<CheckedFungibleAsset, ContractError> {
        match self {
            FungibleAsset::Native { denom } => Ok(CheckedFungibleAsset::Native { denom }),
            FungibleAsset::Cw20 { contract_addr } => {
                let contract_addr = deps.api.addr_validate(&contract_addr)?;
                // Make sure we are dealing with a cw20.
                let _: cw20::TokenInfoResponse = deps
                    .querier
                    .query_wasm_smart(contract_addr.clone(), &cw20::Cw20QueryMsg::TokenInfo {})?;
                Ok(CheckedFungibleAsset::Cw20 { contract_addr })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_spend_message_native() {
        let info = CheckedFungibleAsset::Native {
            denom: "uekez".to_string(),
        };
        let message = info
            .into_send_message(Uint128::new(100), &Addr::unchecked("ekez"))
            .unwrap();

        assert_eq!(
            message,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "ekez".to_string(),
                amount: vec![Coin {
                    amount: Uint128::new(100),
                    denom: "uekez".to_string()
                }]
            })
        );
    }

    #[test]
    fn test_into_spend_message_cw20() {
        let info = CheckedFungibleAsset::Cw20 {
            contract_addr: Addr::unchecked("ekez_token"),
        };
        let message = info
            .into_send_message(Uint128::new(100), &Addr::unchecked("ekez"))
            .unwrap();

        assert_eq!(
            message,
            CosmosMsg::Wasm(WasmMsg::Execute {
                funds: vec![],
                contract_addr: "ekez_token".to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: "ekez".to_string(),
                    amount: Uint128::new(100)
                })
                .unwrap()
            })
        );
    }
}
