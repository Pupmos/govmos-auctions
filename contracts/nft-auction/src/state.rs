use cw_storage_plus::Deque;
use cw_storage_plus::Item;

use crate::auction::{Auction, Bid};

pub const AUCTION_INFO: Item<Auction> = Item::new("auction_info");
pub const BID_HISTORY: Deque<Bid> = Deque::new("bid_history");
