#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use cw_authorizations::msg::{AuthoriazationExecuteMsg, AuthoriazationQueryMsg};
use cw_authorizations::{Authorization, AuthorizationError};
use serde::{Deserialize, Serialize};

pub struct BasicAuth {}

impl Authorization for BasicAuth {
    fn new() -> Self {
        BasicAuth {}
    }

    fn is_authorized(
        &self,
        _deps: Deps,
        _msgs: &Vec<CosmosMsg>,
        _sender: &Addr,
    ) -> Result<bool, AuthorizationError> {
        Ok(true)
    }

    fn get_sub_authorizations(&self, _deps: Deps) -> Result<Vec<Addr>, AuthorizationError> {
        Ok(vec![])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, AuthorizationError> {
    Ok(Response::default().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AuthoriazationExecuteMsg,
) -> Result<Response, AuthorizationError> {
    BasicAuth::new().execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: AuthoriazationQueryMsg) -> StdResult<Binary> {
    BasicAuth::new().query(deps, env, msg)
}

#[cfg(test)]
mod tests;
