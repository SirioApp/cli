export function renderHelp(): string {
  return `
backed

Backed protocol command-line interface.

Usage:
  backed [global options] <command> [subcommand] [options]

Global options:
  --network <testnet|mainnet>
  --rpc-url <url>
  --factory <address>
  --allowlist <address>
  --private-key <hex>        Overrides BACKED_PRIVATE_KEY / PRIVATE_KEY for write commands
  --help

Commands:
  network
  factory
  sale
  allowlist

Factory:
  backed factory info
  backed factory global
  backed factory collateral <address>
  backed factory list [--from <n>] [--limit <n>]
  backed factory project <id>
  backed factory snapshot <id>
  backed factory commitment <project-id> <user>
  backed factory agent-projects <agent-id>
  backed factory create --agent-id <id> --name <name> --description <text> --token-name <name> --token-symbol <symbol> --duration-minutes <n> [--lockup-minutes <n>] [--launch-in-minutes <n>] [--categories <csv>] [--agent-address <address>] [--collateral <address>]
  backed factory approve <project-id>
  backed factory revoke <project-id>
  backed factory update-metadata --project-id <id> --description <text> [--categories <csv>]
  backed factory set-status --project-id <id> --status <raising|deploying|operating|paused|closed> [--status-note <text>]
  backed factory set-collateral <address> <true|false>
  backed factory set-global --min-raise <amount> --max-raise <amount> --platform-fee-bps <bps> --platform-fee-recipient <address>

Sale:
  backed sale status (--sale <address> | --project-id <id>)
  backed sale claimable (--sale <address> | --project-id <id>) <user>
  backed sale refundable (--sale <address> | --project-id <id>) <user>
  backed sale commitment (--sale <address> | --project-id <id>) <user>
  backed sale approve-collateral (--sale <address> | --project-id <id>) <amount> [--raw]
  backed sale commit (--sale <address> | --project-id <id>) <amount> [--raw]
  backed sale finalize (--sale <address> | --project-id <id>)
  backed sale claim (--sale <address> | --project-id <id>)
  backed sale refund (--sale <address> | --project-id <id>)
  backed sale emergency-refund (--sale <address> | --project-id <id>)

Allowlist:
  backed allowlist info
  backed allowlist is-allowed <address>
  backed allowlist add <address>
  backed allowlist remove <address>
  backed allowlist transfer-admin <address>
`.trim();
}
