use cosmwasm_std::{Decimal, Uint128};
use std::ops::{Mul, Sub};

use cosmwasm_schema::cw_serde;

use crate::error::ContractError;

#[cw_serde]
/// Auction Market State
pub struct Market {
    pub start_price: Decimal,
    pub min_price: Decimal,
    pub units_sold: Uint128,
    pub target_duration: Uint128,
    pub total_spent: Uint128,
    pub total_units: Uint128,
}

pub struct Cost(pub Uint128);

impl Market {
    pub fn new(
        start_price: Decimal,
        min_price: Decimal,
        units_sold: Uint128,
        target_duration: Uint128,
        total_spent: Uint128,
        total_units: Uint128,
    ) -> Self {
        Market {
            start_price,
            min_price,
            units_sold,
            total_spent,
            target_duration,
            total_units,
        }
    }

    pub fn calculate_price(
        &self,
        units: Uint128,
        time_elapsed: Uint128,
    ) -> Result<Decimal, ContractError> {
        let expected_units_sold = (time_elapsed * self.total_units) / self.target_duration;
        let ratio = if expected_units_sold > Uint128::zero() {
            let next_units_sold = self.units_sold + units;
            let mkt_vol_err = Decimal::from_ratio(next_units_sold, expected_units_sold);

            mkt_vol_err.pow(2)
        } else {
            Decimal::one()
        };

        let price = self.start_price.mul(ratio);
        Ok(price.max(self.min_price))
    }

    pub fn calculate_received(
        &self,
        amount: Uint128,
        time_elapsed: Uint128,
    ) -> Result<Uint128, ContractError> {
        let available_units = self.total_units - self.units_sold;
        let mut last_units_below = Uint128::zero();
        let mut last_units_above = available_units;

        loop {
            let range = last_units_above.sub(last_units_below);
            if range <= Uint128::one() {
                break Ok(last_units_below);
            }

            let guess = (last_units_above + last_units_below) / Uint128::new(2);

            let price = self.calculate_price(guess, time_elapsed)?;
            let units_dec = Decimal::from_ratio(guess, Uint128::new(1));
            let amount_for_units = price.mul(units_dec);

            let amount_for_units = amount_for_units.to_uint_ceil();
            match amount_for_units.cmp(&amount) {
                // amount_for_units < amount
                std::cmp::Ordering::Less => last_units_below = guess,
                // amount_for_units > amount
                std::cmp::Ordering::Greater => last_units_above = guess,
                // amount_for_units == amount
                std::cmp::Ordering::Equal => return Ok(guess),
            }
        }
    }

    pub fn calculate_cost(
        &self,
        units: Uint128,
        time_elapsed: Uint128,
    ) -> Result<Cost, ContractError> {
        let price = self.calculate_price(units, time_elapsed)?;
        let cost = price
            .mul(Decimal::from_ratio(units, Uint128::one()))
            .to_uint_ceil();
        Ok(Cost(cost))
    }

    pub fn buy_units(
        &mut self,
        units: Uint128,
        time_elapsed: Uint128,
    ) -> Result<Cost, ContractError> {
        let Cost(cost) = self.calculate_cost(units, time_elapsed)?;
        self.total_spent += cost;
        self.units_sold += units;
        Ok(Cost(cost))
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Add, str::FromStr};

    use super::*;
    use cosmwasm_std::Decimal;

    // Helper function to create a Market instance for testing
    fn create_test_market() -> Market {
        Market::new(
            Decimal::from_str("1.0").unwrap(),
            Decimal::from_str("0.1").unwrap(),
            Uint128::zero(),
            Uint128::from(86400u128),
            Uint128::from(0u128),
            Uint128::from(1000u128),
        )
    }

    #[test]
    fn test_new_market() {
        let market = create_test_market();

        assert_eq!(market.start_price, Decimal::from_str("1.0").unwrap());
        assert_eq!(market.min_price, Decimal::from_str("0.1").unwrap());
        assert_eq!(market.units_sold, Uint128::from(0u128));
        assert_eq!(market.target_duration, Uint128::from(86400u128));
        assert_eq!(market.total_spent, Uint128::from(0u128));
    }

    #[test]
    fn test_calculate_price() {
        let mut market = create_test_market();

        // Check price calculation when no units are sold
        let price = market
            .calculate_price(Uint128::new(100), Uint128::zero())
            .unwrap();
        assert_eq!(price, Decimal::from_str("1.0").unwrap());

        // Buy some units and check the price calculation
        market
            .buy_units(Uint128::from(100u128), Uint128::zero())
            .unwrap();
        let price = market
            .calculate_price(Uint128::new(100), Uint128::zero())
            .unwrap();
        assert_eq!(price, Decimal::from_str("1.0").unwrap());
    }

    #[test]
    fn test_buy_units() {
        let mut market = create_test_market();

        // Buy some units
        market
            .buy_units(Uint128::from(100u128), Uint128::zero())
            .unwrap();

        assert_eq!(market.total_spent, Uint128::from(100u128));
        assert_eq!(market.units_sold, Uint128::from(100u128));
    }

