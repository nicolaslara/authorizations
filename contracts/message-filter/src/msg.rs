use cosmwasm_std::{Addr, CustomMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Kind, Matcher};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub parent: Addr,
    pub kind: Kind,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddAuthorization { addr: Addr, msg: String },
    RemoveAuthorization { addr: Addr, msg: String },
}

impl CustomMsg for ExecuteMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAuthorizations { sender: Addr },
}
impl CustomMsg for QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AuthorizationsResponse {
    pub authorizations: Vec<Matcher>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
