# Converter Contract

A CosmWasm smart contract for converting between MFX and different token denominations on the Manifest Network. This contract enables secure token conversion using configurable exchange rates and authorization mechanisms.

## Overview

The Converter contract facilitates the conversion of MFX tokens to target tokens at a predetermined exchange rate.

## Deployed Contracts

| Network | Contract Address |
|---------|------------------|
| Mainnet | `manifest1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqdnm0gk` |
| Testnet | `manifest1c4p5p0eajlymxak8z5ugmksfpvp5zmm3scrwdrtd5a6mwxnhsa9qnsfftk` |

## Features

- **Token Conversion**: Convert between any two token denominations
- **Configurable Exchange Rate**: Set custom conversion rates using decimal precision
- **Pause Functionality**: Emergency pause mechanism to halt conversions
- **Admin Controls**: Administrative functions for configuration management

## Contract Architecture

### Key Components

- **Rate System**: Handles exchange rate calculations and validation
- **Denomination Management**: Validates and manages source/target token types
- **Admin Controls**: Manages contract administration and configuration updates
- **Conversion Logic**: Orchestrates the burn-and-mint token conversion process

### State Management

The contract maintains:
- **Config**: Core configuration including rates, denominations, and pause state
- **Admin**: Contract administrator with privileged access

## Messages

### Instantiate

Initialize the contract with configuration parameters:

```json
{
  "admin": "manifest1...",
  "poa_admin": "manifest1...",
  "rate": "1.5",
  "source_denom": "utoken1",
  "target_denom": "utoken2",
  "paused": false
}
```

### Execute Messages

#### Convert
Convert source tokens to target tokens:
```json
{
  "convert": {}
}
```
*Note: Send the source tokens as funds with this message*

#### Update Config
Update contract configuration (admin only):
```json
{
  "update_config": {
    "config": {
      "poa_admin": "manifest1...",
      "rate": "2.0",
      "source_denom": "unewtoken",
      "target_denom": "uanothertoken",
      "paused": true
    }
  }
}
```

#### Update Admin
Transfer admin privileges (admin only):
```json
{
  "update_admin": {
    "admin": "manifest1..."
  }
}
```

### Query Messages

#### Config
Get current contract configuration:
```json
{
  "config": {}
}
```

#### Admin
Get current admin address:
```json
{
  "admin": {}
}
```

## Interacting with the Contract

You can interact with the deployed converter contract using either REST (via `curl`) or RPC (via `manifestd`).

### Network Endpoints

| Network | REST API | RPC |
|---------|----------|-----|
| Mainnet | `https://nodes.manifest.network/manifest/api` | `https://nodes.manifest.network/manifest/rpc` |
| Testnet | `https://nodes.liftedinit.tech/manifest/testnet/api` | `https://nodes.liftedinit.tech/manifest/testnet/rpc` |

