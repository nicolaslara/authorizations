#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use cw_authorizations::msg::{AuthoriazationExecuteMsg, AuthoriazationQueryMsg};
use cw_authorizations::{Authorization, AuthorizationError};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::WhitelistState;

const CONTRACT_NAME: &str = "crates.io:whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct WhitelistContract {
    state: WhitelistState,
}

impl Authorization<ExecuteMsg> for WhitelistContract {
    fn new() -> Self {
        WhitelistContract {
            state: WhitelistState::new(),
        }
    }

    fn is_authorized(
        &self,
        deps: Deps,
        _msgs: &Vec<CosmosMsg>,
        sender: &Addr,
    ) -> Result<bool, AuthorizationError> {
        Ok(self
            .state
            .authorized
            .may_load(deps.storage, sender.to_string())?
            .is_some())
    }

    fn get_sub_authorizations(&self, _deps: Deps) -> Result<Vec<Addr>, AuthorizationError> {
        Ok(vec![])
    }

    fn execute_extension(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, AuthorizationError> {
        match msg {
            ExecuteMsg::Allow { addr } => {
                if info.sender != self.state.owner.load(deps.storage)? {
                    return Err(AuthorizationError::Unauthorized {
                        //reason: Some("Only the dao can add authorizations".to_string()),
                    }
                    .into());
                }
                self.state.authorized.save(deps.storage, addr, &Empty {})?;
                Ok(Response::default().add_attribute("action", "allow"))
            }
            ExecuteMsg::Remove { addr } => {
                if info.sender != self.state.owner.load(deps.storage)? {
                    return Err(AuthorizationError::Unauthorized {
                     //   reason: Some("Only the dao can remove authorizations".to_string()),
                    }
                    .into());
                }
                self.state.authorized.remove(deps.storage, addr);
                Ok(Response::default().add_attribute("action", "remove"))
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let contract = WhitelistContract::new();
    contract.state.owner.save(deps.storage, &msg.dao)?;
    Ok(Response::default().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AuthoriazationExecuteMsg<ExecuteMsg>,
) -> Result<Response, AuthorizationError> {
    WhitelistContract::new().execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: AuthoriazationQueryMsg) -> StdResult<Binary> {
    WhitelistContract::new().query(deps, env, msg)
}
