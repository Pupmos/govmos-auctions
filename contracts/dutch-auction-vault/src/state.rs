use crate::{
    fungible::{CheckedFungibleAsset},
    market::Market,
    ContractError,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_storage_plus::Item;

#[cw_serde]
pub struct CheckedNft {
    pub contract_addr: Addr,
    pub token_id: String,
}

#[cw_serde]
pub struct MarketConfig {
    pub seller_address: Option<Addr>,
    pub seller_nft: Option<CheckedNft>,
    pub spend_asset: CheckedFungibleAsset,
    pub unit_asset: CheckedFungibleAsset,
    pub start_price: Decimal,
    pub min_price: Decimal,
    pub target_duration: Uint128,
}

// validate impl ensuring either seller address or nft is set but not both
impl MarketConfig {
    pub fn validate(&self) -> Result<(), ContractError> {
        match (self.seller_address.is_some(), self.seller_nft.is_some()) {
            (true, true) => Err(ContractError::CannotSetBothSellerAddressAndNft {}),
            (false, false) => Err(ContractError::MustSetEitherSellerAddressOrNft {}),
            _ => Ok(()),
        }
    }
}

pub const MARKET_STATE: Item<Market> = Item::new("market");
pub const MARKET_CONFIG: Item<MarketConfig> = Item::new("market_config");
pub const UNIX_START: Item<Uint64> = Item::new("unix_start");
pub const TOTAL_WITHDRAWN: Item<Uint128> = Item::new("total_withdrawn");

#[cfg(test)]
mod tests {
    use super::*;

    fn create_market_config(
        seller_address: Option<Addr>,
        seller_nft: Option<CheckedNft>,
        spend_asset: CheckedFungibleAsset,
        unit_asset: CheckedFungibleAsset,
        start_price: Decimal,
        min_price: Decimal,
        target_duration: Uint128,
    ) -> MarketConfig {
        MarketConfig {
            seller_address,
            seller_nft,
            spend_asset,
            unit_asset,
            start_price,
            min_price,
            target_duration,
        }
    }

    #[test]
    fn test_market_config_validation() {
        let spend_asset = CheckedFungibleAsset::Native {
            denom: "spend_asset".to_string(),
        };

        let unit_asset = CheckedFungibleAsset::Native {
            denom: "unit_asset".to_string(),
        };

        let seller_address = Addr::unchecked("seller_address");
        let seller_nft = CheckedNft {
            contract_addr: Addr::unchecked("nft_contract"),
            token_id: "token_id".to_string(),
        };

        let config1 = create_market_config(
            Some(seller_address.clone()),
            None,
            spend_asset.clone(),
            unit_asset.clone(),
            Decimal::one(),
            Decimal::one(),
            Uint128::from(1u64),
        );
        assert_eq!(config1.validate(), Ok(()));

        let config2 = create_market_config(
            None,
            Some(seller_nft.clone()),
            spend_asset.clone(),
            unit_asset.clone(),
            Decimal::one(),
            Decimal::one(),
            Uint128::from(1u64),
        );
        assert_eq!(config2.validate(), Ok(()));

        let config3 = create_market_config(
            Some(seller_address),
            Some(seller_nft),
            spend_asset.clone(),
            unit_asset.clone(),
            Decimal::one(),
            Decimal::one(),
            Uint128::from(1u64),
        );
        assert_eq!(
            config3.validate(),
            Err(ContractError::CannotSetBothSellerAddressAndNft {})
        );

        let config4 = create_market_config(
            None,
            None,
            spend_asset,
            unit_asset,
            Decimal::one(),
            Decimal::one(),
            Uint128::from(1u64),
        );
        assert_eq!(
            config4.validate(),
            Err(ContractError::MustSetEitherSellerAddressOrNft {})
        );
    }
}
