use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CustomMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub parent: Addr,
    pub children: Vec<Addr>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddChild { addr: Addr },
    RemoveChild { addr: Addr },
}

impl CustomMsg for ExecuteMsg {}

#[cw_serde]
pub enum QueryMsg {
    GetAuthorizations {},
}
