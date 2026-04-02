# backed-cli

`backed-cli` is the command-line interface for the Backed protocol contracts defined in `backend`.

It resolves configuration from `backend/deployments`, exposes the current protocol operations, and is intended to be used as a shell command named `backed`.

## Overview

This CLI covers three contract areas:

- `factory`: project creation, approval, metadata, operational status, and global configuration
- `sale`: sale status, commitments, claim/refund flows, and collateral approval
- `allowlist`: executor target management

When you create a raise, the CLI also lets you configure a fund term lockup. Investors can `claim` shares right after a successful finalization, but they cannot redeem those shares back into collateral until the fund term has ended and the treasury has finalized settlement.

## Quick Start

### 1. Install dependencies

```bash
npm install
```

### 2. Register the `backed` command locally

```bash
npm link
```

### 3. Verify that the CLI is available

```bash
backed --help
```

### 4. Check the resolved network configuration

```bash
backed network
```

## Requirements

- Node.js `>= 24`
- npm
- access to the target RPC endpoint
- a private key only for write operations

## Configuration

By default, the CLI reads deployment data from:

- `backend/deployments/megaeth-testnet.json`
- `backend/deployments/megaeth-mainnet.json`

### Global flags

```bash
backed \
  --network testnet \
  --rpc-url https://example-rpc \
  --factory 0x... \
  --allowlist 0x... \
  --private-key 0x... \
  <command>
```

### Environment variables

- `BACKED_NETWORK`
- `BACKED_RPC_URL`
- `BACKED_FACTORY`
- `BACKED_ALLOWLIST`
- `BACKED_PRIVATE_KEY`

### Supported networks

- `testnet`
- `mainnet`

Example:

```bash
backed --network testnet network
```

## The Fundamental Operations

This section covers the most common tasks in the order you will usually need them.

### Inspect the current environment

Use this first to verify RPC, chain, contract addresses, and default collateral:

```bash
backed network
backed factory info
```

### Create a project

```bash
backed --network testnet --private-key 0x... factory create \
  --agent-id 0 \
  --name "Backed Demo" \
  --description "Initial validation raise" \
  --categories "defi,infra" \
  --token-name BACKED \
  --token-symbol BACKED \
  --duration-minutes 60 \
  --lockup-minutes 1440 \
  --launch-in-minutes 5
```

Notes:

- `--collateral` is optional if the deployment file already defines a default collateral
- `--launch-in-minutes` is optional; default is `0`
- `--lockup-minutes` is optional; default is `0`

### Approve a project

The sale cannot accept commitments until the project is approved:

```bash
backed --network testnet --private-key 0x... factory approve 0
```

### Inspect a project and its raise state

```bash
backed --network testnet factory project 0
backed --network testnet factory snapshot 0
```

### Approve collateral for a sale

You must approve the sale contract before committing ERC-20 collateral:

```bash
backed --network testnet --private-key 0x... sale approve-collateral \
  --project-id 0 \
  100
```

### Commit to a sale

```bash
backed --network testnet --private-key 0x... sale commit \
  --project-id 0 \
  100
```

The CLI checks allowance and token balance before sending the transaction.

### Finalize a sale

```bash
backed --network testnet --private-key 0x... sale finalize --project-id 0
```

### Claim or refund

Claim after a successful sale:

```bash
backed --network testnet --private-key 0x... sale claim --project-id 0
```

`claim` transfers fund shares into the wallet. Those shares become redeemable for collateral only after the treasury unwinds and finalizes settlement at the end of the configured fund term.

Refund when the sale failed:

```bash
backed --network testnet --private-key 0x... sale refund --project-id 0
```

### Manage the allowlist

Inspect the current admin:

```bash
backed allowlist info
```

Allow a target:

```bash
backed --network testnet --private-key 0x... allowlist add \
  0x3333333333333333333333333333333333333333
```

## Installation and Local Usage

### Run without linking

If you do not want the global command yet:

```bash
node ./src/bin/backed.ts --help
```

### Local development scripts

```bash
npm run dev -- --help
npm run check
npm run build
```

Current behavior:

- `npm run dev` runs the CLI entrypoint directly
- `npm run check` performs a smoke test
- `npm run build` validates the executable entrypoint in the current runtime
- `factory create --lockup-minutes <n>` configures how long the fund term lasts after the raise ends before settlement and redemption can open

## Command Reference

## `network`

Show the resolved runtime configuration:

```bash
backed network
```

## `factory`

