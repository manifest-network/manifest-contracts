#![allow(dead_code)] // Allow dead code since not all helpers are used in every test file

use const_format::str_splice_out;
use converter::{execute, instantiate, migrate, query};
use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::{coin, Addr, Coin, Empty};
use cw_multi_test::{
    App, AppBuilder, BankKeeper, ContractWrapper, DistributionKeeper, Executor, FailingModule,
    GovFailingModule, IbcFailingModule, StakeKeeper, StargateAccepting, WasmKeeper,
};
use rstest::*;
use serde::Serialize;
use serde_json::{json, Value};
use strum_macros::{AsRefStr, IntoStaticStr};

// Default values for instantiation
const BECH32_PREFIX: &str = "manifest";
pub const DEFAULT_POA_ADMIN: &str =
    "manifest1afk9zr2hn2jsac63h4hm60vl9z3e5u69gndzf7c99cqge3vzwjzsfmy9qj";
pub const DEFAULT_SENDER: &str =
    "manifest1pgm8hyk0pvphmlvfjc8wsvk4daluz5tgrw6pu5mfpemk74uxnx9qdtpy2n";
const DEFAULT_RATE: &str = "0.5";
pub const DEFAULT_SOURCE_DENOM: &str = "umfx";
pub const DEFAULT_TARGET_DENOM: &str = "upwr";
pub const DUMMY_DENOM: &str = "udummy";
const DEFAULT_PAUSED: bool = false;

// Valid test constants
pub const VALID_MANIFEST_ADDRESS: &str = "manifest1hj5fveer5cjtn4wd6wstzugjfdxzl0xp8ws9ct";
pub const VALID_OSMOSIS_ADDRESS: &str = "osmo14nalsczp8rnu5htrtvshqxa9x40x30m96zdrvg";
pub const VALID_FACTORY_DENOM: &str =
    "factory/manifest1hj5fveer5cjtn4wd6wstzugjfdxzl0xp8ws9ct/utgt";
pub const VALID_IBC_DENOM: &str =
    "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2";
pub const VALID_RATE_MIN: &str = "0.000000000000000001";

// Invalid test constants
pub const INVALID_MANIFEST_ADDRESS: &str = str_splice_out!(
    VALID_MANIFEST_ADDRESS,
    VALID_MANIFEST_ADDRESS.len() - 2..VALID_MANIFEST_ADDRESS.len() - 1,
    ""
);
pub const INVALID_RATE_MIN: &str = "0.0000000000000000001";
pub const INVALID_FACTORY_DENOM_TOO_MANY_SEP: &str =
    "factory/manifest1hj5fveer5cjtn4wd6wstzugjfdxzl0xp8ws9ct/utgt/a";
pub const INVALID_FACTORY_DENOM_NOT_ENOUGH_SEP: &str = "factory/a";
pub const INVALID_FACTORY_DENOM_NO_CREATOR: &str = "factory/";
pub const INVALID_FACTORY_DENOM_NO_SUBDENOM: &str =
    "factory/manifest1hj5fveer5cjtn4wd6wstzugjfdxzl0xp8ws9ct/";

// Common error substrings
pub const PARSE_FAILED: &str = "parse failed";
pub const INVALID_CHECKSUM: &str = "invalid checksum";
pub const WRONG_BECH32_PREFIX: &str = "Wrong bech32 prefix";
pub const INVALID_TYPE_NULL: &str = "invalid type: null";
pub const INVALID_TYPE_INTEGER: &str = "invalid type: integer";
pub const INVALID_TYPE_STRING: &str = "invalid type: string";
pub const RATE_IS_ZERO: &str = "rate is zero";
pub const RATE_PARSE_FAILED: &str = "failed to parse rate";
pub const RESULT_IS_ZERO: &str = "resulting amount is zero";
pub const SAME_DENOM: &str = "source and target denom cannot be the same";
pub const EMPTY_DENOM: &str = "denom is empty";
pub const INVALID_DENOM_FORMAT: &str = "invalid denom format";
pub const INVALID_IBC_DENOM_FORMAT: &str = "invalid ibc denom format";
pub const INVALID_FACTORY_DENOM_FORMAT: &str = "invalid factory denom format";
pub const NON_PAYABLE: &str = "non-payable function called with funds";
pub const INVALID_FUNDS: &str = "invalid funds sent";
pub const INVALID_SOURCE_DENOM: &str = "invalid source denom";
pub const CONTRACT_PAUSED: &str = "contract is paused";
pub const ONLY_ADMIN: &str = "only admin can perform this action";
pub const CANNOT_RENOUNCE: &str = "cannot renounce admin role";