    #[test]
    fn test_market_new() {
        let market = Market::new(
            Decimal::from_str("1.0").unwrap(),
            Decimal::from_str("0.5").unwrap(),
            Uint128::zero(),
            Uint128::from(3600u128),
            Uint128::from(0u128),
            Uint128::from(1000u128),
        );

        assert_eq!(market.start_price, Decimal::from_str("1.0").unwrap());
        assert_eq!(market.min_price, Decimal::from_str("0.5").unwrap());
        assert_eq!(market.units_sold, Uint128::from(0u128));
        assert_eq!(market.target_duration, Uint128::from(3600u128));
        assert_eq!(market.total_spent, Uint128::from(0u128));
    }

    #[test]
    fn test_market_calculate_price() {
        let market = Market::new(
            Decimal::from_str("1.0").unwrap(),
            Decimal::from_str("0.5").unwrap(),
            Uint128::from(0u128),
            Uint128::from(3600u128),
            Uint128::from(0u128),
            Uint128::from(1000u128),
        );

        let units = Uint128::new(1000);
        let price = market.calculate_price(units, Uint128::zero()).unwrap();
        assert_eq!(price, Decimal::from_str("1.0").unwrap());
    }

    #[test]
    fn test_market_buy_units() {
        let mut market = Market::new(
            Decimal::from_str("0.8").unwrap(),
            Decimal::from_str("0.5").unwrap(),
            Uint128::from(0u128),
            Uint128::from(3600u128),
            Uint128::from(0u128),
            Uint128::from(1000u128),
        );

        let amount = Uint128::from(1000u128);
        market.buy_units(amount, Uint128::zero()).unwrap();

        assert_eq!(market.total_spent, Uint128::from(800u128));
        assert_eq!(market.units_sold, Uint128::from(1000u128));
    }

    #[test]
    fn test_market_simulation() {
        // let mut market = Market::new(
        //     Decimal::from_str("1.0").unwrap(),
        //     Decimal::from_str("0.5").unwrap(),
        //     DecimalPlaces::new(18, 9),
        //     0,
        //     1,
        //     1000,
        //     0,
        //     1000,
        // );

        // with Uint128::from
        let mut market = Market::new(
            Decimal::from_str("1.0").unwrap(),
            Decimal::from_str("0.5").unwrap(),
            Uint128::from(0u128),
            Uint128::from(1000u128),
            Uint128::from(0u128),
            Uint128::from(1000u128),
        );

        let mut time_elapsed = Uint128::from(1u128);

        let purchase_amounts: Vec<u128> = vec![100, 200, 100, 50, 100, 200];
        // calculated each by hand
        let expected_prices: Vec<Decimal> = vec![
            Decimal::from_str("10000").unwrap(), // (100 / ((1000 * 1.0) / 1000)) ^ 2
            Decimal::from_str("22500").unwrap(), // (300 / ((1000 * 2.0) / 1000)) ^ 2
            Decimal::from_str("17777.777777777777777688").unwrap(), // (400 / ((1000 * 3.0) / 1000)) ^ 2
            Decimal::from_str("12656.25").unwrap(), // (450 / ((1000 * 4.0) / 1000)) ^ 2
            Decimal::from_str("12100").unwrap(),    // (550 / ((1000 * 5.0) / 1000)) ^ 2
            Decimal::from_str("15625").unwrap(),    // (750 / ((1000 * 6.0) / 1000)) ^ 2
        ];

        // price = (next_units / ((1000 * 1.0) / 1000)) ^ 2
        //

        for (i, units) in purchase_amounts.iter().enumerate() {
            let price = market
                .calculate_price(Uint128::new(*units), time_elapsed)
                .unwrap();
            let received = market
                .calculate_received(
                    price
                        .mul(Decimal::from_ratio(Uint128::from(*units), Uint128::one()))
                        .to_uint_ceil(),
                    time_elapsed,
                )
                .unwrap();
            assert_eq!(received, Uint128::new(*units));
            assert_eq!(price.to_string(), expected_prices[i].to_string());
            market
                .buy_units(Uint128::from(*units), time_elapsed)
                .unwrap();
            time_elapsed = time_elapsed.add(Uint128::from(1u128));
        }

        let expected_total_spent =
            purchase_amounts
                .iter()
                .zip(expected_prices)
                .fold(0u128, |acc, (amount, price)| {
                    acc + (price * Decimal::from_str(&amount.to_string()).unwrap())
                        .to_uint_ceil()
                        .u128()
                });
        assert_eq!(market.units_sold, Uint128::from(750u128));
        assert_eq!(market.total_spent, Uint128::from(expected_total_spent));
        assert_eq!(time_elapsed, Uint128::from(7u128));
        assert_eq!(
            // (751 / ((1000 * 7.0) / 1000)) ^ 2
            market
                .calculate_price(Uint128::one(), time_elapsed)
                .unwrap(),
            Decimal::from_str("11510.224489795918367285").unwrap()
        );
    }
}