### Read commands

Factory summary:

```bash
backed factory info
```

Global configuration:

```bash
backed factory global
```

Collateral inspection:

```bash
backed factory collateral 0x9f5A17BD53310D012544966b8e3cF7863fc8F05f
```

Paginated project list:

```bash
backed factory list --from 0 --limit 20
```

Project details:

```bash
backed factory project 0
```

Raise snapshot:

```bash
backed factory snapshot 0
```

User commitment via factory:

```bash
backed factory commitment 0 0x1111111111111111111111111111111111111111
```

Projects for an agent id:

```bash
backed factory agent-projects 0
```

### Write commands

Approve:

```bash
backed --private-key 0x... factory approve 0
```

Revoke:

```bash
backed --private-key 0x... factory revoke 0
```

Update metadata:

```bash
backed --private-key 0x... factory update-metadata \
  --project-id 0 \
  --description "Updated description" \
  --categories "defi,ai"
```

Update status:

```bash
backed --private-key 0x... factory set-status \
  --project-id 0 \
  --status operating \
  --status-note "Treasury live"
```

Set collateral:

```bash
backed --private-key 0x... factory set-collateral \
  0x9f5A17BD53310D012544966b8e3cF7863fc8F05f true
```

Set global config:

```bash
backed --private-key 0x... factory set-global \
  --min-raise 100 \
  --max-raise 100000 \
  --platform-fee-bps 100 \
  --platform-fee-recipient 0x1111111111111111111111111111111111111111
```

## `sale`

Sale commands support either:

- `--sale <sale-address>`
- `--project-id <project-id>`

### Read commands

Status:

```bash
backed sale status --project-id 0
```

Claimable:

```bash
backed sale claimable --project-id 0 0x1111111111111111111111111111111111111111
```

Refundable:

```bash
backed sale refundable --project-id 0 0x1111111111111111111111111111111111111111
```

Commitment:

```bash
backed sale commitment --project-id 0 0x1111111111111111111111111111111111111111
```

### Write commands

Approve collateral:

```bash
backed --private-key 0x... sale approve-collateral --project-id 0 100
```

Commit:

```bash
backed --private-key 0x... sale commit --project-id 0 100
```

Finalize:

```bash
backed --private-key 0x... sale finalize --project-id 0
```

Claim:

```bash
backed --private-key 0x... sale claim --project-id 0
```

Refund:

```bash
backed --private-key 0x... sale refund --project-id 0
```

Emergency refund:

```bash
backed --private-key 0x... sale emergency-refund --project-id 0
```

### Amount handling

`sale approve-collateral` and `sale commit` accept human-readable amounts by default.

Example:

```bash
backed --private-key 0x... sale commit --project-id 0 100.5
```

Use `--raw` to pass raw `uint256` values:

```bash
backed --private-key 0x... sale commit --project-id 0 100500000 --raw
```

## `allowlist`

### Read commands

Admin:

```bash
backed allowlist info
```

Check target:

```bash
backed allowlist is-allowed 0x3333333333333333333333333333333333333333
```

### Write commands

Add target:

```bash
backed --private-key 0x... allowlist add 0x3333333333333333333333333333333333333333
```

Remove target:

```bash
backed --private-key 0x... allowlist remove 0x3333333333333333333333333333333333333333
```

Transfer admin:

```bash
backed --private-key 0x... allowlist transfer-admin 0x4444444444444444444444444444444444444444
```

## Troubleshooting

### `backed` is not available in the shell

Run:

```bash
npm link
```

If needed, verify that the npm global bin directory is in `PATH`.

### A write command fails before sending a transaction

Provide one of:

- `--private-key`
- `BACKED_PRIVATE_KEY`

### A command resolves the wrong deployment

Override the relevant values:

- `--rpc-url`
- `--factory`
- `--allowlist`
- `--sale`

### A read call reverts

The deployment JSON is usually out of sync with the active contracts.

Recommended action:

- update `backend/deployments/*.json`
- or pass explicit addresses through CLI flags

### RPC issues

Verify:

- selected network
- RPC reachability
- deployment RPC value

## Internal Structure

```text
src/
  bin/
    backed.ts
  cli/
    help.ts
    parser.ts
    types.ts
  commands/
    allowlist.ts
    factory.ts
    network.ts
    sale.ts
  chain/
    abis.ts
    client.ts
    contracts.ts
  config/
    runtime.ts
  lib/
    output.ts
    utils.ts
  types/
    project.ts
```
