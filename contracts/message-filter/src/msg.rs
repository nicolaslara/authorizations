use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CustomMsg};

use crate::state::{Kind, Matcher};

#[cw_serde]
pub struct InstantiateMsg {
    pub parent: Addr,
    pub kind: Kind,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddAuthorization { addr: Addr, msg: String },
    RemoveAuthorization { addr: Addr, msg: String },
}

impl CustomMsg for ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AuthorizationsResponse)]
    GetAuthorizations { sender: Addr },
}
impl CustomMsg for QueryMsg {}

#[cw_serde]
pub struct AuthorizationsResponse {
    pub authorizations: Vec<Matcher>,
}

#[cw_serde]
pub enum MigrateMsg {}
