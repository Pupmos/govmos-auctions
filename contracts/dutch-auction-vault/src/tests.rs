use std::ops::Sub;
use std::str::FromStr;

use crate::contract::*;
use crate::fungible::CheckedFungibleAsset;
use crate::fungible::FungibleAsset;
use crate::market::Cost;
use crate::market::Market;
use crate::msg::*;
use crate::state::CheckedNft;
use crate::state::MarketConfig;
use crate::state::MARKET_CONFIG;
use crate::state::MARKET_STATE;
use crate::ContractError;

use cosmwasm_std::to_binary;
use cosmwasm_std::Coin;
use cosmwasm_std::Decimal;

use cosmwasm_std::Uint128;
use cosmwasm_std::Uint64;
use cosmwasm_std::{Addr, Empty};
use cw20::Cw20Coin;

use cw_multi_test::AppResponse;
use cw_multi_test::BankSudo;
use cw_multi_test::Contract;
use cw_multi_test::SudoMsg;
use cw_multi_test::{App, ContractWrapper, Executor};
use rust_decimal::prelude::ToPrimitive;

// courtesy of DAO DAO
pub fn cw721_base_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}

pub fn cw20_base_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

pub fn dutch_auction_vault_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

pub fn instantiate_cw721_base(app: &mut App, sender: &str, minter: &str) -> Addr {
    let cw721_id = app.store_code(cw721_base_contract());

    app.instantiate_contract(
        cw721_id,
        Addr::unchecked(sender),
        &cw721_base::InstantiateMsg {
            name: "good boiz".to_string(),
            symbol: "good boiz".to_string(),
            minter: minter.to_string(),
        },
        &[],
        "cw721_base".to_string(),
        None,
    )
    .unwrap()
}

pub fn instantiate_cw20_base(app: &mut App, sender: &str, minter: &str) -> Addr {
    let cw20_id = app.store_code(cw20_base_contract());

    app.instantiate_contract(
        cw20_id,
        Addr::unchecked(sender),
        &cw20_base::msg::InstantiateMsg {
            name: "good boiz".to_string(),
            symbol: "gud".to_string(),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: sender.to_string(),
                amount: Uint128::new(100),
            }],
            mint: Some(cw20::MinterResponse {
                minter: minter.to_string(),
                cap: None,
            }),
            marketing: None,
        },
        &[],
        "cw20_base".to_string(),
        Some(sender.to_string()),
    )
    .unwrap()
}

pub fn instantiate_dutch_auction_vault(app: &mut App, sender: &str, msg: &InstantiateMsg) -> Addr {
    let dutch_auction_vault_id = app.store_code(dutch_auction_vault_contract());

    app.instantiate_contract(
        dutch_auction_vault_id,
        Addr::unchecked(sender),
        &msg,
        &[],
        "dutch_auction_vault".to_string(),
        None,
    )
    .unwrap()
}

pub fn default_dutch_auction_instantiate_msg(
    spend_fungible: &FungibleAsset,
    nft_addr: &Addr,
    nft_token_id: &str,
    unit_fungible: &FungibleAsset,
) -> InstantiateMsg {
    InstantiateMsg {
        start_price: Decimal::from_str("1.0").unwrap(),
        min_price: Decimal::from_str("0.1").unwrap(),
        target_duration: Uint128::from(100u128),
        spend_asset: spend_fungible.clone(),
        unit_asset: unit_fungible.clone(),
        seller_address: None,
        seller_nft: Some(UncheckedNft {
            contract_addr: nft_addr.to_string(),
            token_id: nft_token_id.to_string(),
        }),
    }
}

pub fn mint_sample_nft(
    app: &mut App,
    minter: &Addr,
    owner: &Addr,
    nft_addr: &Addr,
    token_id: &str,
) -> anyhow::Result<AppResponse> {
    let mint_msg = cw721_base::msg::ExecuteMsg::<Empty, Empty>::Mint {
        token_id: token_id.to_string(),
        owner: owner.to_string(),
        token_uri: None,
        extension: Empty {},
    };

    app.execute_contract(minter.clone(), nft_addr.clone(), &mint_msg, &[])
}

