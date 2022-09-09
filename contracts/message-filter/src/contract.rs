use cosmwasm_std::{entry_point, to_binary, Addr, CosmosMsg};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_authorizations::msg::{AuthoriazationExecuteMsg, AuthoriazationQueryMsg};
use cw_authorizations::{Authorization, AuthorizationError};

use crate::msg::{AuthorizationsResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Kind, Matcher, MessageFilterState};
use crate::utils::{deep_partial_match, msg_to_value, str_to_value};
use crate::MessageFilterError;

const CONTRACT_NAME: &str = "crates.io:whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct MessageFilterContract {
    state: MessageFilterState,
}

impl MessageFilterContract {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        msg: InstantiateMsg,
    ) -> Result<(), AuthorizationError> {
        let config = Config {
            parent: msg.parent,
            kind: msg.kind,
        };
        self.state.config.save(deps.storage, &config)?;
        Ok(())
    }
}

impl Authorization<ExecuteMsg, QueryMsg, MessageFilterError> for MessageFilterContract {
    fn new() -> Self {
        MessageFilterContract {
            state: MessageFilterState::new(),
        }
    }

    fn is_authorized(
        &self,
        deps: Deps,
        msgs: &Vec<CosmosMsg>,
        sender: &Addr,
    ) -> Result<bool, AuthorizationError<MessageFilterError>> {
        let config = self.state.config.load(deps.storage)?;
        let auths = self.state.matchers.load(deps.storage, sender.clone());

        // If there are no auths, return the default for each Kind
        if auths.is_err() {
            return Ok(config.default_authorization());
        }

        let auths = auths.unwrap();

        // check that all messages can be converted to values
        for m in msgs {
            msg_to_value(&m).map_err(|e| {
                AuthorizationError::ContractError(MessageFilterError::UnauthorizedBecause {
                    reason: e.to_string(),
                })
            })?;
        }
        // check that all auths can be converted to values
        for a in &auths {
            str_to_value(&a.matcher).map_err(|e| {
                AuthorizationError::ContractError(MessageFilterError::UnauthorizedBecause {
                    reason: e.to_string(),
                })
            })?;
        }

        let matched = auths.iter().any(|a| {
            msgs.iter().all(|m| {
                deep_partial_match(
                    &msg_to_value(&m).unwrap(),
                    &str_to_value(&a.matcher).unwrap(),
                )
            })
        });

        if matched {
            return match config.kind {
                Kind::Allow {} => Ok(true),
                Kind::Reject {} => Ok(false),
            };
        }
        Ok(config.default_authorization())
    }

    fn get_sub_authorizations(
        &self,
        _deps: Deps,
    ) -> Result<Vec<Addr>, AuthorizationError<MessageFilterError>> {
        Ok(vec![])
    }

    fn execute_extension(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, AuthorizationError<MessageFilterError>> {
        let config = self.state.config.load(deps.storage)?;
        if info.sender != config.parent {
            return Err(AuthorizationError::ContractError(
                MessageFilterError::UnauthorizedBecause {
                    reason: "Only the parent can add or remove authorizations on this contract"
                        .to_string(),
                },
            ));
        }

        match msg {
            ExecuteMsg::AddAuthorization { addr, msg } => {
                self.execute_add_authorization(deps, info, addr, msg)
            }
            ExecuteMsg::RemoveAuthorization { addr, msg } => {
                self.execute_remove_authorization(deps, info, addr, msg)
            }
        }
    }

    fn query_extension(&self, deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::GetAuthorizations { sender } => {
                let auths = self.state.matchers.may_load(deps.storage, sender)?;
                match auths {
                    Some(authorizations) => to_binary(&AuthorizationsResponse { authorizations }),
                    None => to_binary(&AuthorizationsResponse {
                        authorizations: vec![],
                    }),
                }
            }
        }
    }
}

impl MessageFilterContract {
    fn execute_add_authorization(
        &self,
        deps: DepsMut,
        _info: MessageInfo,
        authorized_addr: Addr,
        authorization_matcher: String,
    ) -> Result<Response, AuthorizationError<MessageFilterError>> {
        // If the message can't be converted to a string, we fail
        str_to_value(&authorization_matcher)?;
        self.state.matchers.update(
            deps.storage,
            authorized_addr.clone(),
            |auth: Option<Vec<Matcher>>| -> Result<Vec<Matcher>, AuthorizationError<MessageFilterError>> {
                let new_auth = Matcher {
                    addr: authorized_addr,
                    matcher: authorization_matcher,
                };
                match auth {
                    Some(mut auth) => {
                        auth.push(new_auth);
                        Ok(auth)
                    }
                    None => Ok(vec![new_auth]),
                }
            },
        )?;

        Ok(Response::default().add_attribute("action", "allow_message"))
    }

    fn execute_remove_authorization(
        &self,
        deps: DepsMut,
        _info: MessageInfo,
        authorized_addr: Addr,
        authorization_matcher: String,
    ) -> Result<Response, AuthorizationError<MessageFilterError>> {
        self.state.matchers.update(
            deps.storage,
            authorized_addr.clone(),
            |auth: Option<Vec<Matcher>>| -> Result<Vec<Matcher>, AuthorizationError<MessageFilterError>> {
                match auth {
                    Some(mut auth) => {
                        let i = auth
                            .iter()
                            .position(|x| *x.matcher == authorization_matcher);
                        if i.is_none() {
                            return Err(AuthorizationError::ContractError(
                                MessageFilterError::NotFound {},
                            ))
                        }
                        auth.remove(i.unwrap());
                        Ok(auth)
                    }
                    None => {
                        Err(AuthorizationError::ContractError(
                            MessageFilterError::NotFound {},
                        ))
                    }
                }
            },
        )?;
        Ok(Response::default().add_attribute("action", "removed"))
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
    MessageFilterContract::new().instantiate(deps, msg)?;
    Ok(Response::default().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AuthoriazationExecuteMsg<ExecuteMsg>,
) -> Result<Response, AuthorizationError<MessageFilterError>> {
    MessageFilterContract::new().execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: AuthoriazationQueryMsg<QueryMsg>) -> StdResult<Binary> {
    MessageFilterContract::new().query(deps, env, msg)
}
