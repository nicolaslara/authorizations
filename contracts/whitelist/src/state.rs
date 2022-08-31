use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub struct WhitelistState {
    pub owner: Item<'static, Addr>,
    pub authorized: Map<'static, String, cosmwasm_std::Empty>,
}

impl WhitelistState {
    pub fn new() -> Self {
        WhitelistState {
            owner: Item::new("dao"),
            authorized: Map::new("authorized"),
        }
    }
}
