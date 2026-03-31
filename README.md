# backed-cli

Rust CLI to operate the Backed backend contracts (Factory, Sale, Allowlist) using the deployment artifacts in `backend/deployments`.

## Prerequisites

- Rust + Cargo
- RPC access to the chosen network
- Private key for write operations

## Build

```bash
cd /Users/lucatropea/Desktop/backed/backed-cli
cargo build
```

## Configuration

Deployment files auto-loaded from the repo root:

- `backend/deployments/megaeth-testnet.json`
- `backend/deployments/megaeth-mainnet.json`

CLI/env overrides:

- `--network testnet|mainnet` / `BACKED_NETWORK`
- `--rpc-url` / `BACKED_RPC_URL`
- `--factory` / `BACKED_FACTORY`
- `--allowlist` / `BACKED_ALLOWLIST`
- `--private-key` / `BACKED_PRIVATE_KEY` (required for write)

## Commands

### Network

```bash
cargo run -- --network testnet network
```

### Factory (read)

```bash
cargo run -- --network testnet factory info
cargo run -- --network testnet factory global
cargo run -- --network testnet factory collateral 0x9f5A17BD53310D012544966b8e3cF7863fc8F05f
cargo run -- --network testnet factory list --from 0 --limit 20
cargo run -- --network testnet factory project 0
cargo run -- --network testnet factory snapshot 0
cargo run -- --network testnet factory commitment --project-id 0 --user 0xUser
cargo run -- --network testnet factory agent-projects 0
```

### Factory (write)

```bash
cargo run -- --network testnet --private-key <PK> factory create \
  --agent-id 0 \
  --name Backed Demo \
  --description "Backed sample raise" \
  --categories "defi,infra" \
  --token-name BACKED \
  --token-symbol BACKED \
  --duration-minutes 10 \
  --launch-in-minutes 1 \
  --collateral 0x9f5A17BD53310D012544966b8e3cF7863fc8F05f

cargo run -- --network testnet --private-key <PK> factory approve 0
cargo run -- --network testnet --private-key <PK> factory revoke 0
cargo run -- --network testnet --private-key <PK> factory update-metadata \
  --project-id 0 \
  --description "Updated deck link" \
  --categories "defi,infra"
cargo run -- --network testnet --private-key <PK> factory set-status \
  --project-id 0 \
  --status operating \
  --status-note "mainnet live"
cargo run -- --network testnet --private-key <PK> factory set-collateral 0x... true

cargo run -- --network testnet --private-key <PK> factory set-global \
  --min-raise 100 \
  --max-raise 100000 \
  --platform-fee-bps 100 \
  --platform-fee-recipient 0x... \
  --min-duration-minutes 10 \
  --max-duration-minutes 1440 \
  --min-launch-delay-minutes 0 \
  --max-launch-delay-minutes 1440
```

### Sale

Target a sale with `--sale <SALE_ADDRESS>` or `--project-id <ID>`.

Read:

```bash
cargo run -- --network testnet sale status --sale 0x...
cargo run -- --network testnet sale claimable --sale 0x... 0xUser
cargo run -- --network testnet sale refundable --sale 0x... 0xUser
cargo run -- --network testnet sale commitment --sale 0x... 0xUser
```

Write:

```bash
cargo run -- --network testnet --private-key <PK> sale approve-collateral --sale 0x... 100
cargo run -- --network testnet --private-key <PK> sale commit --sale 0x... 100
cargo run -- --network testnet --private-key <PK> sale finalize --sale 0x...
cargo run -- --network testnet --private-key <PK> sale claim --sale 0x...
cargo run -- --network testnet --private-key <PK> sale refund --sale 0x...
cargo run -- --network testnet --private-key <PK> sale emergency-refund --sale 0x...
```

`approve-collateral` and `commit` accept human-readable token amounts by default; use `--raw` to pass uint256 values directly.

### Allowlist

```bash
cargo run -- --network testnet allowlist info
cargo run -- --network testnet allowlist is-allowed 0x...

cargo run -- --network testnet --private-key <PK> allowlist add 0x...
cargo run -- --network testnet --private-key <PK> allowlist remove 0x...
cargo run -- --network testnet --private-key <PK> allowlist transfer-admin 0x...
```

## Notes

- Write commands require the correct on-chain admin/owner role.
- If a call reverts due to legacy deployments, pass explicit `--sale` or update `backend/deployments/*.json` to current contract addresses.
- `sale commit` checks allowance and balance before sending transactions.
