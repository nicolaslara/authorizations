use cosmwasm_std::{Addr, CosmosMsg, CustomMsg, Empty};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthoriazationExecuteMsg<ExecuteExt = Empty>
where
    ExecuteExt: CustomMsg,
{
    /// Some authorizations may want to track information about the users or
    /// messages to determine if they authorize or not. This message should be
    /// sent every time the authorizations are successfully used so that
    /// sub-authorizations can update their internal state.
    UpdateExecutedAuthorizationState {
        msgs: Vec<CosmosMsg>,
        sender: Addr,
    },

    // Extensions allow implementors to add their own custom messages to the contract
    Extension(ExecuteExt),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthoriazationQueryMsg<QueryExt = Empty>
where
    QueryExt: CustomMsg,
{
    IsAuthorized { msgs: Vec<CosmosMsg>, sender: Addr },

    // Extensions allow implementors to add their own custom messages to the contract
    Extension(QueryExt),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsAuthorizedResponse {
    pub authorized: bool,
}
