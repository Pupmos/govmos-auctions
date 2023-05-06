# #[cw_serde]
# pub struct InstantiateMsg {
#     /// how many spend_asset tokens are required to buy one unit_asset token
#     pub start_price: Decimal,
#     /// the minimum price for which the auction will sell
#     pub min_price: Decimal,
#     /// the duration of the auction in seconds
#     pub target_duration: Uint128,
#     /// the asset to be used for bids
#     pub spend_asset: FungibleAsset,
#     /// the asset to be bid on and sold
#     pub unit_asset: FungibleAsset,
#     /// the address of the seller or None if the seller is an NFT owner
#     pub seller_address: Option<String>,
#     /// the NFT to be sold or None if the seller is an address
#     pub seller_nft: Option<UncheckedNft>,
# }


# if a "dutch" flag is set, run this 

if [ "$1" == "dutch" ]; then
  junod tx wasm instantiate 2831 '{
    "start_price": "300.0",
    "min_price": "0.0001",
    "target_duration": "10800",
    "spend_asset": {
      "native": {
          "denom": "JUNO"
      }
    },
    "unit_asset": {
      "cw20": {
        "contract_addr": "juno1zkwveux7y6fmsr88atf3cyffx96p0c96qr8tgcsj7vfnhx7sal3s3zu3ps"
      }
    },
    "seller_address": "juno10z6sfeeqd3hfthhxc58le30q93zy4n03qcxtha"
  }' \
    --node https://juno-rpc.polkachu.com:443 \
    --chain-id juno-1 --from flix-pupmos-burner \
    --gas-prices 0.1ujuno --gas "auto" \
    --gas-adjustment 1.5 -y -b block \
    --label "dutch_auction" \
    --admin "juno10z6sfeeqd3hfthhxc58le30q93zy4n03qcxtha"
  exit 0
fi

# #[cw_serde]
# pub struct FeeUnverified {
#     pub bps: Uint64,
#     pub address: String,
#     pub label: String,
# }

# #[cw_serde]
# pub struct NftUnverified {
#     pub token_id: String,
#     pub owner: String,
#     pub contract_addr: String,
# }

# /// All times are in seconds since epoch

# #[cw_serde]
# pub struct InstantiateMsg {
#     pub nft: NftUnverified,
#     pub start_time: Uint64,
#     pub end_time: Uint64,
#     pub reserve_price: Uint64,
#     pub dynamic_bidding_extension: DynamicBiddingExtension,
#     pub denom: String,
#     pub fees: Vec<FeeUnverified>,
#     pub payout_address: String,
# }

# // Dynamic bidding extension configuration
# #[cw_serde]
# pub struct DynamicBiddingExtension {
#     pub enabled: bool,
#     pub time_extension_secs: Uint64,
#     pub min_bid_increase: Uint128,
# }

HOURS=24
# offset of 1 hour
OFFSET=$((1 * 60 * 60))
DATE=$(date +%s)
START_TIME=$(($DATE + $OFFSET))
DURATION=$(($HOURS * 60 * 60))
END_TIME=$(($START_TIME + $DURATION))

# date is current time in seconds since epoch plus 
# for start time use 

if [ "$1" == "nft" ]; then
  junod tx wasm instantiate 2832 '{
    "nft": {
      "token_id": "1",
      "owner": "juno1zkwveux7y6fmsr88atf3cyffx96p0c96qr8tgcsj7vfnhx7sal3s3zu3ps",
      "contract_addr": "juno1zkwveux7y6fmsr88atf3cyffx96p0c96qr8tgcsj7vfnhx7sal3s3zu3ps"
    },
    "start_time": "'$START_TIME'",
    "end_time": "'$END_TIME'",
    "reserve_price": "1",
    "dynamic_bidding_extension": {
      "enabled": true,
      "time_extension_secs": "120",
      "min_bid_increase": "1000000"
    },
    "denom": "ujuno",
    "fees": [
      {
        "bps": "500",
        "address": "juno10z6sfeeqd3hfthhxc58le30q93zy4n03qcxtha",
        "label": "creator"
      },
      {
        "bps": "500",
        "address": "juno1zkwveux7y6fmsr88atf3cyffx96p0c96qr8tgcsj7vfnhx7sal3s3zu3ps",
        "label": "dao"
      }
    ],
    "payout_address": "juno10z6sfeeqd3hfthhxc58le30q93zy4n03qcxtha"
  }' \
    --node https://juno-rpc.polkachu.com:443 \
    --chain-id juno-1 --from flix-pupmos-burner \
    --gas-prices 0.1ujuno --gas "auto" \
    --gas-adjustment 1.5 -y -b block \
    --label "nft_auction" \
    --admin "juno10z6sfeeqd3hfthhxc58le30q93zy4n03qcxtha"
  exit 0;
fi