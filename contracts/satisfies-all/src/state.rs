use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub struct ProxyState {
    pub parent: Item<'static, Addr>,
    pub children: Map<'static, Addr, cosmwasm_std::Empty>,
}

impl ProxyState {
    pub const fn new() -> Self {
        ProxyState {
            parent: Item::new("parent"),
            children: Map::new("children"),
        }
    }
}
