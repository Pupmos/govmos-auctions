junod tx wasm store ./artifacts/nft_auction.wasm \
  --node https://juno-rpc.polkachu.com:443 \
  --chain-id juno-1 --from flix-pupmos-burner \
  --gas-prices 1.5ujuno --gas 5000000 \
  --gas-adjustment 1.5 -y -b block

  # 899.9K $TOKEN