// The following errors are not defined in the contract, but are common CosmWasm errors

// Error thrown when trying to execute the contract with an invalid address and some funds
// CosmWasm tries to send the funds from the invalid address to the contract, which fails
pub const CANNOT_SUB: &str = "Cannot Sub with given operands";
pub const ZERO_FUNDS: &str = "Cannot transfer empty coins amount";
pub const UNKNOWN_VARIANT: &str = "unknown variant";
pub const EXPECTED_VALUE: &str = "expected value";

// One can't use the `App` type directly when `.with_stargate(StargateAccepting)` is used
// See https://github.com/CosmWasm/cw-multi-test/issues/285
pub type AppAccepting<ExecC = Empty, QueryC = Empty> = App<
    BankKeeper,
    MockApi,
    MockStorage,
    FailingModule<ExecC, QueryC, Empty>,
    WasmKeeper<ExecC, QueryC>,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    StargateAccepting,
>;

#[derive(Copy, Clone)]
pub enum Expect<'a> {
    Ok,
    ErrContains(&'a str),
}

#[fixture]
pub fn default_sender() -> &'static str {
    DEFAULT_SENDER
}

#[fixture]
pub fn no_funds() -> &'static [Coin] {
    &[]
}

// Provide some initial funds to the default sender
#[fixture]
pub fn default_initial_funds() -> Vec<Coin> {
    vec![
        coin(1_000_000, DEFAULT_SOURCE_DENOM),
        coin(1_000_000, DEFAULT_TARGET_DENOM),
        coin(1_000_000, DUMMY_DENOM),
    ]
}

#[fixture]
pub fn default_convert_amount() -> Coin {
    coin(1_000, DEFAULT_SOURCE_DENOM)
}

fn base_config_map() -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert(
        "poa_admin".to_string(),
        Value::String(DEFAULT_POA_ADMIN.to_string()),
    );
    map.insert("rate".to_string(), Value::String(DEFAULT_RATE.to_string()));
    map.insert(
        "source_denom".to_string(),
        Value::String(DEFAULT_SOURCE_DENOM.to_string()),
    );
    map.insert(
        "target_denom".to_string(),
        Value::String(DEFAULT_TARGET_DENOM.to_string()),
    );
    map.insert("paused".to_string(), Value::Bool(DEFAULT_PAUSED));
    map
}

#[fixture]
pub fn default_admin() -> &'static str {
    DEFAULT_POA_ADMIN
}

#[fixture]
pub fn default_config() -> Value {
    Value::Object(base_config_map())
}

#[fixture]
pub fn default_instantiate() -> Value {
    let mut map = base_config_map();
    map.insert(
        "admin".to_string(),
        Value::String(default_admin().to_string()),
    );
    Value::Object(map)
}

#[fixture]
pub fn default_rate() -> &'static str {
    DEFAULT_RATE
}

#[fixture]
pub fn default_convert() -> Value {
    json!({"convert": {}})
}

#[fixture]
pub fn setup() -> (AppAccepting, u64) {
    let mut app = AppBuilder::default()
        .with_api(MockApi::default().with_prefix(BECH32_PREFIX))
        .with_stargate(StargateAccepting)
        .build(|_, _, _| {});
    let code_id = app.store_code(Box::new(
        ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate),
    ));
    (app, code_id)
}