struct TestEnv {
    app: App,
    nft_addr: Addr,
    cw20_a_addr: Addr,
    cw20_b_addr: Addr,
    dutch_auction_vault_addr: Addr,
    owner: Addr,
    seller: Addr,
    buyer: Addr,
    nft_token_id: String,
    denom_spend: String,
    denom_unit: String,
    fungible_spend: FungibleAsset,
    fungible_unit: FungibleAsset,
    instantiate_msg: InstantiateMsg,
}

impl TestEnv {
    fn setup(_has_cw20_units: bool, _has_cw20_spend: bool) -> TestEnv {
        let owner: Addr = Addr::unchecked("owner");
        let seller: Addr = Addr::unchecked("seller");
        let buyer: Addr = Addr::unchecked("buyer");
        let nft_token_id: String = "token_id".to_string();
        let denom_spend: String = "ujuno".to_string();
        let denom_unit: String = "uatom".to_string();
        let mut app = App::default();

        let nft_addr = instantiate_cw721_base(&mut app, owner.as_ref(), owner.as_ref());
        mint_sample_nft(&mut app, &owner, &seller, &nft_addr, &nft_token_id).unwrap();

        println!("nft_addr: {}", nft_addr);

        let cw20_spend_addr = instantiate_cw20_base(&mut app, owner.as_ref(), owner.as_ref());

        let cw20_unit_addr = instantiate_cw20_base(&mut app, owner.as_ref(), owner.as_ref());

        let fungible_spend = if _has_cw20_spend {
            FungibleAsset::Cw20 {
                contract_addr: cw20_spend_addr.to_string(),
            }
        } else {
            FungibleAsset::Native {
                denom: denom_spend.clone(),
            }
        };
        let fungible_unit = if _has_cw20_units {
            FungibleAsset::Cw20 {
                contract_addr: cw20_unit_addr.to_string(),
            }
        } else {
            FungibleAsset::Native {
                denom: denom_unit.clone(),
            }
        };
        println!("spend_cw20_addr: {}", cw20_spend_addr);
        let instantiate_msg = default_dutch_auction_instantiate_msg(
            &fungible_spend,
            &nft_addr,
            &nft_token_id,
            &fungible_unit,
        );

        println!("instantiate_msg: {:?}", instantiate_msg);

        let dutch_auction_vault_addr =
            instantiate_dutch_auction_vault(&mut app, owner.as_ref(), &instantiate_msg);

        println!("dutch_auction_vault_addr: {}", dutch_auction_vault_addr);

        TestEnv {
            app,
            nft_addr,
            dutch_auction_vault_addr,
            owner,
            seller,
            buyer,
            nft_token_id,
            cw20_a_addr: cw20_spend_addr,
            cw20_b_addr: cw20_unit_addr,
            denom_spend,
            denom_unit,
            fungible_spend: fungible_spend.clone(),
            fungible_unit: fungible_unit.clone(),
            instantiate_msg,
        }
    }

    pub fn market_config(&self) -> MarketConfig {
        MARKET_CONFIG
            .query(&self.app.wrap(), self.dutch_auction_vault_addr.clone())
            .unwrap()
    }

    pub fn market_state(&self) -> Market {
        MARKET_STATE
            .query(&self.app.wrap(), self.dutch_auction_vault_addr.clone())
            .unwrap()
    }

