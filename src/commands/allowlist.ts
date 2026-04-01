import type { GlobalOptions, AllowlistCommand } from "../cli/types.ts";
import type { RuntimeConfig } from "../config/runtime.ts";
import { printReceipt } from "../lib/output.ts";
import { sendAndWait } from "../lib/utils.ts";
import {
  createAllowlistContract,
  createReadClient,
  createWriteClient,
  normalizeAddress,
} from "../chain/client.ts";

export async function runAllowlist(
  global: GlobalOptions,
  config: RuntimeConfig,
  command: AllowlistCommand,
): Promise<void> {
  const readClient = createReadClient(config);

  switch (command.kind) {
    case "info": {
      const allowlist = createAllowlistContract(config.allowlist, readClient);
      console.log(`allowlist: ${config.allowlist}`);
      console.log(`admin: ${String(await allowlist.admin())}`);
      return;
    }
    case "is-allowed": {
      const allowlist = createAllowlistContract(config.allowlist, readClient);
      const target = normalizeAddress(command.target, "target");
      console.log(`target: ${target}`);
      console.log(`allowed: ${Boolean(await allowlist.isAllowed(target))}`);
      return;
    }
    case "add": {
      const writeClient = createWriteClient(global, config);
      const allowlist = createAllowlistContract(config.allowlist, writeClient);
      printReceipt(await sendAndWait(allowlist.addContract(normalizeAddress(command.target, "target"))));
      return;
    }
    case "remove": {
      const writeClient = createWriteClient(global, config);
      const allowlist = createAllowlistContract(config.allowlist, writeClient);
      printReceipt(await sendAndWait(allowlist.removeContract(normalizeAddress(command.target, "target"))));
      return;
    }
    case "transfer-admin": {
      const writeClient = createWriteClient(global, config);
      const allowlist = createAllowlistContract(config.allowlist, writeClient);
      printReceipt(
        await sendAndWait(allowlist.transferAdmin(normalizeAddress(command.newAdmin, "new-admin"))),
      );
      return;
    }
  }
}
