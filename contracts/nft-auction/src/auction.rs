use cosmwasm_schema::cw_serde;
// Import necessary crates
use crate::ContractError;
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};

// Fee structure
#[cw_serde]
pub struct Fee {
    pub bps: Uint64,
    pub address: Addr,
    pub label: String,
}

// NFT representation
#[cw_serde]
pub struct Nft {
    pub token_id: String,
    pub owner: Addr,
    pub contract_addr: Addr,
    pub received: bool,
}

// Bid structure
#[cw_serde]
pub struct Bid {
    pub bidder: Addr,
    pub amount: Uint128,
    pub timestamp: Timestamp,
}

// Dynamic bidding extension configuration
#[cw_serde]
pub struct DynamicBiddingExtension {
    pub enabled: bool,
    pub time_extension_secs: Uint64,
    pub min_bid_increase: Uint128,
}

// Auction structure
#[cw_serde]
pub struct Auction {
    pub nft: Nft,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub reserve_price: Uint128,
    pub current_bid: Option<Bid>,
    pub dynamic_bidding_extension: DynamicBiddingExtension,
    pub denom: String,
    pub winning_bid: Option<Bid>,
    pub fees: Vec<Fee>,
    pub payout_address: Addr,
}

impl Fee {
    pub fn new(bps: Uint64, address: Addr, label: String) -> Self {
        Fee {
            bps,
            address,
            label,
        }
    }
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.bps > Uint64::from(10000u64) {
            return Err(ContractError::FeeBpsTooHigh {});
        }
        // fee label should be format of twitter handle
        if self.label.len() > 15 {
            return Err(ContractError::FeeLabelTooLong {});
        }
        if self.label.is_empty() {
            return Err(ContractError::FeeLabelTooShort {});
        }
        // only alphanumeric and underscore allowed
        if !self.label.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ContractError::FeeLabelInvalidCharacters {});
        }
        // only lowercase allowed
        if self.label != self.label.to_lowercase() {
            return Err(ContractError::FeeLabelNotLowerCase {});
        }

        Ok(())
    }
}

impl Nft {
    pub fn new(token_id: String, owner: Addr, contract_addr: Addr) -> Self {
        Nft {
            token_id,
            owner,
            contract_addr,
            received: false,
        }
    }
}

impl Bid {
    pub fn new(bidder: Addr, amount: Uint128, timestamp: Timestamp) -> Self {
        Bid {
            bidder,
            amount,
            timestamp,
        }
    }
}

impl DynamicBiddingExtension {
    pub fn new(enabled: bool, time_extension_secs: Uint64, min_bid_increase: Uint128) -> Self {
        DynamicBiddingExtension {
            enabled,
            time_extension_secs,
            min_bid_increase,
        }
    }
}