    pub fn mint_native(&mut self, recipient: Addr, denom: &str, amount: Uint128) {
        self.app
            .sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: recipient.to_string(),
                amount: vec![Coin {
                    amount,
                    denom: denom.to_string(),
                }],
            }))
            .unwrap();
    }

    pub fn mint(&mut self, token: CheckedFungibleAsset, recipient: Addr, amount: Uint128) {
        match token {
            CheckedFungibleAsset::Native { denom } => self.mint_native(recipient, &denom, amount),
            CheckedFungibleAsset::Cw20 { contract_addr } => {
                self.mint_cw20(contract_addr, recipient, amount)
            }
        }
    }

    pub fn mint_cw20(&mut self, contract_addr: Addr, recipient: Addr, amount: Uint128) {
        let msg = cw20_base::msg::ExecuteMsg::Mint {
            recipient: recipient.to_string(),
            amount,
        };

        self.app
            .execute_contract(self.owner.clone(), contract_addr, &msg, &[])
            .unwrap();
    }
    pub fn mint_cw20_a(&mut self, recipient: Addr, amount: Uint128) {
        let msg = cw20_base::msg::ExecuteMsg::Mint {
            recipient: recipient.to_string(),
            amount,
        };

        self.app
            .execute_contract(self.owner.clone(), self.cw20_a_addr.clone(), &msg, &[])
            .unwrap();
    }

    pub fn mint_cw20_b(&mut self, recipient: Addr, amount: Uint128) {
        let msg = cw20_base::msg::ExecuteMsg::Mint {
            recipient: recipient.to_string(),
            amount,
        };

        self.app
            .execute_contract(self.owner.clone(), self.cw20_b_addr.clone(), &msg, &[])
            .unwrap();
    }

    pub fn schedule_auction(&mut self, start_time_unix: u64) {
        let msg = ExecuteMsg::ScheduleAuction {
            start_time_unix: start_time_unix.into(),
        };

        self.app
            .execute_contract(
                self.nft_addr.clone(),
                self.dutch_auction_vault_addr.clone(),
                &ExecuteMsg::ReceiveNft(cw721::Cw721ReceiveMsg {
                    sender: self.seller.to_string(),
                    token_id: self.nft_token_id.clone(),
                    msg: to_binary(&msg).unwrap(),
                }),
                &[],
            )
            .unwrap();
    }

    pub fn block(&mut self) {
        self.app.update_block(|block| {
            block.height += 1;
            block.time = block.time.plus_seconds(2);
        });
    }
}

#[test]
fn test_instantiate() {
    let TestEnv {
        app,
        nft_addr,
        cw20_a_addr: spend_cw20_addr,
        dutch_auction_vault_addr,
        owner: _,
        seller: _,
        buyer: _,
        nft_token_id,
        denom_spend,
        cw20_b_addr: _,
        denom_unit,
        fungible_spend,
        fungible_unit,
        instantiate_msg,
    } = TestEnv::setup(false, true);

    let resp: MarketStateResponse = app
        .wrap()
        .query_wasm_smart(dutch_auction_vault_addr, &QueryMsg::GetMarketState {})
        .unwrap();

    assert_eq!(
        resp,
        MarketStateResponse {
            market: None,
            unix_start_time: None,
            config: MarketConfig {
                seller_address: instantiate_msg.seller_address.map(Addr::unchecked),
                seller_nft: instantiate_msg.seller_nft.map(|nft| CheckedNft {
                    contract_addr: Addr::unchecked(nft.contract_addr),
                    token_id: nft.token_id
                }),
                spend_asset: crate::fungible::CheckedFungibleAsset::Cw20 {
                    contract_addr: spend_cw20_addr
                },
                unit_asset: crate::fungible::CheckedFungibleAsset::Native { denom: denom_unit },
                start_price: instantiate_msg.start_price,
                min_price: instantiate_msg.min_price,
                target_duration: instantiate_msg.target_duration
            }
        }
    );
}

/// ---------------------------------------------
/// SUCCESS CASES
/// ---------------------------------------------
///
/// These tests are for the happy path of the contract.
/// They simply test that the contract executes as expected with normal inputs.