For more information on network configurations, see:
- [Mainnet Configuration](https://docs.manifest.network/network-configurations/mainnet)
- [Testnet Configuration](https://docs.manifest.network/network-configurations/testnet)

### Prerequisites

#### Installing manifestd

To use RPC commands, you need `manifestd` installed. It is a CosmosSDK-based chain binary with CosmWasm enabled.

```bash
# Clone and build from source
git clone https://github.com/manifest-network/manifest-ledger
cd manifest-ledger
make install
```

#### Base64 Encoding for REST Queries

REST API queries require the JSON query message to be base64-encoded. You can encode messages using:

```bash
# Linux/macOS
echo -n '{"config":{}}' | base64
# Output: eyJjb25maWciOnt9fQ==

echo -n '{"admin":{}}' | base64
# Output: eyJhZG1pbiI6e319
```

### Query Examples

#### Get Contract Configuration

**Mainnet (REST)**
```bash
curl "https://nodes.manifest.network/manifest/api/cosmwasm/wasm/v1/contract/manifest1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqdnm0gk/smart/eyJjb25maWciOnt9fQ=="
```

**Mainnet (RPC)**
```bash
manifestd q wasm contract-state smart manifest1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqdnm0gk '{"config":{}}' --node https://nodes.manifest.network/manifest/rpc
```

**Testnet (REST)**
```bash
curl "https://nodes.liftedinit.tech/manifest/testnet/api/cosmwasm/wasm/v1/contract/manifest1c4p5p0eajlymxak8z5ugmksfpvp5zmm3scrwdrtd5a6mwxnhsa9qnsfftk/smart/eyJjb25maWciOnt9fQ=="
```

**Testnet (RPC)**
```bash
manifestd q wasm contract-state smart manifest1c4p5p0eajlymxak8z5ugmksfpvp5zmm3scrwdrtd5a6mwxnhsa9qnsfftk '{"config":{}}' --node https://nodes.liftedinit.tech/manifest/testnet/rpc
```

#### Get Contract Admin

**Mainnet (REST)**
```bash
curl "https://nodes.manifest.network/manifest/api/cosmwasm/wasm/v1/contract/manifest1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqdnm0gk/smart/eyJhZG1pbiI6e319"
```

**Mainnet (RPC)**
```bash
manifestd q wasm contract-state smart manifest1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqdnm0gk '{"admin":{}}' --node https://nodes.manifest.network/manifest/rpc
```

**Testnet (REST)**
```bash
curl "https://nodes.liftedinit.tech/manifest/testnet/api/cosmwasm/wasm/v1/contract/manifest1c4p5p0eajlymxak8z5ugmksfpvp5zmm3scrwdrtd5a6mwxnhsa9qnsfftk/smart/eyJhZG1pbiI6e319"
```

**Testnet (RPC)**
```bash
manifestd q wasm contract-state smart manifest1c4p5p0eajlymxak8z5ugmksfpvp5zmm3scrwdrtd5a6mwxnhsa9qnsfftk '{"admin":{}}' --node https://nodes.liftedinit.tech/manifest/testnet/rpc
```

### Execute Examples

Execute transactions require a wallet with funds to pay for gas fees. The examples below use `manifestd` with a configured wallet.

#### Convert Tokens

Convert source tokens to target tokens. You must send the source tokens as funds with the transaction.

**Mainnet**
```bash
manifestd tx wasm execute manifest1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqdnm0gk '{"convert":{}}' \
  --amount 1000000umfx \
  --from <your-wallet> \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 1.0umfx \
  --node https://nodes.manifest.network/manifest/rpc \
  --chain-id manifest-ledger-mainnet
```

**Testnet**
```bash
manifestd tx wasm execute manifest1c4p5p0eajlymxak8z5ugmksfpvp5zmm3scrwdrtd5a6mwxnhsa9qnsfftk '{"convert":{}}' \
  --amount 1000000umfx \
  --from <your-wallet> \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 1.0umfx \
  --node https://nodes.liftedinit.tech/manifest/testnet/rpc \
  --chain-id manifest-ledger-testnet
```

> **Note**: Replace `1000000umfx` with the actual source token denomination and amount configured in the contract. Query the contract configuration first to see the current `source_denom`.

#### Update Config (Admin Only)

Update the contract configuration. Only the contract admin can execute this.

**Mainnet**
```bash
manifestd tx wasm execute manifest1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqdnm0gk \
  '{"update_config":{"config":{"rate":"2.0","paused":false}}}' \
  --from <admin-wallet> \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 1.0umfx \
  --node https://nodes.manifest.network/manifest/rpc \
  --chain-id manifest-ledger-mainnet
```

**Testnet**
```bash
manifestd tx wasm execute manifest1c4p5p0eajlymxak8z5ugmksfpvp5zmm3scrwdrtd5a6mwxnhsa9qnsfftk \
  '{"update_config":{"config":{"rate":"2.0","paused":false}}}' \
  --from <admin-wallet> \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 1.0umfx \
  --node https://nodes.liftedinit.tech/manifest/testnet/rpc \
  --chain-id manifest-ledger-testnet
```

> **Note**: All fields in `update_config` are optional. Only include the fields you want to change.

#### Update Admin (Admin Only)

Transfer admin privileges to a new address. Only the current admin can execute this.

**Mainnet**
```bash
manifestd tx wasm execute manifest1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqdnm0gk \
  '{"update_admin":{"admin":"manifest1newadminaddress..."}}' \
  --from <current-admin-wallet> \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 1.0umfx \
  --node https://nodes.manifest.network/manifest/rpc \
  --chain-id manifest-ledger-mainnet
```

**Testnet**
```bash
manifestd tx wasm execute manifest1c4p5p0eajlymxak8z5ugmksfpvp5zmm3scrwdrtd5a6mwxnhsa9qnsfftk \
  '{"update_admin":{"admin":"manifest1newadminaddress..."}}' \
  --from <current-admin-wallet> \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 1.0umfx \
  --node https://nodes.liftedinit.tech/manifest/testnet/rpc \
  --chain-id manifest-ledger-testnet
```

> **Warning**: Admin transfer is irreversible. The contract does not allow renouncing admin privileges (setting admin to null).

## Development

### Building
```bash
cargo wasm
```

### Testing
```bash
cargo test
```

## Migration

The contract supports migration with version checking to ensure compatibility. Migration logic can be extended as needed for future versions.

## License

Apache-2.0

## Links

- **Repository**: https://github.com/manifest-network/manifest-contracts
- **Homepage**: https://manifest.network
- **manifestd**: https://github.com/manifest-network/manifest-ledger
