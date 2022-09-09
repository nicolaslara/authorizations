use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Binary, CosmosMsg, CustomMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Reply, ReplyOn, Response, StdResult, SubMsg,
};

use crate::error::{AuthorizationError, EmptyError};
use crate::msg;
use crate::msg::IsAuthorizedResponse;

const UPDATE_REPLY_ID: u64 = 1000;

pub trait Authorization<ExecuteExt = Empty, QueryExt = Empty, ErrorExt = EmptyError>
where
    ExecuteExt: CustomMsg,
    QueryExt: CustomMsg,
{
    // Required
    fn new() -> Self;
    fn get_sub_authorizations(&self, deps: Deps)
        -> Result<Vec<Addr>, AuthorizationError<ErrorExt>>;
    fn is_authorized(
        &self,
        deps: Deps,
        msgs: &Vec<CosmosMsg>,
        sender: &Addr,
    ) -> Result<bool, AuthorizationError<ErrorExt>>;

    fn query_authorizations(
        &self,
        deps: Deps,
        msgs: Vec<CosmosMsg>,
        sender: Addr,
    ) -> StdResult<Binary> {
        to_binary(&IsAuthorizedResponse {
            authorized: self.is_authorized(deps, &msgs, &sender).unwrap_or(false),
        })
    }

    // Useful
    fn update_own_state(
        &self,
        _deps: Deps,
        _msgs: &Vec<CosmosMsg>,
        _sender: &Addr,
        _original_sender: &Addr,
    ) -> Result<Response, AuthorizationError<ErrorExt>> {
        Ok(Response::default())
    }

    // Helpers
    fn get_update_reply_id(&self) -> u64 {
        UPDATE_REPLY_ID
    }

    fn reply_on(&self) -> ReplyOn {
        ReplyOn::Error
    }

    /// If this authorization has children. We need to generate messages to
    /// update all of its children. This function generates those messages. It
    /// is the responsibility of update_authorization_state() to pass those in
    /// the response
    fn generate_child_update_msgs(
        &self,
        deps: Deps,
        msgs: &Vec<CosmosMsg>,
        original_sender: &Addr,
    ) -> Result<Vec<SubMsg>, AuthorizationError<ErrorExt>> {
        let auths = self.get_sub_authorizations(deps)?;
        let msgs: Result<Vec<_>, _> = auths
            .iter()
            .map(|auth| -> Result<SubMsg, AuthorizationError<ErrorExt>> {
                // All errors from submessages are ignored by default. If they matter, implementors should take care of checking them.
                // In the future we may need a better way to handle updates
                Ok(SubMsg{
                    id: self.get_update_reply_id(),
                    msg: wasm_execute(
                        auth.to_string(),
                        &msg::AuthoriazationExecuteMsg::<ExecuteExt>::UpdateExecutedAuthorizationState {
                            msgs: msgs.clone(),
                            sender: original_sender.clone(),
                        },
                        vec![],
                    )?.into(),
                    reply_on: self.reply_on(),
                    gas_limit: None
                })
            })
            .collect();

        Ok(msgs?)
    }

    fn update_authorization_state(
        &self,
        deps: Deps,
        msgs: &Vec<CosmosMsg>,
        sender: &Addr,
        original_sender: &Addr,
    ) -> Result<Response, AuthorizationError<ErrorExt>> {
        let response = self.update_own_state(deps, msgs, sender, original_sender)?;
        Ok(response.add_submessages(self.generate_child_update_msgs(
            deps,
            msgs,
            original_sender,
        )?))
    }

    fn sub_message_reply(&self, msg: Reply) -> Result<Response, AuthorizationError<ErrorExt>> {
        if msg.result.is_err() {
            return Ok(Response::new().add_attribute("update_error", msg.result.unwrap_err()));
        }
        Ok(Response::new().add_attribute("update_success", format!("{:?}", msg.result.unwrap())))
    }

    // Entry Points
    fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: msg::AuthoriazationExecuteMsg<ExecuteExt>,
    ) -> Result<Response, AuthorizationError<ErrorExt>> {
        match msg {
            msg::AuthoriazationExecuteMsg::UpdateExecutedAuthorizationState { msgs, sender } => {
                self.update_authorization_state(deps.as_ref(), &msgs, &sender, &info.sender)
            }
            msg::AuthoriazationExecuteMsg::Extension(msg) => {
                self.execute_extension(deps, env, info, msg)
            }
        }
    }

    fn execute_extension(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: ExecuteExt,
    ) -> Result<Response, AuthorizationError<ErrorExt>> {
        unimplemented!()
    }

    fn query(
        &self,
        deps: Deps,
        env: Env,
        msg: msg::AuthoriazationQueryMsg<QueryExt>,
    ) -> StdResult<Binary> {
        match msg {
            msg::AuthoriazationQueryMsg::IsAuthorized { msgs, sender } => {
                self.query_authorizations(deps, msgs, sender)
            }
            msg::AuthoriazationQueryMsg::Extension(msg) => self.query_extension(deps, env, msg),
        }
    }

    fn query_extension(&self, _deps: Deps, _env: Env, _msg: QueryExt) -> StdResult<Binary> {
        unimplemented!()
    }
}
