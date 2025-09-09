use crate::common::*;
use rstest::*;
use serde_json::to_value;

mod common;

#[rstest]
#[case::default(default_config())]
#[case::poa_admin(modify_config(Field::PoaAdmin, DEFAULT_SENDER))]
#[case::rate(modify_config(Field::Rate, "1.5"))]
#[case::src_denom(modify_config(Field::SourceDenom, "uatom"))]
#[case::tgt_denom(modify_config(Field::TargetDenom, "uosmo"))]
#[case::paused(modify_config(Field::Paused, true))]
fn query_config(setup_with_funds: (AppAccepting, u64), #[case] config: impl serde::Serialize) {
    let exec_msg = create_msg_update_config_from_config(&config);
    let (app, contract_addr, _code_id) = prepare_and_execute(
        setup_with_funds,
        default_admin(),
        &default_instantiate(),
        &[],
        default_admin(),
        &exec_msg,
        &[],
        Expect::Ok,
    );

    let query_msg = serde_json::json!({"config": {}});
    let res: serde_json::Value = app
        .wrap()
        .query_wasm_smart(contract_addr, &query_msg)
        .unwrap();
    assert_eq!(res, to_value(config).unwrap());
}

#[rstest]
#[case::default_admin(DEFAULT_POA_ADMIN)]
fn query_admin(setup_with_funds: (AppAccepting, u64), #[case] admin: &str) {
    let exec_msg = create_msg_update_admin(Some(admin));
    let (app, contract_addr, _code_id) = prepare_and_execute(
        setup_with_funds,
        default_admin(),
        &default_instantiate(),
        &[],
        default_admin(),
        &exec_msg,
        &[],
        Expect::Ok,
    );

    let query_msg = serde_json::json!({"admin": {}});
    let res: serde_json::Value = app
        .wrap()
        .query_wasm_smart(contract_addr, &query_msg)
        .unwrap();
    assert_eq!(res, serde_json::json!({"admin": default_admin()}));
}
