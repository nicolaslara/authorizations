use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub struct ProxyState {
    pub admin: Item<'static, Addr>,
    pub parent: Item<'static, Addr>,
    pub children: Map<'static, Addr, cosmwasm_std::Empty>,
}

impl ProxyState {
    pub const fn new() -> Self {
        ProxyState {
            admin: Item::new("admin"),
            parent: Item::new("parent"),
            children: Map::new("children"),
        }
    }
}
