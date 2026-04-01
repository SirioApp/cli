# backed-cli

`backed-cli` is a Rust command-line interface for operating the current Backed on-chain backend.
It reads the canonical deployment artifacts from `backend/deployments` and exposes a clean interface
for the three core protocol surfaces:

- `factory`: project creation, approval, metadata, status, and global configuration
- `sale`: raise status, user claim/refund views, approvals, commitments, and sale lifecycle actions
- `allowlist`: target contract administration for executor permissions

## What This CLI Targets

This CLI is aligned with the current contracts inside `backend`, in particular:

- `AgentRaiseFactory`
- `Sale`
- `ContractAllowlist`

It assumes the deployment JSON files in `backend/deployments` are the source of truth for:

- RPC endpoint
- `AgentRaiseFactory` address
- `ContractAllowlist` address
- optional default collateral (`USDM`)

## Project Structure

The CLI is split by responsibility instead of keeping protocol logic in a single file:

```text
src/
  main.rs             # entrypoint + command dispatch
  cli.rs              # clap definitions
  config.rs           # deployment resolution and runtime config
  util.rs             # shared numeric and tx helpers
  types.rs            # common domain types
  chain/
    client.rs         # provider and signer creation
    contracts.rs      # ABI bindings and on-chain read helpers
  commands/
    network.rs        # network summary command
    factory.rs        # factory command handlers
    sale.rs           # sale command handlers
    allowlist.rs      # allowlist command handlers
  output/
    mod.rs            # formatting and terminal output helpers
```

## Requirements

- Rust and Cargo
- access to a valid RPC endpoint for the selected network
- a private key only for write commands

## Build

From the repository root:

```bash
cd backed-cli
cargo build
```

For a release binary:

```bash
cd backed-cli
cargo build --release
```

## Run

From the repository root:

```bash
cargo run -p backed-cli -- --network testnet network
```

Or from inside the CLI directory:

```bash
cargo run -- --network testnet network
```

## Configuration

The CLI automatically resolves configuration from:

- `backend/deployments/megaeth-testnet.json`
- `backend/deployments/megaeth-mainnet.json`

### Supported Overrides

Every runtime value can be overridden through flags or environment variables:

| Purpose | Flag | Environment variable |
|---|---|---|
| network | `--network` | `BACKED_NETWORK` |
| rpc url | `--rpc-url` | `BACKED_RPC_URL` |
| factory address | `--factory` | `BACKED_FACTORY` |
| allowlist address | `--allowlist` | `BACKED_ALLOWLIST` |
| private key | `--private-key` | `BACKED_PRIVATE_KEY` |

### Network Selection

Supported networks:

- `testnet`
- `mainnet`

Example:

```bash
cargo run -- --network testnet network
```

### Write Commands

Write commands require a signer. The current implementation supports:

- `--private-key <HEX_KEY>`
- `BACKED_PRIVATE_KEY=<HEX_KEY>`

Example:

```bash
BACKED_PRIVATE_KEY=0x... cargo run -- --network testnet factory approve 0
```

## Command Reference

## `network`

Shows the resolved runtime configuration:

```bash
cargo run -- --network testnet network
```

Output includes:

- selected network label
- chain id
- RPC URL
- factory address
- allowlist address
- default collateral, when present
- deployment file path

## `factory`

Factory commands operate on `AgentRaiseFactory`.

### Read Commands

Show overall factory status:

```bash
cargo run -- --network testnet factory info
```

Show only the global config:

```bash
cargo run -- --network testnet factory global
```

Inspect a collateral token:

```bash
cargo run -- --network testnet factory collateral 0x9f5A17BD53310D012544966b8e3cF7863fc8F05f
```

Paginate projects:

```bash
cargo run -- --network testnet factory list --from 0 --limit 20
```

Inspect one project:

```bash
cargo run -- --network testnet factory project 0
```

Inspect the live raise snapshot for a project:

```bash
cargo run -- --network testnet factory snapshot 0
```

Inspect one wallet commitment through the factory:

```bash
cargo run -- --network testnet factory commitment 0 0x1111111111111111111111111111111111111111
```

List projects linked to an agent id:

```bash
cargo run -- --network testnet factory agent-projects 0
```

### Write Commands

Create a project:

```bash
cargo run -- --network testnet --private-key 0x... factory create \
  --agent-id 0 \
  --name "Backed Demo" \
  --description "Demo raise for validation" \
  --categories "defi,infra" \
  --token-name BACKED \
  --token-symbol BACKED \
  --duration-minutes 60 \
  --launch-in-minutes 5 \
  --collateral 0x9f5A17BD53310D012544966b8e3cF7863fc8F05f
```

Approve a project:

```bash
cargo run -- --network testnet --private-key 0x... factory approve 0
```

Revoke a project:

```bash
cargo run -- --network testnet --private-key 0x... factory revoke 0
```

Update project metadata:

```bash
cargo run -- --network testnet --private-key 0x... factory update-metadata \
  --project-id 0 \
  --description "Updated project description" \
  --categories "defi,ai"
```

Update operational status:

```bash
cargo run -- --network testnet --private-key 0x... factory set-status \
  --project-id 0 \
  --status operating \
  --status-note "Treasury live"
```

Enable or disable a collateral:

```bash
cargo run -- --network testnet --private-key 0x... factory set-collateral \
  0x9f5A17BD53310D012544966b8e3cF7863fc8F05f true
```

Update global config:

```bash
cargo run -- --network testnet --private-key 0x... factory set-global \
  --min-raise 100 \
  --max-raise 100000 \
  --platform-fee-bps 100 \
  --platform-fee-recipient 0x1111111111111111111111111111111111111111 \
  --min-duration-minutes 10 \
  --max-duration-minutes 1440 \
  --min-launch-delay-minutes 0 \
  --max-launch-delay-minutes 1440
```

### Factory Notes

- `duration-minutes` and `launch-in-minutes` are converted to seconds internally.
- create uses the default collateral from deployments when `--collateral` is omitted.
- factory views assume the deployed ABI matches the current backend contracts.

## `sale`

Sale commands can target a sale in two ways:

- direct address: `--sale <SALE_ADDRESS>`
- indirect project lookup: `--project-id <ID>`

### Read Commands

Show sale status:

```bash
cargo run -- --network testnet sale status --sale 0x2222222222222222222222222222222222222222
```

Show claimable amounts for one user:

```bash
cargo run -- --network testnet sale claimable \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

Show refundable amount for one user:

```bash
cargo run -- --network testnet sale refundable \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

Show committed amount for one user:

```bash
cargo run -- --network testnet sale commitment \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

### Write Commands

Approve collateral for a sale:

```bash
cargo run -- --network testnet --private-key 0x... sale approve-collateral \
  --sale 0x2222222222222222222222222222222222222222 \
  100
```

Commit collateral:

```bash
cargo run -- --network testnet --private-key 0x... sale commit \
  --sale 0x2222222222222222222222222222222222222222 \
  100
```

Finalize a sale:

```bash
cargo run -- --network testnet --private-key 0x... sale finalize \
  --sale 0x2222222222222222222222222222222222222222
```

Claim:

```bash
cargo run -- --network testnet --private-key 0x... sale claim \
  --sale 0x2222222222222222222222222222222222222222
```

Refund:

```bash
cargo run -- --network testnet --private-key 0x... sale refund \
  --sale 0x2222222222222222222222222222222222222222
```

Emergency refund:

```bash
cargo run -- --network testnet --private-key 0x... sale emergency-refund \
  --sale 0x2222222222222222222222222222222222222222
```

### Raw Amount Mode

By default:

- `sale approve-collateral`
- `sale commit`

accept human-readable token amounts using the token decimals fetched on-chain.

To pass raw `uint256` values directly, add `--raw`:

```bash
cargo run -- --network testnet --private-key 0x... sale commit \
  --sale 0x2222222222222222222222222222222222222222 \
  1000000 \
  --raw
```

### Sale Safety Checks

Before `sale commit`, the CLI verifies:

- allowance
- token balance

If either check fails, the transaction is not sent.

## `allowlist`

Allowlist commands operate on `ContractAllowlist`.

### Read Commands

Show admin:

```bash
cargo run -- --network testnet allowlist info
```

Check if a target is allowed:

```bash
cargo run -- --network testnet allowlist is-allowed \
  0x3333333333333333333333333333333333333333
```

### Write Commands

Add a target:

```bash
cargo run -- --network testnet --private-key 0x... allowlist add \
  0x3333333333333333333333333333333333333333
```

Remove a target:

```bash
cargo run -- --network testnet --private-key 0x... allowlist remove \
  0x3333333333333333333333333333333333333333
```

Transfer admin:

```bash
cargo run -- --network testnet --private-key 0x... allowlist transfer-admin \
  0x4444444444444444444444444444444444444444
```

## Typical Workflows

### 1. Validate Environment

```bash
cargo run -- --network testnet network
cargo run -- --network testnet factory info
```

### 2. Create and Approve a Raise

```bash
cargo run -- --network testnet --private-key 0x... factory create \
  --agent-id 0 \
  --name "Backed Demo" \
  --description "Demo raise" \
  --categories "defi" \
  --token-name BACKED \
  --token-symbol BACKED \
  --duration-minutes 30 \
  --launch-in-minutes 2

cargo run -- --network testnet --private-key 0x... factory approve 0
```

### 3. Commit to a Sale

```bash
cargo run -- --network testnet --private-key 0x... sale approve-collateral --project-id 0 100
cargo run -- --network testnet --private-key 0x... sale commit --project-id 0 100
```

### 4. Finalize and Claim

```bash
cargo run -- --network testnet --private-key 0x... sale finalize --project-id 0
cargo run -- --network testnet --private-key 0x... sale claim --project-id 0
```

## Troubleshooting

### Wrong Contract Address

If a command points to an outdated deployment:

- override `--factory`
- override `--allowlist`
- use `--sale` directly for sale commands

### ABI Mismatch or Revert on Read

The CLI is designed for the current contracts in `backend`.
If the deployment JSON points to older contracts, some reads may revert.

Fix:

- align `backend/deployments/*.json` with the active contracts
- or pass explicit addresses with CLI flags

### Missing Signer

If a write command fails immediately, verify one of:

- `--private-key`
- `BACKED_PRIVATE_KEY`

### RPC Issues

If the CLI cannot connect:

- verify `BACKED_RPC_URL` or `--rpc-url`
- verify the RPC matches the selected network
- rerun `network` to inspect the resolved configuration

## Verification

Run:

```bash
cargo fmt
cargo check
```

## Current Limitations

- signer support is currently private-key based
- there is no keystore or hardware-wallet integration yet
- terminal output is intentionally plain and script-friendly
