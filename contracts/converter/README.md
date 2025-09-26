# Converter Contract

A CosmWasm smart contract for converting between MFX and different token denominations on the Manifest Network. This contract enables secure token conversion using configurable exchange rates and authorization mechanisms.

## Overview

The Converter contract facilitates the conversion of MFX tokens to target tokens at a predetermined exchange rate. 

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
