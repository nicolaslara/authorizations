use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CustomMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub dao: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Allow { addr: String },
    Remove { addr: String },
}

impl CustomMsg for ExecuteMsg {}

#[cw_serde]
pub enum MigrateMsg {}
