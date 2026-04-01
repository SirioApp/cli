#!/usr/bin/env node

import { CliUserError, parseCli } from "../cli/parser.ts";
import { runAllowlist } from "../commands/allowlist.ts";
import { runFactory } from "../commands/factory.ts";
import { runNetwork } from "../commands/network.ts";
import { runSale } from "../commands/sale.ts";
import { resolveRuntimeConfig } from "../config/runtime.ts";

async function main(): Promise<void> {
  const parsed = parseCli(process.argv.slice(2));
  const config = resolveRuntimeConfig(parsed.global);

  switch (parsed.command.scope) {
    case "network":
      runNetwork(config);
      return;
    case "factory":
      await runFactory(parsed.global, config, parsed.command.command);
      return;
    case "sale":
      await runSale(parsed.global, config, parsed.command.command);
      return;
    case "allowlist":
      await runAllowlist(parsed.global, config, parsed.command.command);
      return;
  }
}

main().catch((error: unknown) => {
  if (error instanceof CliUserError) {
    if (error.showHelp) {
      console.error(error.message);
      process.exit(1);
    }
    console.log(error.message);
    process.exit(0);
  }

  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