#[fixture]
pub fn setup_with_funds() -> (AppAccepting, u64) {
    let mut app = AppBuilder::default()
        .with_api(MockApi::default().with_prefix(BECH32_PREFIX))
        .with_stargate(StargateAccepting)
        .build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(default_sender()),
                    default_initial_funds(),
                )
                .expect("failed to init balance");
        });
    let code_id = app.store_code(Box::new(
        ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate),
    ));
    (app, code_id)
}

pub fn run_instantiate(
    mut app: AppAccepting,
    code_id: u64,
    sender: &str,
    msg: &impl serde::Serialize,
    funds: &[Coin],
    expect: Expect<'_>,
) {
    let res = app.instantiate_contract(
        code_id,
        Addr::unchecked(sender),
        msg,
        funds,
        "converter",
        None,
    );
    match expect {
        Expect::Ok => {
            let addr = res.expect("expected Ok");
            assert!(!addr.as_str().is_empty());
        }
        Expect::ErrContains(s) => {
            let err = res.err().unwrap();
            let text = format!("{err:#}");
            assert!(
                text.contains(s),
                "error didn't contain expected substring.\nGot:\n{:#}\nExpected to contain:\n{:#}",
                text,
                s
            );
        }
    }
}

fn run_execute(
    app: &mut AppAccepting,
    sender: &str,
    contract_addr: &str,
    msg: &(impl serde::Serialize + std::fmt::Debug),
    funds: &[Coin],
    expect: Expect<'_>,
) {
    let res = app.execute_contract(
        Addr::unchecked(sender),
        Addr::unchecked(contract_addr),
        msg,
        funds,
    );
    match expect {
        Expect::Ok => {
            let _res = res.expect("expected Ok");
        }
        Expect::ErrContains(s) => {
            let err = res.err().unwrap();
            let text = format!("{err:#}");
            assert!(
                text.contains(s),
                "error didn't contain expected substring.\nGot:\n{:#}\nExpected to contain:\n{:#}",
                text,
                s
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_and_execute(
    setup_with_funds: (AppAccepting, u64),
    instantiate_sender: &str,
    instantiate_msg: &impl serde::Serialize,
    instantiate_funds: &[Coin],
    exec_sender: &str,
    exec_msg: &(impl serde::Serialize + std::fmt::Debug),
    funds: &[Coin],
    expect: Expect<'_>,
) -> (AppAccepting, Addr, u64) {
    let (mut app, code_id) = setup_with_funds;
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(instantiate_sender),
            instantiate_msg,
            instantiate_funds,
            "converter",
            None,
        )
        .expect("failed to instantiate");
    run_execute(
        &mut app,
        exec_sender,
        contract_addr.as_ref(),
        exec_msg,
        funds,
        expect,
    );
    (app, contract_addr, code_id)
}

#[derive(Clone, Copy, Debug, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Field {
    Admin,
    PoaAdmin,
    Rate,
    SourceDenom,
    TargetDenom,
    Paused,
}

pub fn modify_config(field: Field, value: impl serde::Serialize) -> Value {
    let mut default_config = default_config();
    default_config[field.as_ref()] = json!(value);
    default_config
}

pub fn modify_instantiate(field: Field, value: impl serde::Serialize) -> Value {
    let mut default_instantiate = default_instantiate();
    default_instantiate[field.as_ref()] = json!(value);
    default_instantiate
}

pub fn create_msg_update_config_from_config(config: &impl Serialize) -> Value {
    json!({"update_config": {"config": config}})
}

pub fn create_msg_update_config(field: Field, value: impl serde::Serialize) -> Value {
    json!({"update_config": {"config": modify_config(field, value)}})
}

pub fn create_msg_update_config_noop() -> Value {
    json!({"update_config": {"config": {}}})
}

pub fn create_msg_update_admin(new_admin: Option<&str>) -> Value {
    json!({"update_admin": {"admin": new_admin}})
}