/// DENOM UNITS + CW20 SPEND
#[test]
fn test_execute_receive_cw20() {
    let mut env = TestEnv::setup(false, true);
    let time = env.app.block_info().time;
    let total_units = Uint128::from(100000u128);

    env.mint(
        env.market_config().unit_asset,
        env.dutch_auction_vault_addr.clone(),
        total_units,
    );

    env.mint(
        env.market_config().spend_asset,
        env.buyer.clone(),
        Uint128::new(20),
    );

    env.schedule_auction(time.seconds() + 1);
    env.block();
    env.block();
    env.block();
    env.block();
    let _seller_initial_balance = env
        .market_config()
        .spend_asset
        .query_balance(&env.app.wrap(), &env.seller)
        .unwrap();
    let buyer_initial_balance: Uint128 = env
        .market_config()
        .spend_asset
        .query_balance(&env.app.wrap(), &env.buyer)
        .unwrap();
    let contract_initial_balance: Uint128 = env
        .market_config()
        .unit_asset
        .query_balance(&env.app.wrap(), &env.dutch_auction_vault_addr)
        .unwrap();

    // buyer sends cw20 tokens to dutch_auction_vault to buy units
    let cw20_send_msg = cw20::Cw20ExecuteMsg::Send {
        amount: Uint128::new(10),
        msg: to_binary(&ReceiveMsg::BuyUnits {
            units: Uint128::new(10),
        })
        .unwrap(),
        contract: env.dutch_auction_vault_addr.to_string(),
    };

    env.app
        .execute_contract(
            env.buyer.clone(),
            env.cw20_a_addr.clone(),
            &cw20_send_msg,
            &[],
        )
        .unwrap();

    let Cost(cost) = env
        .market_state()
        .calculate_cost(
            Uint128::new(10),
            Uint128::new(
                env.app
                    .block_info()
                    .time
                    .seconds()
                    .sub(time.seconds())
                    .to_u128()
                    .unwrap(),
            ),
        )
        .unwrap();

    let buyer_new_balance: Uint128 = env
        .market_config()
        .spend_asset
        .query_balance(&env.app.wrap(), &env.buyer)
        .unwrap();
    assert_eq!(buyer_initial_balance - cost, buyer_new_balance);

    let contract_new_balance: Uint128 = env
        .market_config()
        .unit_asset
        .query_balance(&env.app.wrap(), &env.dutch_auction_vault_addr)
        .unwrap();
    assert_eq!(
        contract_initial_balance - Uint128::new(10),
        contract_new_balance
    );

    env.mint_cw20_b(env.buyer.clone(), Uint128::new(20));
    // buyer sends cw20 tokens to dutch_auction_vault to buy units
    let cw20_send_msg = cw20::Cw20ExecuteMsg::Send {
        amount: Uint128::new(20),
        msg: to_binary(&ReceiveMsg::BuyUnits {
            units: Uint128::new(10),
        })
        .unwrap(),
        contract: env.dutch_auction_vault_addr.to_string(),
    };

    // should fail because buyer is sending wrong asset
    let bad_result = env
        .app
        .execute_contract(
            env.buyer.clone(),
            env.cw20_b_addr.clone(),
            &cw20_send_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert!(matches!(bad_result, ContractError::WrongAsset {}));

    env.mint_cw20_a(env.buyer.clone(), Uint128::new(10000000000));
    // try to buy more units than are available
    let _cw20_send_msg = cw20::Cw20ExecuteMsg::Send {
        amount: Uint128::new(20000000),
        msg: to_binary(&ReceiveMsg::BuyUnits {
            units: total_units + Uint128::new(1),
        })
        .unwrap(),
        contract: env.dutch_auction_vault_addr.to_string(),
    };
}

/// CW20 UNITS + DENOM SPEND
#[test]
fn test_execute_receive_funds() {
    let mut env = TestEnv::setup(true, false);
    let time = env.app.block_info().time;
    let total_units = Uint128::from(100000u128);
    env.mint(
        env.market_config().unit_asset,
        env.dutch_auction_vault_addr.clone(),
        total_units,
    );
    env.mint(
        env.market_config().spend_asset,
        env.buyer.clone(),
        Uint128::new(20),
    );
    env.schedule_auction(time.seconds() + 1);
    env.block();
    env.block();
    env.block();
    env.block();
    let _seller_initial_balance = env
        .market_config()
        .spend_asset
        .query_balance(&env.app.wrap(), &env.seller)
        .unwrap();
    let buyer_initial_balance: Uint128 = env
        .market_config()
        .spend_asset
        .query_balance(&env.app.wrap(), &env.buyer)
        .unwrap();
    let contract_initial_balance: Uint128 = env
        .market_config()
        .unit_asset
        .query_balance(&env.app.wrap(), &env.dutch_auction_vault_addr)
        .unwrap();

    env.app
        .execute_contract(
            env.buyer.clone(),
            env.dutch_auction_vault_addr.clone(),
            &ExecuteMsg::Fund(ReceiveMsg::BuyUnits {
                units: Uint128::new(10),
            }),
            &[Coin {
                denom: env.denom_spend.clone(),
                amount: Uint128::new(10),
            }],
        )
        .unwrap();

    let market_state = env.market_state();
    let Cost(cost) = market_state
        .calculate_cost(
            Uint128::new(10),
            Uint128::new(
                env.app
                    .block_info()
                    .time
                    .seconds()
                    .sub(time.seconds())
                    .to_u128()
                    .unwrap(),
            ),
        )
        .unwrap();

    let buyer_new_balance: Uint128 = env
        .market_config()
        .spend_asset
        .query_balance(&env.app.wrap(), &env.buyer)
        .unwrap();
    assert_eq!(buyer_initial_balance - cost, buyer_new_balance);

    let contract_new_balance: Uint128 = env
        .market_config()
        .unit_asset
        .query_balance(&env.app.wrap(), &env.dutch_auction_vault_addr)
        .unwrap();
    assert_eq!(
        contract_initial_balance - Uint128::new(10),
        contract_new_balance
    );

    env.mint_cw20_b(env.buyer.clone(), Uint128::new(20));
    // buyer sends cw20 tokens to dutch_auction_vault to buy units
    let cw20_send_msg = cw20::Cw20ExecuteMsg::Send {
        amount: Uint128::new(20),
        msg: to_binary(&ReceiveMsg::BuyUnits {
            units: Uint128::new(10),
        })
        .unwrap(),
        contract: env.dutch_auction_vault_addr.to_string(),
    };

    // should fail because buyer is sending wrong asset
    let bad_result = env
        .app
        .execute_contract(
            env.buyer.clone(),
            env.cw20_b_addr.clone(),
            &cw20_send_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert!(matches!(bad_result, ContractError::WrongAsset {}));

    env.mint_cw20_a(env.buyer.clone(), Uint128::new(10000000000));
    // try to buy more units than are available
    let _cw20_send_msg = cw20::Cw20ExecuteMsg::Send {
        amount: Uint128::new(20000000),
        msg: to_binary(&ReceiveMsg::BuyUnits {
            units: total_units + Uint128::new(1),
        })
        .unwrap(),
        contract: env.dutch_auction_vault_addr.to_string(),
    };
}

#[test]
fn test_execute_fund() {
    let mut env = TestEnv::setup(false, true);

    let fund_msg = ExecuteMsg::Fund(ReceiveMsg::ProvideUnits {});
    env.mint_native(
        env.buyer.clone(),
        &env.denom_unit.clone(),
        Uint128::new(100),
    );
    env.app
        .execute_contract(
            env.buyer.clone(),
            env.dutch_auction_vault_addr.clone(),
            &fund_msg,
            &[Coin {
                denom: env.denom_unit.clone(),
                amount: Uint128::new(30),
            }],
        )
        .unwrap();

    let buyer_coin_balance: Uint128 = env
        .market_config()
        .unit_asset
        .query_balance(&env.app.wrap(), &env.buyer)
        .unwrap();
    let contract_coin_balance: Uint128 = env
        .market_config()
        .unit_asset
        .query_balance(&env.app.wrap(), &env.dutch_auction_vault_addr)
        .unwrap();

    assert_eq!(Uint128::new(70), buyer_coin_balance);
    assert_eq!(Uint128::new(30), contract_coin_balance);
}

#[test]
fn test_execute_receive_nft() {
    let mut env = TestEnv::setup(false, true);

    let start_time = env.app.block_info().time.seconds() + 2000u64;

    let cw721_receive_msg = ExecuteMsg::ReceiveNft(cw721::Cw721ReceiveMsg {
        sender: env.seller.to_string(),
        token_id: env.nft_token_id.clone(),
        msg: to_binary(&ReceiveNftMsg::ScheduleAuction {
            start_time_unix: Uint64::new(start_time),
        })
        .unwrap(),
    });

    env.app
        .execute_contract(
            env.nft_addr.clone(),
            env.dutch_auction_vault_addr.clone(),
            &cw721_receive_msg,
            &[],
        )
        .unwrap();

    let market_state: MarketStateResponse = env
        .app
        .wrap()
        .query_wasm_smart(
            env.dutch_auction_vault_addr.clone(),
            &QueryMsg::GetMarketState {},
        )
        .unwrap();

    assert_eq!(
        market_state.unix_start_time.unwrap(),
        Uint64::new(start_time)
    );
}
