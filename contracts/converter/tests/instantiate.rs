use crate::common::*;
use rstest::*;

mod common;

#[rustfmt::skip]
#[rstest]
// --- admin: OK ---
#[case::admin_default(Field::Admin, DEFAULT_POA_ADMIN, Expect::Ok)]
#[case::admin_valid(Field::Admin, VALID_MANIFEST_ADDRESS, Expect::Ok)]
// --- admin: invalid ---
#[case::admin_empty(Field::Admin, "", Expect::ErrContains(PARSE_FAILED))]
#[case::admin_invalid(Field::Admin, "invalid_address", Expect::ErrContains(PARSE_FAILED))]
#[case::admin_bad_checksum(Field::Admin, INVALID_MANIFEST_ADDRESS, Expect::ErrContains(INVALID_CHECKSUM))]
#[case::admin_unicode(Field::Admin, "😀", Expect::ErrContains(PARSE_FAILED))]
#[case::admin_wrong_prefix(Field::Admin, VALID_OSMOSIS_ADDRESS, Expect::ErrContains(WRONG_BECH32_PREFIX))]
#[case::admin_null(Field::Admin, serde_json::Value::Null, Expect::ErrContains(INVALID_TYPE_NULL))]
#[case::admin_number(Field::Admin, 1, Expect::ErrContains(INVALID_TYPE_INTEGER))]
// --- poa_admin: OK ---
#[case::poa_admin_default(Field::PoaAdmin, DEFAULT_POA_ADMIN, Expect::Ok)]
#[case::poa_admin_valid(Field::PoaAdmin, VALID_MANIFEST_ADDRESS, Expect::Ok)]
// --- poa_admin: invalid ---
#[case::poa_admin_empty(Field::PoaAdmin, "", Expect::ErrContains(PARSE_FAILED))]
#[case::poa_admin_invalid(Field::PoaAdmin, "invalid", Expect::ErrContains(PARSE_FAILED))]
#[case::poa_admin_checksum(Field::PoaAdmin, INVALID_MANIFEST_ADDRESS, Expect::ErrContains(INVALID_CHECKSUM))]
#[case::poa_admin_unicode(Field::PoaAdmin, "😀", Expect::ErrContains(PARSE_FAILED))]
#[case::poa_admin_wrong_prefix(Field::PoaAdmin, VALID_OSMOSIS_ADDRESS, Expect::ErrContains(WRONG_BECH32_PREFIX))]
#[case::poa_admin_null(Field::PoaAdmin, serde_json::Value::Null, Expect::ErrContains(INVALID_TYPE_NULL))]
#[case::poa_admin_number(Field::PoaAdmin, 1, Expect::ErrContains(INVALID_TYPE_INTEGER))]
// --- rate: OK ---
#[case::rate_one(Field::Rate, "1", Expect::Ok)]
#[case::rate_fractional(Field::Rate, "0.001", Expect::Ok)]
#[case::rate_minimum(Field::Rate, VALID_RATE_MIN, Expect::Ok)]
// --- rate: invalid ---
#[case::rate_zero(Field::Rate, "0", Expect::ErrContains(RATE_IS_ZERO))]
#[case::rate_negative(Field::Rate, "-0.5", Expect::ErrContains(RATE_PARSE_FAILED))]
#[case::rate_invalid(Field::Rate, "abc", Expect::ErrContains(RATE_PARSE_FAILED))]
#[case::rate_unicode(Field::Rate, "😀", Expect::ErrContains(RATE_PARSE_FAILED))]
#[case::rate_too_small(Field::Rate, INVALID_RATE_MIN, Expect::ErrContains(RATE_PARSE_FAILED))]
#[case::rate_null(Field::Rate, serde_json::Value::Null, Expect::ErrContains(INVALID_TYPE_NULL))]
#[case::rate_number(Field::Rate, 1, Expect::ErrContains(INVALID_TYPE_INTEGER))]
// --- src_denom: OK ---
#[case::src_denom_default(Field::SourceDenom, DEFAULT_SOURCE_DENOM, Expect::Ok)]
#[case::src_denom_valid(Field::SourceDenom, "umfx", Expect::Ok)]
#[case::src_denom_factory(Field::SourceDenom, VALID_FACTORY_DENOM, Expect::Ok)]
#[case::src_denom_ibc(Field::SourceDenom, VALID_IBC_DENOM, Expect::Ok)]
// --- src_denom: invalid
#[case::src_denom_empty(Field::SourceDenom, "", Expect::ErrContains(EMPTY_DENOM))]
#[case::src_denom_same(Field::SourceDenom, DEFAULT_TARGET_DENOM, Expect::ErrContains(SAME_DENOM))]
#[case::src_denom_unicode(Field::SourceDenom, "😀", Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::src_denom_invalid_ibc(Field::SourceDenom, "ibc/a", Expect::ErrContains(INVALID_IBC_DENOM_FORMAT))]
#[case::src_denom_invalid_factory(Field::SourceDenom, INVALID_FACTORY_DENOM_NOT_ENOUGH_SEP, Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::src_denom_factory_too_many_sep(Field::SourceDenom, INVALID_FACTORY_DENOM_TOO_MANY_SEP, Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::src_denom_factory_no_creator(Field::SourceDenom, INVALID_FACTORY_DENOM_NO_CREATOR, Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::src_denom_factory_empty_subdenom( Field::SourceDenom, INVALID_FACTORY_DENOM_NO_SUBDENOM, Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::src_denom_invalid_format(Field::SourceDenom, "a", Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::src_denom_too_long(Field::SourceDenom, "a".repeat(256), Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::src_denom_null(Field::SourceDenom, serde_json::Value::Null, Expect::ErrContains(INVALID_TYPE_NULL))]
#[case::src_denom_number(Field::SourceDenom, 1, Expect::ErrContains(INVALID_TYPE_INTEGER))]
// --- tgt_denom: OK ---
#[case::tgt_denom_default(Field::TargetDenom, DEFAULT_TARGET_DENOM, Expect::Ok)]
#[case::tgt_denom_valid(Field::TargetDenom, "upwr", Expect::Ok)]
#[case::tgt_denom_factory(Field::TargetDenom, VALID_FACTORY_DENOM, Expect::Ok)]
#[case::tgt_denom_ibc(Field::TargetDenom, VALID_IBC_DENOM, Expect::Ok)]
// --- tgt_denom: invalid
#[case::tgt_denom_empty(Field::TargetDenom, "", Expect::ErrContains(EMPTY_DENOM))]
#[case::tgt_denom_same_as_src(Field::TargetDenom, DEFAULT_SOURCE_DENOM, Expect::ErrContains(SAME_DENOM))]
#[case::tgt_denom_unicode(Field::TargetDenom, "😀", Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::tgt_denom_invalid_ibc(Field::TargetDenom, "ibc/invalid_denom", Expect::ErrContains(INVALID_IBC_DENOM_FORMAT))]
#[case::tgt_denom_invalid_factory(Field::TargetDenom, INVALID_FACTORY_DENOM_NOT_ENOUGH_SEP, Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::tgt_denom_factory_too_many_separators(Field::TargetDenom, INVALID_FACTORY_DENOM_TOO_MANY_SEP, Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::tgt_denom_factory_no_creator(Field::TargetDenom, INVALID_FACTORY_DENOM_NO_CREATOR, Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::tgt_denom_factory_empty_subdenom(Field::TargetDenom, INVALID_FACTORY_DENOM_NO_SUBDENOM, Expect::ErrContains(INVALID_FACTORY_DENOM_FORMAT))]
#[case::tgt_denom_invalid_format(Field::TargetDenom, "invalid_format", Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::tgt_denom_too_long(Field::TargetDenom, "a".repeat(256), Expect::ErrContains(INVALID_DENOM_FORMAT))]
#[case::tgt_denom_null(Field::TargetDenom, serde_json::Value::Null, Expect::ErrContains(INVALID_TYPE_NULL))]
#[case::tgt_denom_number(Field::TargetDenom, 1, Expect::ErrContains(INVALID_TYPE_INTEGER))]
// --- paused: OK ---
#[case::paused_true(Field::Paused, true, Expect::Ok)]
#[case::paused_false(Field::Paused, false, Expect::Ok)]
// --- paused: invalid ---
#[case::paused_invalid(Field::Paused, "a", Expect::ErrContains(INVALID_TYPE_STRING))]
#[case::paused_unicode(Field::Paused, "😀", Expect::ErrContains(INVALID_TYPE_STRING))]
#[case::paused_null(Field::Paused, serde_json::Value::Null, Expect::ErrContains(INVALID_TYPE_NULL))]
#[case::paused_number(Field::Paused, 1, Expect::ErrContains(INVALID_TYPE_INTEGER))]
fn instantiate_field_variations(
    setup: (AppAccepting, u64),
    #[case] field: Field,
    #[case] val: impl serde::Serialize,
    #[case] expect: Expect<'static>,
) {
    let (app, code_id) = setup;
    run_instantiate(app, code_id, default_sender(), &modify_instantiate(field, val), no_funds(), expect);
}

#[rstest]
fn instantiate_with_funds(setup_with_funds: (AppAccepting, u64)) {
    let (app, code_id) = setup_with_funds;
    run_instantiate(
        app,
        code_id,
        default_sender(),
        &default_instantiate(),
        &default_initial_funds(),
        Expect::ErrContains(NON_PAYABLE),
    );
}