impl Auction {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        nft: Nft,
        start_time: Timestamp,
        end_time: Timestamp,
        reserve_price: Uint128,
        dynamic_bidding_extension: DynamicBiddingExtension,
        denom: String,
        fees: Vec<Fee>,
        payout_address: Addr,
    ) -> Self {
        Auction {
            nft,
            start_time,
            end_time,
            reserve_price,
            current_bid: None,
            dynamic_bidding_extension,
            denom,
            winning_bid: None,
            fees,
            payout_address,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.start_time >= self.end_time {
            return Err(ContractError::AuctionEndTimeBeforeStartTime {});
        }

        if self.dynamic_bidding_extension.min_bid_increase.is_zero() {
            return Err(ContractError::MinBidIncreaseZero {});
        }

        if self.fees.len() > 10 {
            return Err(ContractError::TooManyFees {});
        }

        let mut total_fee_bps = Uint64::zero();
        for fee in &self.fees {
            total_fee_bps += fee.bps;
            fee.validate()?;
        }

        if total_fee_bps > Uint64::from(10000u64) {
            return Err(ContractError::TotalFeeBpsTooHigh {});
        }

        Ok(())
    }

    pub fn receive_nft(
        &mut self,
        token_id: String,
        owner: Addr,
        contract_addr: Addr,
    ) -> Result<(), ContractError> {
        if self.nft.received {
            return Err(ContractError::NftAlreadyReceived {});
        }

        if self.nft.token_id != token_id {
            return Err(ContractError::NftTokenIdMismatch {});
        }

        if self.nft.owner != owner {
            return Err(ContractError::NftOwnerMismatch {});
        }

        if self.nft.contract_addr != contract_addr {
            return Err(ContractError::NftContractAddrMismatch {});
        }

        self.nft.received = true;

        Ok(())
    }

    pub fn place_bid(
        &mut self,
        bidder: Addr,
        amount: Uint128,
        now: Timestamp,
    ) -> Result<(), ContractError> {
        if now < self.start_time {
            return Err(ContractError::AuctionNotStarted {});
        }

        if now >= self.end_time {
            return Err(ContractError::AuctionEnded {});
        }

        if amount < self.reserve_price {
            return Err(ContractError::BidBelowReservePrice {});
        }

        if !self.nft.received {
            return Err(ContractError::NftNotReceived {});
        }

        if let Some(current_bid) = &self.current_bid {
            if amount <= current_bid.amount {
                return Err(ContractError::BidAmountTooLow {});
            }

            if self.dynamic_bidding_extension.enabled
                && amount < current_bid.amount + self.dynamic_bidding_extension.min_bid_increase
            {
                return Err(ContractError::BidAmountBelowMinIncrease {});
            }
        }

        let bid = Bid::new(bidder, amount, now);

        if self.dynamic_bidding_extension.enabled {
            self.end_time =
                now.plus_seconds(self.dynamic_bidding_extension.time_extension_secs.u64());
        }

        self.current_bid = Some(bid);
        Ok(())
    }

    pub fn end_auction(&mut self, now: Timestamp) -> Result<Option<Bid>, ContractError> {
        if now < self.end_time {
            return Err(ContractError::AuctionNotEnded {});
        }

        let winning_bid = self.current_bid.clone();
        self.winning_bid = winning_bid.clone();
        self.current_bid = None;
        Ok(winning_bid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Timestamp;

    fn create_test_auction() -> Auction {
        let nft = Nft::new(
            1.to_string(),
            Addr::unchecked("Alice"),
            Addr::unchecked("nft"),
        );
        let start_time = Timestamp::from_seconds(100000);
        let end_time = start_time.plus_seconds(60 * 5); // 5 minutes from now
        let reserve_price = Uint128::new(100);
        let dynamic_bidding_extension =
            DynamicBiddingExtension::new(true, Uint64::new(30), 10_u128.into());

        Auction::new(
            nft,
            start_time,
            end_time,
            reserve_price,
            dynamic_bidding_extension,
            "ujuno".to_string(),
            vec![],
            Addr::unchecked("payout"),
        )
    }

    fn receive_test_nft(auction: &mut Auction) {
        auction
            .receive_nft(
                auction.nft.token_id.clone(),
                auction.nft.owner.clone(),
                auction.nft.contract_addr.clone(),
            )
            .unwrap();
    }

    #[test]
    fn test_place_bid_before_start_time() {
        let mut auction = create_test_auction();
        receive_test_nft(&mut auction);
        let now = auction.start_time;
        auction.start_time = auction.start_time.plus_seconds(60 * 2); // Start 2 minutes from now

        let result = auction.place_bid(Addr::unchecked("Bob"), 120_u128.into(), now);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::AuctionNotStarted {});
    }

    #[test]
    fn test_place_bid_below_reserve_price() {
        let mut auction = create_test_auction();
        receive_test_nft(&mut auction);
        let now = auction.start_time;
        let result = auction.place_bid(Addr::unchecked("Bob"), 90_u128.into(), now);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::BidBelowReservePrice {});
    }

    #[test]
    fn test_place_bid_success() {
        let mut auction = create_test_auction();
        receive_test_nft(&mut auction);
        let now = auction.start_time;
        let result = auction.place_bid(Addr::unchecked("Bob"), 120_u128.into(), now);
        assert!(result.is_ok());

        let current_bid = auction.current_bid.unwrap();
        assert_eq!(current_bid.bidder, "Bob");
        assert_eq!(current_bid.amount, Uint128::from(120_u128));
    }

    #[test]
    fn test_place_bid_lower_than_current_bid() {
        let mut auction = create_test_auction();
        receive_test_nft(&mut auction);
        let now = auction.start_time;
        auction
            .place_bid(Addr::unchecked("Bob"), 120_u128.into(), now)
            .unwrap();

        let result = auction.place_bid(Addr::unchecked("Charlie"), 110_u128.into(), now);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::BidAmountTooLow {});
    }

    #[test]
    fn test_place_bid_after_end_time() {
        let mut auction = create_test_auction();
        receive_test_nft(&mut auction);
        let now = auction.start_time;
        auction.start_time = auction.start_time.minus_seconds(60 * 2); // Start 2 minutes ago
        auction.end_time = auction.start_time.plus_seconds(60); // End 1 minute ago

        let result = auction.place_bid(Addr::unchecked("Bob"), 120_u128.into(), now);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::AuctionEnded {});
    }

    #[test]
    fn test_receive_nft_already_received() {
        let mut auction = create_test_auction();
        receive_test_nft(&mut auction);

        let result = auction.receive_nft(
            auction.nft.token_id.clone(),
            auction.nft.owner.clone(),
            auction.nft.contract_addr.clone(),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::NftAlreadyReceived {});
    }

    #[test]
    fn test_receive_nft_wrong_token_id() {
        let mut auction = create_test_auction();
        let result = auction.receive_nft(
            "2".to_string(),
            auction.nft.owner.clone(),
            auction.nft.contract_addr.clone(),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::NftTokenIdMismatch {});
    }

    #[test]
    fn test_receive_nft_wrong_owner() {
        let mut auction = create_test_auction();
        let result = auction.receive_nft(
            auction.nft.token_id.clone(),
            Addr::unchecked("Bob"),
            auction.nft.contract_addr.clone(),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::NftOwnerMismatch {});
    }

    #[test]
    fn test_receive_nft_wrong_contract_addr() {
        let mut auction = create_test_auction();
        let result = auction.receive_nft(
            auction.nft.token_id.clone(),
            auction.nft.owner.clone(),
            Addr::unchecked("Bob"),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ContractError::NftContractAddrMismatch {}
        );
    }

    #[test]
    fn test_receive_nft_success() {
        let mut auction = create_test_auction();
        let result = auction.receive_nft(
            auction.nft.token_id.clone(),
            auction.nft.owner.clone(),
            auction.nft.contract_addr.clone(),
        );
        assert!(result.is_ok());
        assert!(auction.nft.received);
    }

    #[test]
    fn test_place_bid_before_nft_received() {
        let mut auction = create_test_auction();
        let now = auction.start_time;
        auction.nft.received = false;

        let result = auction.place_bid(Addr::unchecked("Bob"), 120_u128.into(), now);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::NftNotReceived {});
    }

    #[test]
    fn test_end_auction_before_end_time() {
        let mut auction = create_test_auction();
        let now = auction.start_time;
        let result = auction.end_auction(now);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::AuctionNotEnded {});
    }

    #[test]
    fn test_end_auction_success() {
        let mut auction = create_test_auction();
        receive_test_nft(&mut auction);
        let now = auction.start_time;
        auction
            .place_bid(Addr::unchecked("Bob"), 120_u128.into(), now)
            .unwrap();
        auction.end_time = now; // Set end time to now

        let result = auction.end_auction(now);
        assert!(result.is_ok());
        let winning_bid = result.unwrap();
        assert!(winning_bid.is_some());

        let bid = winning_bid.unwrap();
        assert_eq!(bid.bidder, "Bob");
        assert_eq!(bid.amount, Uint128::from(120_u128));
        assert_eq!(bid, auction.winning_bid.unwrap());
    }

    #[test]
    fn test_fee_validation_success() {
        let fee = Fee::new(
            Uint64::from(5000u64),
            Addr::unchecked("some_address"),
            "valid_label".to_string(),
        );
        assert!(fee.validate().is_ok());
    }

    #[test]
    fn test_fee_validation_bps_too_high() {
        let fee = Fee::new(
            Uint64::from(10001u64),
            Addr::unchecked("some_address"),
            "valid_label".to_string(),
        );
        assert_eq!(fee.validate(), Err(ContractError::FeeBpsTooHigh {}));
    }

    #[test]
    fn test_fee_validation_label_too_long() {
        let fee = Fee::new(
            Uint64::from(5000u64),
            Addr::unchecked("some_address"),
            "label_is_too_long123".to_string(),
        );
        assert_eq!(fee.validate(), Err(ContractError::FeeLabelTooLong {}));
    }

    #[test]
    fn test_fee_validation_label_too_short() {
        let fee = Fee::new(
            Uint64::from(5000u64),
            Addr::unchecked("some_address"),
            "".to_string(),
        );
        assert_eq!(fee.validate(), Err(ContractError::FeeLabelTooShort {}));
    }

    #[test]
    fn test_fee_validation_label_invalid_characters() {
        let fee = Fee::new(
            Uint64::from(5000u64),
            Addr::unchecked("some_address"),
            "invalid*label".to_string(),
        );
        assert_eq!(
            fee.validate(),
            Err(ContractError::FeeLabelInvalidCharacters {})
        );
    }

    #[test]
    fn test_fee_validation_label_not_lowercase() {
        let fee = Fee::new(
            Uint64::from(5000u64),
            Addr::unchecked("some_address"),
            "InvalidLabel".to_string(),
        );
        assert_eq!(fee.validate(), Err(ContractError::FeeLabelNotLowerCase {}));
    }

    fn sample_nft() -> Nft {
        Nft::new(
            "token_id".to_string(),
            Addr::unchecked("owner"),
            Addr::unchecked("contract_addr"),
        )
    }

    fn sample_dynamic_bidding_extension() -> DynamicBiddingExtension {
        DynamicBiddingExtension::new(true, Uint64::from(60u64), Uint128::from(10u64))
    }

    fn sample_fees() -> Vec<Fee> {
        vec![
            Fee::new(
                Uint64::from(250u64),
                Addr::unchecked("fee_address1"),
                "fee_label1".to_string(),
            ),
            Fee::new(
                Uint64::from(250u64),
                Addr::unchecked("fee_address2"),
                "fee_label2".to_string(),
            ),
        ]
    }

    #[test]
    fn test_auction_validation_success() {
        let auction = Auction::new(
            sample_nft(),
            Timestamp::from_seconds(100),
            Timestamp::from_seconds(200),
            Uint128::from(100u64),
            sample_dynamic_bidding_extension(),
            "ust".to_string(),
            sample_fees(),
            Addr::unchecked("payout"),
        );
        assert!(auction.validate().is_ok());
    }

    #[test]
    fn test_auction_validation_end_time_before_start_time() {
        let auction = Auction::new(
            sample_nft(),
            Timestamp::from_seconds(200),
            Timestamp::from_seconds(100),
            Uint128::from(100u64),
            sample_dynamic_bidding_extension(),
            "ust".to_string(),
            sample_fees(),
            Addr::unchecked("payout"),
        );
        assert_eq!(
            auction.validate(),
            Err(ContractError::AuctionEndTimeBeforeStartTime {})
        );
    }

    #[test]
    fn test_auction_validation_min_bid_increase_zero() {
        let auction = Auction::new(
            sample_nft(),
            Timestamp::from_seconds(100),
            Timestamp::from_seconds(200),
            Uint128::from(100u64),
            DynamicBiddingExtension::new(true, Uint64::from(60u64), Uint128::from(0u64)),
            "ust".to_string(),
            sample_fees(),
            Addr::unchecked("payout"),
        );
        assert_eq!(
            auction.validate(),
            Err(ContractError::MinBidIncreaseZero {})
        );
    }

    #[test]
    fn test_auction_validation_too_many_fees() {
        let fees = (0..11)
            .map(|i| {
                Fee::new(
                    Uint64::from(100u64),
                    Addr::unchecked(format!("fee_address{}", i)),
                    format!("fee_label{}", i),
                )
            })
            .collect::<Vec<Fee>>();

        let auction = Auction::new(
            sample_nft(),
            Timestamp::from_seconds(100),
            Timestamp::from_seconds(200),
            Uint128::from(100u64),
            sample_dynamic_bidding_extension(),
            "ust".to_string(),
            fees,
            Addr::unchecked("payout"),
        );
        assert_eq!(auction.validate(), Err(ContractError::TooManyFees {}));
    }

    #[test]
    fn test_auction_validation_total_fee_bps_too_high() {
        let fees = vec![
            Fee::new(
                Uint64::from(9000u64),
                Addr::unchecked("fee_address1"),
                "fee_label1".to_string(),
            ),
            Fee::new(
                Uint64::from(2000u64),
                Addr::unchecked("fee_address2"),
                "fee_label2".to_string(),
            ),
        ];

        let auction = Auction::new(
            sample_nft(),
            Timestamp::from_seconds(100),
            Timestamp::from_seconds(200),
            Uint128::from(100u64),
            sample_dynamic_bidding_extension(),
            "ust".to_string(),
            fees,
            Addr::unchecked("payout"),
        );
        assert_eq!(
            auction.validate(),
            Err(ContractError::TotalFeeBpsTooHigh {})
        );
    }
}
