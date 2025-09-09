use crate::common::*;
use rstest::*;

mod common;

#[rustfmt::skip]
#[rstest]
// --- none: ok
#[case::ok_none_admin(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Admin, None::<&str>), Expect::Ok)]
#[case::ok_none_poa_admin(DEFAULT_POA_ADMIN, create_msg_update_config(Field::PoaAdmin, None::<&str>), Expect::Ok)]
#[case::ok_none_rate(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Rate, None::<u64>), Expect::Ok)]
#[case::ok_none_src_denom(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, None::<&str>), Expect::Ok)]
#[case::ok_none_tgt_denom(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, None::<&str>), Expect::Ok)]
#[case::ok_none_paused(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Paused, None::<bool>), Expect::Ok)]
// --- some: ok
#[case::ok_some_admin(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Admin, Some(DEFAULT_SENDER)), Expect::Ok)]
#[case::ok_some_poa_admin(DEFAULT_POA_ADMIN, create_msg_update_config(Field::PoaAdmin, Some(DEFAULT_SENDER)), Expect::Ok)]
#[case::ok_some_rate(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Rate, Some("1.5")), Expect::Ok)]
#[case::ok_some_src_denom(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some("uatom")), Expect::Ok)]
#[case::ok_some_tgt_denom(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some("uosmo")), Expect::Ok)]
#[case::ok_some_paused(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Paused, Some(true)), Expect::Ok)]
// --- noop: ok
#[case::ok_noop_admin(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Admin, Some(DEFAULT_POA_ADMIN)), Expect::Ok)]
#[case::ok_noop(DEFAULT_POA_ADMIN, create_msg_update_config_noop(), Expect::Ok)]
// --- unauthorized
#[case::unauthorized("unauthorized", create_msg_update_config(Field::Admin, Some(DEFAULT_SENDER)), Expect::ErrContains(ONLY_ADMIN))]
// --- same denom
#[case::same_denom_src(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some(DEFAULT_TARGET_DENOM)), Expect::ErrContains(SAME_DENOM))]
// --- invalid poa admin
#[case::invalid_poa_admin(DEFAULT_POA_ADMIN, create_msg_update_config(Field::PoaAdmin, Some("invalid_address")), Expect::ErrContains(PARSE_FAILED))]
// --- invalid rate
#[case::invalid_rate(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Rate, Some("invalid_rate")), Expect::ErrContains(RATE_PARSE_FAILED))]
#[case::invalid_rate_too_small(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Rate, Some(INVALID_RATE_MIN)), Expect::ErrContains(RATE_PARSE_FAILED))]
// --- invalid src denom
#[case::invalid_src_denom_empty(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some("")), Expect::ErrContains(EMPTY_DENOM))]
#[case::invalid_src_denom_unicode(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some("😀")), Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::invalid_src_denom_invalid_ibc(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some("ibc/a")), Expect::ErrContains(INVALID_IBC_DENOM_FORMAT))]
#[case::invalid_src_denom_invalid_factory(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some(INVALID_FACTORY_DENOM_NOT_ENOUGH_SEP)), Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::invalid_src_denom_factory_too_many_sep(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some(INVALID_FACTORY_DENOM_TOO_MANY_SEP)), Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::invalid_src_denom_factory_no_creator(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some(INVALID_FACTORY_DENOM_NO_CREATOR)), Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::invalid_src_denom_factory_empty_subdenom(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some(INVALID_FACTORY_DENOM_NO_SUBDENOM)), Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::invalid_src_denom_invalid_format(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some("a")), Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::invalid_src_denom_too_long(DEFAULT_POA_ADMIN, create_msg_update_config(Field::SourceDenom, Some("a".repeat(256))), Expect::ErrContains(INVALID_DENOM_FORMAT))]
// --- invalid tgt denom
#[case::invalid_tgt_denom_empty(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some("")), Expect::ErrContains(EMPTY_DENOM))]
#[case::invalid_tgt_denom_same_as_src(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some(DEFAULT_SOURCE_DENOM)), Expect::ErrContains(SAME_DENOM))]
#[case::invalid_tgt_denom_unicode(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some("😀")), Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::invalid_tgt_denom_invalid_ibc(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some("ibc/invalid_denom")), Expect::ErrContains(INVALID_IBC_DENOM_FORMAT))]
#[case::invalid_tgt_denom_invalid_factory(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some(INVALID_FACTORY_DENOM_NOT_ENOUGH_SEP)), Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::invalid_tgt_denom_factory_too_many_separators(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some(INVALID_FACTORY_DENOM_TOO_MANY_SEP)), Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::invalid_tgt_denom_factory_no_creator(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some(INVALID_FACTORY_DENOM_NO_CREATOR)), Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::invalid_tgt_denom_factory_empty_subdenom(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some(INVALID_FACTORY_DENOM_NO_SUBDENOM)), Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::invalid_tgt_denom_invalid_format(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some("invalid_format")), Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::invalid_tgt_denom_too_long(DEFAULT_POA_ADMIN, create_msg_update_config(Field::TargetDenom, Some("a".repeat(256))), Expect::ErrContains(INVALID_DENOM_FORMAT))]
// --- invalid paused
#[case::invalid_paused_string(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Paused, Some("a")), Expect::ErrContains(INVALID_TYPE_STRING))]
#[case::invalid_paused_unicode(DEFAULT_POA_ADMIN, create_msg_update_config(Field::Paused, Some("😀")), Expect::ErrContains(INVALID_TYPE_STRING))]
fn update_config(
    setup_with_funds: (AppAccepting, u64),
    #[case] exec_sender: &str,
    #[case] exec_msg: serde_json::Value,
    #[case] expect: Expect<'_>,
) {
    prepare_and_execute(
        setup_with_funds,
        default_sender(),
        &default_instantiate(),
        &[],
        exec_sender,
        &exec_msg,
        &[],
        expect,
    );
}
