use cosmwasm_std::{coin, Addr, Empty};
use cw_authorizations::msg::{
    AuthoriazationExecuteMsg, AuthoriazationQueryMsg, IsAuthorizedResponse,
};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

const CREATOR_ADDR: &str = "creator";

// Authorization Contracts
fn cw_basic_auth() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(crate::execute, crate::instantiate, crate::query);
    Box::new(contract)
}

#[test]
fn test_basic() {
    let init_funds = vec![coin(1000000, "juno"), coin(100, "other")];
    let mut app = App::new(|router, _, storage| {
        // initialization moved to App construction
        router
            .bank
            .init_balance(storage, &Addr::unchecked("McDuck"), init_funds)
            .unwrap();
    });

    // Create a proposal manager (gov module)
    let code_id = app.store_code(cw_basic_auth());

    // Create the DAO (core)
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(CREATOR_ADDR),
            &crate::InstantiateMsg {},
            &[],
            "Basic Auth",
            None,
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("Anyone"),
        contract_addr.clone(),
        &AuthoriazationExecuteMsg::UpdateExecutedAuthorizationState::<Empty> {
            msgs: vec![],
            sender: Addr::unchecked("Anyone"),
        },
        &[],
    )
    .unwrap();

    let response: IsAuthorizedResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &AuthoriazationQueryMsg::IsAuthorized::<Empty> {
                msgs: vec![],
                sender: Addr::unchecked("test"),
            },
        )
        .unwrap();
    assert!(response.authorized)
}
