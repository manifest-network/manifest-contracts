#!/usr/bin/env bats

load '../test_helper/bats-support/load'
load '../test_helper/bats-assert/load'
load '../test_helper/common.sh'

setup() {
  export CHAIN_ID="test-1"
  export RPC="http://localhost:26657"
  export CONTAINER_NAME="e2e-manifest-ledger-1"
  export WASM_FILE="artifacts/converter.wasm"
  export USER_ADDR="manifest1hj5fveer5cjtn4wd6wstzugjfdxzl0xp8ws9ct"
  export USER2_ADDR="manifest1efd63aw40lxf3n4mhf7dzhjkr453axurm6rp3z"
  export POA_ADMIN_ADDR=$USER_ADDR
  export RATE="2"
  export SOURCE_DENOM="umfx"
  export TARGET_DENOM="factory/manifest1afk9zr2hn2jsac63h4hm60vl9z3e5u69gndzf7c99cqge3vzwjzsfmy9qj/upwr"
  export GAS_PRICES="0.0011umfx"

  run make build-opt
  assert_success
  assert [ -f "$WASM_FILE" ]

  run make e2e-chain-up
  assert_success
  sleep 5
}

teardown() {
  [ "$BATS_TEST_COMPLETED" = "1" ] && make e2e-chain-down
}

@test "converter end-to-end" {
  store_contract

  INIT_MSG=$(jq -n --arg admin "$USER_ADDR" --arg poa "$POA_ADMIN_ADDR" \
    --arg rate "$RATE" --arg src "$SOURCE_DENOM" --arg tgt "$TARGET_DENOM" \
    '{"admin":$admin,"poa_admin":$poa,"rate":$rate,"source_denom":$src,"target_denom":$tgt,"paused":false}')
  instantiate_contract "$INIT_MSG"

  grant_authz "/osmosis.tokenfactory.v1beta1.MsgMint"
  grant_authz "/liftedinit.manifest.v1.MsgBurnHeldBalance"

  SOURCE_SUPPLY_BEFORE=$(query_json "Querying MFX total supply before conversion" bank total-supply-of "$SOURCE_DENOM" | jq -r '.amount.amount')
  TARGET_SUPPLY_BEFORE=$(query_json "Querying PWR total supply before conversion" bank total-supply-of "$TARGET_DENOM" | jq -r '.amount.amount')

  execute_contract '{"convert":{}}' "$USER2_ADDR" "1000000$SOURCE_DENOM"

  PWR_BALANCE=$(query_json "Querying user2 PWR balance" bank balances "$USER2_ADDR" | jq -r '.balances[] | select(.denom=="'"$TARGET_DENOM"'") | .amount')
  assert [ "$PWR_BALANCE" == "2000000" ]

  execute_contract '{"update_admin":{"admin":"'"$USER2_ADDR"'"}}' "$USER_ADDR"
  execute_contract '{"update_config":{"config":{"rate":"3"}}}' "$USER2_ADDR"

  execute_contract '{"convert":{}}' "$USER2_ADDR" "1000000$SOURCE_DENOM"
  PWR_BALANCE=$(query_json "Querying updated PWR balance" bank balances "$USER2_ADDR" | jq -r '.balances[] | select(.denom=="'"$TARGET_DENOM"'") | .amount')
  assert [ "$PWR_BALANCE" == "5000000" ]

  execute_contract '{"update_config":{"config":{"paused":true}}}' "$USER2_ADDR"
  run docker exec "$CONTAINER_NAME" manifestd tx wasm execute "$CONTRACT_ADDR" '{"convert":{}}' \
    --from "$USER2_ADDR" --home "/persistent/.manifest" --keyring-backend "test" \
    --amount "1000000$SOURCE_DENOM" --gas auto --gas-prices "$GAS_PRICES" \
    --chain-id "$CHAIN_ID" --node "$RPC" --yes --output json
  assert_failure

  execute_contract '{"update_config":{"config":{"paused":false}}}' "$USER2_ADDR"
  execute_contract '{"convert":{}}' "$USER2_ADDR" "1000000$SOURCE_DENOM"
  PWR_BALANCE=$(query_json "Querying PWR balance after unpause" bank balances "$USER2_ADDR" | jq -r '.balances[] | select(.denom=="'"$TARGET_DENOM"'") | .amount')
  assert [ "$PWR_BALANCE" == "8000000" ]

  run_tx "Migrating contract" wasm migrate "$CONTRACT_ADDR" "$CODE_ID" '{"migrate":{}}' --from "$USER_ADDR"
  NOTE=$(docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" --node "$RPC" --output json | \
    jq -r '.events[] | select(.type=="wasm") | .attributes[] | select(.key=="note") | .value')
  assert [ "$NOTE" == "already at latest version" ]
}
