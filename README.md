# backed-cli

`backed-cli` is the command-line interface for the Backed protocol contracts and deployment artifacts maintained in `backend`.

The CLI resolves network configuration from `backend/deployments`, exposes the current protocol surfaces (`factory`, `sale`, `allowlist`), and is designed to be installed as a shell command named `backed`.

## Scope

The CLI targets the current backend contract stack:

- `AgentRaiseFactory`
- `Sale`
- `ContractAllowlist`

Default configuration is loaded from:

- `backend/deployments/megaeth-testnet.json`
- `backend/deployments/megaeth-mainnet.json`

## Installation

### Requirements

- Node.js `>= 24`
- npm
- access to a valid RPC endpoint for the selected network

### Local setup

```bash
npm install
```

### Register the `backed` command locally

```bash
npm link
```

After linking, the CLI is available as:

```bash
backed --help
```

### Local execution without linking

```bash
node ./src/bin/backed.ts --help
```

## Build and Verification

Project scripts:

```bash
npm run dev -- --help
npm run check
npm run build
```

Current behavior:

- `npm run dev` executes the CLI entrypoint directly
- `npm run check` performs a CLI smoke test
- `npm run build` validates the executable entrypoint in the current Node runtime

## Runtime Configuration

### Global flags

```bash
backed \
  --network testnet \
  --rpc-url https://example-rpc \
  --factory 0x... \
  --allowlist 0x... \
  --private-key 0x...
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

## Command Model

The CLI exposes four top-level command groups:

- `network`
- `factory`
- `sale`
- `allowlist`

Full command help:

```bash
backed --help
```

## Network Commands

Display the resolved runtime configuration:

```bash
backed --network testnet network
```

The output includes:

- selected network
- deployment label
- chain id
- RPC URL
- factory address
- allowlist address
- default collateral, when defined
- deployment file path

## Factory Commands

Factory commands operate on `AgentRaiseFactory`.

### Read operations

Factory summary:

```bash
backed --network testnet factory info
```

Global configuration only:

```bash
backed --network testnet factory global
```

Collateral inspection:

```bash
backed --network testnet factory collateral 0x9f5A17BD53310D012544966b8e3cF7863fc8F05f
```

Project pagination:

```bash
backed --network testnet factory list --from 0 --limit 20
```

Single project view:

```bash
backed --network testnet factory project 0
```

Project raise snapshot:

```bash
backed --network testnet factory snapshot 0
```

User commitment through the factory:

```bash
backed --network testnet factory commitment 0 0x1111111111111111111111111111111111111111
```

Projects for an agent id:

```bash
backed --network testnet factory agent-projects 0
```

### Write operations

Project creation:

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

Project approval:

```bash
backed --network testnet --private-key 0x... factory approve 0
```

Project revocation:

```bash
backed --network testnet --private-key 0x... factory revoke 0
```

Metadata update:

```bash
backed --network testnet --private-key 0x... factory update-metadata \
  --project-id 0 \
  --description "Updated description" \
  --categories "defi,ai"
```

Operational status update:

```bash
backed --network testnet --private-key 0x... factory set-status \
  --project-id 0 \
  --status operating \
  --status-note "Treasury live"
```

Collateral enable or disable:

```bash
backed --network testnet --private-key 0x... factory set-collateral \
  0x9f5A17BD53310D012544966b8e3cF7863fc8F05f true
```

Global configuration update:

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

### Notes

- `duration-minutes` and `launch-in-minutes` are converted to seconds internally.
- If `--collateral` is omitted during project creation, the CLI uses the default collateral from the deployment file when available.
- Factory read operations assume the deployment ABI is aligned with the current contracts in `backend`.

## Sale Commands

Sale commands accept one of the following target selectors:

- `--sale <sale-address>`
- `--project-id <project-id>`

### Read operations

Sale status:

```bash
backed --network testnet sale status --sale 0x2222222222222222222222222222222222222222
```

Claimable amounts:

```bash
backed --network testnet sale claimable \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

Refundable amount:

```bash
backed --network testnet sale refundable \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

Committed amount:

```bash
backed --network testnet sale commitment \
  --sale 0x2222222222222222222222222222222222222222 \
  0x1111111111111111111111111111111111111111
```

### Write operations

Collateral approval:

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

### Amount handling

The following commands accept human-readable token amounts by default:

- `sale approve-collateral`
- `sale commit`

Example:

```bash
backed --network testnet --private-key 0x... sale commit --project-id 0 100.5
```

To pass raw `uint256` values directly:

```bash
backed --network testnet --private-key 0x... sale commit --project-id 0 100500000 --raw
```

### Pre-flight checks

Before a `sale commit` transaction is sent, the CLI verifies:

- token allowance
- token balance

## Allowlist Commands

Allowlist commands operate on `ContractAllowlist`.

### Read operations

Current admin:

```bash
backed --network testnet allowlist info
```

Target allow status:

```bash
backed --network testnet allowlist is-allowed \
  0x3333333333333333333333333333333333333333
```

### Write operations

Add target:

```bash
backed --network testnet --private-key 0x... allowlist add \
  0x3333333333333333333333333333333333333333
```

Remove target:

```bash
backed --network testnet --private-key 0x... allowlist remove \
  0x3333333333333333333333333333333333333333
```

Transfer admin:

```bash
backed --network testnet --private-key 0x... allowlist transfer-admin \
  0x4444444444444444444444444444444444444444
```

## Standard Workflows

### Environment validation

```bash
backed --network testnet network
backed --network testnet factory info
```

### Project creation and approval

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

### Commit flow

```bash
backed --network testnet --private-key 0x... sale approve-collateral --project-id 0 100
backed --network testnet --private-key 0x... sale commit --project-id 0 100
```

### Finalization and claim

```bash
backed --network testnet --private-key 0x... sale finalize --project-id 0
backed --network testnet --private-key 0x... sale claim --project-id 0
```

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

## Troubleshooting

### `backed` is not available in the shell

Run:

```bash
npm link
```

If required, verify that the npm global bin directory is included in `PATH`.

### A write command fails before sending a transaction

Ensure one of the following is available:

- `--private-key`
- `BACKED_PRIVATE_KEY`

### A command resolves the wrong deployment

Override the relevant values explicitly:

- `--rpc-url`
- `--factory`
- `--allowlist`
- `--sale`

### A read call reverts

In most cases, the deployment JSON does not match the currently deployed contract version.

Recommended action:

- align `backend/deployments/*.json` with the active deployment
- or pass explicit addresses through CLI flags

### RPC connectivity issues

Verify:

- network selection
- RPC reachability
- deployment file RPC value

## Verification

Smoke test:

```bash
npm run check
```

Runtime resolution check:

```bash
backed network
```
