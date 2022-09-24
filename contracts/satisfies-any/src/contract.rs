#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Reply, Response,
    StdError, StdResult,
};
use cw2::set_contract_version;
use cw_authorizations::msg::{
    AuthoriazationExecuteMsg, AuthoriazationQueryMsg, IsAuthorizedResponse,
};
use cw_authorizations::{Authorization, AuthorizationError};

use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::ProxyState;

const CONTRACT_NAME: &str = "crates.io:satisfies-any";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct SatisfiesAnyContract {
    state: ProxyState,
}

impl SatisfiesAnyContract {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        msg: InstantiateMsg,
    ) -> Result<(), AuthorizationError> {
        self.state.admin.save(deps.storage, &msg.admin)?;
        self.state.parent.save(deps.storage, &msg.parent)?;
        for child in msg.children {
            self.state.children.save(deps.storage, child, &Empty {})?;
        }
        Ok(())
    }
}

impl Authorization<ExecuteMsg> for SatisfiesAnyContract {
    fn new() -> Self {
        SatisfiesAnyContract {
            state: ProxyState::new(),
        }
    }

    fn is_authorized(
        &self,
        deps: Deps,
        msgs: &Vec<CosmosMsg>,
        sender: &Addr,
    ) -> Result<bool, AuthorizationError> {
        // Right now this defaults to an *or*. We could update the contract to
        // support a custom allow/reject behaviour (similarly to how it's done in
        // message-filter)
        let children: Vec<(Addr, Empty)> = self
            .state
            .children
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<Result<_, StdError>>()?;

        if children.is_empty() {
            return Ok(false);
        }

        Ok(children.into_iter().map(|c| c.0).any(|a| {
            deps.querier
                .query_wasm_smart(
                    a.clone(),
                    &AuthoriazationQueryMsg::IsAuthorized::<Empty> {
                        msgs: msgs.clone(),
                        sender: sender.clone(),
                    },
                )
                .unwrap_or(IsAuthorizedResponse { authorized: false })
                .authorized
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

    fn update_authorization_state(
        &self,
        deps: Deps,
        msgs: &Vec<CosmosMsg>,
        sender: &Addr,
        real_sender: &Addr,
    ) -> Result<Response, AuthorizationError> {
        let parent = self.state.parent.load(deps.storage)?;
        if sender != real_sender && real_sender != &parent {
            return Err(AuthorizationError::Unauthorized {
            //reason: Some("Auth updates that aren't triggered by a parent contract cannot specify a sender other than the caller ".to_string()),
            });
        }

        // If at least one authorization module authorized this message, we send the
        // Authorize execute message to all the authorizations so that they can update their
        // state if needed.
        if self.is_authorized(deps, &msgs, &sender)? {
            let sub_msgs = self.generate_child_update_msgs(deps, msgs, sender)?;
            Ok(Response::default().add_submessages(sub_msgs))
        } else {
            Err(AuthorizationError::Unauthorized {
                //reason: Some("No sub authorization passed".to_string()),
            })
        }
    }

    fn execute_extension(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, AuthorizationError> {
        if info.sender != self.state.admin.load(deps.storage)? {
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
    SatisfiesAnyContract::new().instantiate(deps, msg)?;
    Ok(Response::default().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AuthoriazationExecuteMsg<ExecuteMsg>,
) -> Result<Response, AuthorizationError> {
    SatisfiesAnyContract::new().execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: AuthoriazationQueryMsg) -> StdResult<Binary> {
    SatisfiesAnyContract::new().query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, AuthorizationError> {
    let contract = SatisfiesAnyContract::new();
    match msg.id {
        // Update reply errors are always ignored.
        id if id == contract.get_update_reply_id() => contract.sub_message_reply(msg),
        id => Err(AuthorizationError::Std(
            cosmwasm_std::StdError::GenericErr {
                msg: format!("Unknown reply id: {}", id),
            },
        )),
    }
}
