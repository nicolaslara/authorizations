use cosmwasm_std::{Addr, Response};
use cw_authorizations::AuthorizationError;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Kind {
    Allow {},
    Reject {},
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Config {
    /// The address of the owner that this authorization module is associated
    /// with. This is usually another contract, which is why the name parent is
    /// used
    pub parent: Addr,
    /// The type of authorization this is. Kind::Allow means messages will only
    /// be authorized (allowed) if there is a matching Authorization in the
    /// contract. Kind::Reject means all messages will be authorized (allowed)
    /// by this contract unless explicitly rejected by one of the stored
    /// authorizations
    pub kind: Kind,
}

impl Config {
    pub fn default_response(&self) -> Result<Response, AuthorizationError> {
        match self.kind {
            Kind::Allow {} => Err(AuthorizationError::Unauthorized {
                //reason: Some("No authorizations allowed the request. Rejecting.".to_string()),
            }
            .into()),
            Kind::Reject {} => Ok(Response::default()
                .add_attribute("allowed", "true")
                .add_attribute(
                    "reason",
                    "No authorizations rejected the request. Allowing.",
                )),
        }
    }

    pub fn default_authorization(&self) -> bool {
        match self.kind {
            Kind::Allow {} => false,
            Kind::Reject {} => true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Matcher {
    /// A json representation of a CosmosMsg. Incomming messages will be
    /// recursively compared to the matcher to determine if they are authorized.
    ///
    /// To short-circuit the recursive comparison (i.e.: allow everything under
    /// an object key), you can use the empty object.
    ///
    /// For example:
    ///
    /// {"bank": {"to_address": "an_address", "amount":[{"denom": "juno", "amount": 1}]}}
    ///
    /// will match exactly that message but not a message where any of the fields are different.
    ///
    /// However, {"bank": {}} will match all bank messages, and
    /// {"bank": {"send": {"to_address": "an_address", "amount": {}}}} will match all bank messages to "an_address".
    ///
    pub matcher: String,
    /// The address of this matcher is applicable to.
    pub addr: Addr,
}

pub struct MessageFilterState {
    pub config: Item<'static, Config>,
    pub matchers: Map<'static, Addr, Vec<Matcher>>,
}

impl MessageFilterState {
    pub const fn new() -> Self {
        MessageFilterState {
            config: Item::new("config"),
            matchers: Map::new("matchers"),
        }
    }
}
