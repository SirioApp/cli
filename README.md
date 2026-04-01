# backed-cli

`backed-cli` is the command-line interface for the current Backed backend contracts.

It is implemented in TypeScript and runs directly on Node.js 24+, with a real terminal command:

```bash
backed
```

No Rust toolchain is required.

## Purpose

The CLI is designed around the contracts and deployment artifacts inside `backend`:

- `AgentRaiseFactory`
- `Sale`
- `ContractAllowlist`
- `backend/deployments/*.json`

It resolves addresses and RPC configuration from the deployment files by default, while still allowing explicit overrides from flags or environment variables.

## Requirements

- Node.js `>= 24`
- npm
- access to an RPC endpoint for the target network
- a private key only when running write commands

## Installation

Install dependencies:

```bash
npm install
```

Expose the CLI globally on your machine:

```bash
npm link
```

After that, the command is available as:

```bash
backed --help
```

If you do not want a global link yet, you can still run it locally:

```bash
node ./src/bin/backed.ts --help
```

## Development Scripts

Run the CLI locally:

```bash
npm run dev -- --help
```

Run the smoke check:

```bash
npm run check
```

The current `build` script validates the executable entrypoint under Node 24:

```bash
npm run build
```

## Runtime Configuration

The CLI resolves deployment data from:

- `backend/deployments/megaeth-testnet.json`
- `backend/deployments/megaeth-mainnet.json`

### Global Flags

```bash
backed \
  --network testnet \
  --rpc-url https://example-rpc \
  --factory 0x... \
  --allowlist 0x... \
  --private-key 0x...
```

### Supported Environment Variables

- `BACKED_NETWORK`
- `BACKED_RPC_URL`
- `BACKED_FACTORY`
- `BACKED_ALLOWLIST`
- `BACKED_PRIVATE_KEY`

### Network Selection

Supported values:

- `testnet`
- `mainnet`

Example:

```bash
backed --network testnet network
```

## Command Overview

The CLI is split into four areas:

- `network`
- `factory`
- `sale`
- `allowlist`

Show the full help page:

```bash
backed --help
```

## `network`

Displays the resolved runtime configuration:

```bash
backed --network testnet network
```

Typical output includes:

- selected network
- deployment label
- chain id
- RPC URL
- factory address
- allowlist address
- default collateral
- deployment file path

## `factory`

Factory commands operate on `AgentRaiseFactory`.

### Read Commands

Show high-level factory data:

```bash
backed --network testnet factory info
```

Show only the global configuration:

```bash
backed --network testnet factory global
```

Inspect one collateral:

```bash
backed --network testnet factory collateral 0x9f5A17BD53310D012544966b8e3cF7863fc8F05f
```

List projects with pagination:

```bash
backed --network testnet factory list --from 0 --limit 20
```

Inspect one project:

```bash
backed --network testnet factory project 0
```

Inspect raise snapshot:

```bash
backed --network testnet factory snapshot 0
```

Inspect one user commitment through the factory:

```bash
backed --network testnet factory commitment 0 0x1111111111111111111111111111111111111111
```

List project ids for an agent id:

```bash
backed --network testnet factory agent-projects 0
```

### Write Commands

Create a project:

```bash
backed --network testnet --private-key 0x... factory create \
  --agent-id 0 \
  --name "Backed Demo" \
  --description "Demo raise used for validation" \
  --categories "defi,infra" \
  --token-name BACKED \
  --token-symbol BACKED \
  --duration-minutes 60 \
  --launch-in-minutes 5 \
  --collateral 0x9f5A17BD53310D012544966b8e3cF7863fc8F05f
```

Approve a project:

```bash
backed --network testnet --private-key 0x... factory approve 0
```

Revoke a project:

```bash
backed --network testnet --private-key 0x... factory revoke 0
```

Update metadata:

```bash
backed --network testnet --private-key 0x... factory update-metadata \
  --project-id 0 \
  --description "Updated description" \
  --categories "defi,ai"
```

Update operational status:

```bash
backed --network testnet --private-key 0x... factory set-status \
  --project-id 0 \
  --status operating \
  --status-note "Treasury live"
```

Enable or disable a collateral:

```bash
backed --network testnet --private-key 0x... factory set-collateral \
  0x9f5A17BD53310D012544966b8e3cF7863fc8F05f true
```

Update global config:

```bash
backed --network testnet --private-key 0x... factory set-global \
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

- `duration-minutes` and `launch-in-minutes` are converted internally to seconds
- if `--collateral` is omitted during create, the CLI uses the default collateral from the deployment file when available
- factory views assume the deployment ABI matches the current contracts in `backend`

## `sale`

Sale commands can target a sale in two ways:

- direct sale address with `--sale`
- indirect lookup through `--project-id`

### Read Commands

Show sale status:

```bash
backed --network testnet sale status --sale 0x2222222222222222222222222222222222222222
```

Show claimable amounts for one user:

```bash
backed --network testnet sale claimable \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

