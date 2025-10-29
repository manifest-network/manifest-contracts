#!/usr/bin/env bash

run_tx() {
  local desc=$1
  shift
  echo "# --> $desc" >&3
  run docker exec "$CONTAINER_NAME" manifestd tx "$@" \
    --home "/persistent/.manifest" \
    --keyring-backend "test" \
    --gas auto \
    --gas-adjustment 1.8 \
    --gas-prices "$GAS_PRICES" \
    --chain-id "$CHAIN_ID" \
    --node "$RPC" \
    --yes \
    --output json

  assert_success
  local json_output=$(echo "$output" | tail -n +3)
  TX_HASH=$(echo "$json_output" | jq -r '.txhash')
  ERR_CODE=$(echo "$json_output" | jq -r '.code')
  assert [ -n "$TX_HASH" ]
  assert [ "$ERR_CODE" -eq 0 ]
  echo "# --> Transaction submitted with hash: $TX_HASH" >&3
  sleep 3

  run docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" \
    --node "$RPC" \
    --output json
  assert_success
  ERR_CODE=$(echo "$output" | tail -n +2 | jq -r '.code')
  if [ "$ERR_CODE" != "0" ]; then
    echo "# ⚠️  Tx returned error code $ERR_CODE" >&3
    echo "$output" >&3
  fi
}

query_json() {
  local desc=$1
  shift
  echo "# --> $desc" >&3
  run docker exec "$CONTAINER_NAME" manifestd query "$@" \
    --node "$RPC" --output json
  assert_success
  echo "$output" | tail -n +2
}

store_contract() {
  run_tx "Storing contract on-chain" wasm store "/workspace/$WASM_FILE" --from "$USER_ADDR"
  CODE_ID=$(docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" --node "$RPC" --output json | \
    jq -r '.events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
  assert [ "$CODE_ID" -gt 0 ]
  echo "# --> Contract uploaded with code ID: $CODE_ID" >&3
}

instantiate_contract() {
  local init_msg=$1
  run_tx "Instantiating contract" wasm instantiate "$CODE_ID" "$init_msg" \
    --from "$USER_ADDR" --label "converter-test" --admin "$USER_ADDR"
  CONTRACT_ADDR=$(docker exec "$CONTAINER_NAME" manifestd query tx "$TX_HASH" --node "$RPC" --output json | \
    jq -r '.events[] | select(.type=="wasm") | .attributes[] | select(.key=="_contract_address") | .value')
  assert [ -n "$CONTRACT_ADDR" ]
  echo "# --> Contract instantiated at: $CONTRACT_ADDR" >&3
}

grant_authz() {
  local msg_type=$1
  run_tx "Granting authz for $msg_type" authz grant "$CONTRACT_ADDR" generic \
    --msg-type "$msg_type" --from "$USER_ADDR"
}

execute_contract() {
  local msg=$1
  local from=$2
  local amount=${3:-}  # optional third argument

  echo "# --> Executing contract message" >&3

  if [ -n "$amount" ]; then
    run_tx "Executing contract (with funds)" wasm execute "$CONTRACT_ADDR" "$msg" \
      --from "$from" \
      --amount "$amount"
  else
    run_tx "Executing contract" wasm execute "$CONTRACT_ADDR" "$msg" \
      --from "$from"
  fi
}
