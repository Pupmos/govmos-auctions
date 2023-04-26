# dutch-auction-vault

<!-- checklist -->
- [X] Instantiate only defines market configuration parameters
  - [X] seller_address (optional)
  - [X] seller_token_id (optional)
    - [X] when ReceiveNft is called, set seller_address to msg.msg.sender.
  - [X] token info
  - [X] Market doesnt exist until auction is scheduled
- [X] Auction can be scheduled by seller or by nft owner
  - [X] struct Seller(Addr | NftOwner { contract_addr, token_id })
- [X] Funding
  - [X] by sending funds. this amount is locked in when auction is scheduled
- [X] Buying 
  - [X] Assert enough tokens sent
  - [X] Send back change
  - [X] Keep track of total spent
  - [X] Send units to buyer
- [X] Withdrawing
  - [X] Assert seller 
  - [X] Subtract withdrawn amount from total spent
  - [X] Update withdrawn amount
  - [X] Send proceeds to seller