import type { RuntimeConfig } from "../config/runtime.ts";
import { printNetwork } from "../lib/output.ts";

export function runNetwork(config: RuntimeConfig): void {
  printNetwork(config);
}
