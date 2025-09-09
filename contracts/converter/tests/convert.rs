use crate::common::*;
use cosmwasm_std::{coin, Coin};
use rstest::*;

mod common;

#[rstest]
#[case::no_funds(&[], Expect::ErrContains(INVALID_FUNDS))]
#[case::zero_funds(&[coin(0, DEFAULT_SOURCE_DENOM)], Expect::ErrContains(ZERO_FUNDS))]
#[case::one(&[coin(1, DEFAULT_SOURCE_DENOM)], Expect::ErrContains(RESULT_IS_ZERO))]
#[case::same_denom(&[coin(100, DEFAULT_TARGET_DENOM)], Expect::ErrContains(INVALID_SOURCE_DENOM))]
#[case::multi_funds(&[default_convert_amount(), coin(500, DUMMY_DENOM)], Expect::ErrContains(INVALID_FUNDS))]
fn execute_convert_invalid_funds(
    setup_with_funds: (AppAccepting, u64),
    #[case] funds: &[Coin],
    #[case] expect: Expect<'_>,
) {
    prepare_and_execute(
        setup_with_funds,
        default_sender(),
        &default_instantiate(),
        &[],
        default_sender(),
        &default_convert(),
        funds,
        expect,
    );
}

#[rstest]
#[case::invalid_sender(INVALID_MANIFEST_ADDRESS, Expect::ErrContains(CANNOT_SUB))]
#[case::empty_sender("", Expect::ErrContains(CANNOT_SUB))]
fn execute_convert_invalid_sender(
    setup_with_funds: (AppAccepting, u64),
    #[case] sender: &str,
    #[case] expect: Expect<'_>,
) {
    prepare_and_execute(
        setup_with_funds,
        default_sender(),
        &default_instantiate(),
        &[],
        sender,
        &default_convert(),
        &[default_convert_amount()],
        expect,
    );
}

#[rstest]
#[case::invalid_message(serde_json::json!({"invalid":{}}), Expect::ErrContains(UNKNOWN_VARIANT))]
#[case::empty_message(serde_json::json!({}), Expect::ErrContains(EXPECTED_VALUE))]
fn execute_convert_invalid_message(
    setup_with_funds: (AppAccepting, u64),
    #[case] exec_msg: impl serde::Serialize + std::fmt::Debug,
    #[case] expect: Expect<'_>,
) {
    prepare_and_execute(
        setup_with_funds,
        default_sender(),
        &default_instantiate(),
        &[],
        default_sender(),
        &exec_msg,
        &[default_convert_amount()],
        expect,
    );
}

#[rstest]
fn execute_convert_when_paused(setup_with_funds: (AppAccepting, u64)) {
    let mut instantiate_msg = default_instantiate();
    instantiate_msg["paused"] = serde_json::json!(true);
    prepare_and_execute(
        setup_with_funds,
        default_sender(),
        &instantiate_msg,
        &[],
        default_sender(),
        &default_convert(),
        &[default_convert_amount()],
        Expect::ErrContains(CONTRACT_PAUSED),
    );
}

#[rstest]
#[case::ten(coin(10, DEFAULT_SOURCE_DENOM))]
fn execute_convert_ok(setup_with_funds: (AppAccepting, u64), #[case] funds: Coin) {
    prepare_and_execute(
        setup_with_funds,
        default_sender(),
        &default_instantiate(),
        &[],
        default_sender(),
        &default_convert(),
        &[funds],
        Expect::Ok,
    );
}