Show refundable amount for one user:

```bash
backed --network testnet sale refundable \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

Show committed amount for one user:

```bash
backed --network testnet sale commitment \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

### Write Commands

Approve collateral:

```bash
backed --network testnet --private-key 0x... sale approve-collateral \
  --project-id 0 \
  100
```

Commit collateral:

```bash
backed --network testnet --private-key 0x... sale commit \
  --project-id 0 \
  100
```

Finalize:

```bash
backed --network testnet --private-key 0x... sale finalize --project-id 0
```

Claim:

```bash
backed --network testnet --private-key 0x... sale claim --project-id 0
```

Refund:

```bash
backed --network testnet --private-key 0x... sale refund --project-id 0
```

Emergency refund:

```bash
backed --network testnet --private-key 0x... sale emergency-refund --project-id 0
```

### Human-Readable vs Raw Amounts

These commands accept human-readable token amounts by default:

- `sale approve-collateral`
- `sale commit`

Example:

```bash
backed --network testnet --private-key 0x... sale commit --project-id 0 100.5
```

To pass raw `uint256` values, add `--raw`:

```bash
backed --network testnet --private-key 0x... sale commit --project-id 0 100500000 --raw
```

### Safety Checks

Before sending `sale commit`, the CLI checks:

- ERC-20 allowance
- ERC-20 balance

If either is insufficient, the transaction is not sent.

## `allowlist`

Allowlist commands operate on `ContractAllowlist`.

### Read Commands

Show current admin:

```bash
backed --network testnet allowlist info
```

Check whether a target is allowed:

```bash
backed --network testnet allowlist is-allowed \
  0x3333333333333333333333333333333333333333
```

### Write Commands

Add a target:

```bash
backed --network testnet --private-key 0x... allowlist add \
  0x3333333333333333333333333333333333333333
```

Remove a target:

```bash
backed --network testnet --private-key 0x... allowlist remove \
  0x3333333333333333333333333333333333333333
```

Transfer admin:

```bash
backed --network testnet --private-key 0x... allowlist transfer-admin \
  0x4444444444444444444444444444444444444444
```

## Typical Workflows

### Validate the Environment

```bash
backed --network testnet network
backed --network testnet factory info
```

### Create and Approve a Raise

```bash
backed --network testnet --private-key 0x... factory create \
  --agent-id 0 \
  --name "Backed Demo" \
  --description "Demo raise" \
  --categories "defi" \
  --token-name BACKED \
  --token-symbol BACKED \
  --duration-minutes 30 \
  --launch-in-minutes 2

backed --network testnet --private-key 0x... factory approve 0
```

### Commit to a Sale

```bash
backed --network testnet --private-key 0x... sale approve-collateral --project-id 0 100
backed --network testnet --private-key 0x... sale commit --project-id 0 100
```

### Finalize and Claim

```bash
backed --network testnet --private-key 0x... sale finalize --project-id 0
backed --network testnet --private-key 0x... sale claim --project-id 0
```

## Architecture

The project is intentionally split by responsibility:

```text
src/
  bin/
    backed.ts         # executable entrypoint
  cli/
    help.ts           # help output
    parser.ts         # argument parsing and validation
    types.ts          # CLI command types
  commands/
    allowlist.ts      # allowlist handlers
    factory.ts        # factory handlers
    network.ts        # network handler
    sale.ts           # sale handlers
  chain/
    abis.ts           # contract ABIs
    client.ts         # provider, wallet, contract factories
    contracts.ts      # higher-level chain helpers
  config/
    runtime.ts        # deployment resolution and runtime config
  lib/
    output.ts         # terminal printing helpers
    utils.ts          # amount, time, tx helpers
  types/
    project.ts        # project view type
```

## Troubleshooting

### The command `backed` is not found

Run:

```bash
npm link
```

If needed, verify your npm global bin directory is in `PATH`.

### A write command fails immediately

Verify one of the following is present:

- `--private-key`
- `BACKED_PRIVATE_KEY`

### A command points to the wrong deployment

Override one or more of:

- `--rpc-url`
- `--factory`
- `--allowlist`
- `--sale`

### A read call reverts

This usually means the deployment JSON does not match the current contract version.

Fix:

- align `backend/deployments/*.json` with the current deployed contracts
- or pass explicit addresses through CLI flags

### RPC issues

Verify:

- the selected network is correct
- the RPC endpoint is reachable
- the deployment file contains the intended RPC URL

## Verification

Smoke check the executable:

```bash
npm run check
```

Inspect the resolved environment:

```bash
backed network
```
