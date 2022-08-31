#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;
use cw_authorizations::msg::{AuthoriazationExecuteMsg, AuthoriazationQueryMsg};
use cw_authorizations::{Authorization, AuthorizationError};

use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::ProxyState;

const CONTRACT_NAME: &str = "crates.io:whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct SatisfiesAllContract {
    state: ProxyState,
}

impl SatisfiesAllContract {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        msg: InstantiateMsg,
    ) -> Result<(), AuthorizationError> {
        self.state.parent.save(deps.storage, &msg.parent)?;
        for child in msg.children {
            self.state.children.save(deps.storage, child, &Empty {})?;
        }
        Ok(())
    }
}

impl Authorization<ExecuteMsg> for SatisfiesAllContract {
    fn new() -> Self {
        SatisfiesAllContract {
            state: ProxyState::new(),
        }
    }

    fn is_authorized(
        &self,
        deps: Deps,
        msgs: &Vec<CosmosMsg>,
        sender: &Addr,
    ) -> Result<bool, AuthorizationError> {
        let children: Vec<(Addr, Empty)> = self
            .state
            .children
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<Result<_, StdError>>()?;

        // This checks all the registered authorizations return true
        Ok(children.into_iter().map(|c| c.0).all(|a| {
            deps.querier
                .query_wasm_smart(
                    a.clone(),
                    &AuthoriazationQueryMsg::IsAuthorized::<Empty> {
                        msgs: msgs.to_owned(),
                        sender: sender.to_owned(),
                    },
                )
                .unwrap_or(false)
        }))
    }

    fn get_sub_authorizations(&self, deps: Deps) -> Result<Vec<Addr>, AuthorizationError> {
        Ok(self
            .state
            .children
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|e| e.ok())
            .map(|e| e.0)
            .collect())
    }

    fn execute_extension(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, AuthorizationError> {
        if info.sender != self.state.parent.load(deps.storage)? {
            return Err(AuthorizationError::Unauthorized {
                //reason: Some("Only the parent can execute on this contract".to_string()),
            }
            .into());
        }

        match msg {
            ExecuteMsg::AddChild { addr } => {
                self.state.children.save(deps.storage, addr, &Empty {})?;
                Ok(Response::default().add_attribute("action", "allow"))
            }
            ExecuteMsg::RemoveChild { addr } => {
                self.state.children.remove(deps.storage, addr);
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
) -> Result<Response, AuthorizationError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    SatisfiesAllContract::new().instantiate(deps, msg)?;
    Ok(Response::default().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AuthoriazationExecuteMsg<ExecuteMsg>,
) -> Result<Response, AuthorizationError> {
    SatisfiesAllContract::new().execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: AuthoriazationQueryMsg) -> StdResult<Binary> {
    SatisfiesAllContract::new().query(deps, env, msg)
}
