use crate::common::*;
use rstest::*;

mod common;

fn create_msg_update_admin(new_admin: Option<&str>) -> impl serde::Serialize + std::fmt::Debug {
    serde_json::json!({"update_admin": {"admin": new_admin}})
}

#[rstest]
#[case::ok(
    DEFAULT_POA_ADMIN,
    create_msg_update_admin(Some(DEFAULT_SENDER)),
    Expect::Ok
)]
#[case::unauthorized(
    "unauthorized",
    create_msg_update_admin(Some(DEFAULT_SENDER)),
    Expect::ErrContains(ONLY_ADMIN)
)]
#[case::invalid_admin(
    DEFAULT_POA_ADMIN,
    create_msg_update_admin(Some("invalid_address")),
    Expect::ErrContains(PARSE_FAILED)
)]
#[case::null_admin(
    DEFAULT_POA_ADMIN,
    create_msg_update_admin(None),
    Expect::ErrContains(CANNOT_RENOUNCE)
)]
fn update_admin(
    setup_with_funds: (AppAccepting, u64),
    default_instantiate: impl serde::Serialize,
    default_sender: &str,
    #[case] exec_sender: &str,
    #[case] exec_msg: impl serde::Serialize + std::fmt::Debug,
    #[case] expect: Expect<'_>,
) {
    prepare_and_execute(
        setup_with_funds,
        default_sender,
        &default_instantiate,
        &[],
        exec_sender,
        &exec_msg,
        &[],
        expect,
    );
}
