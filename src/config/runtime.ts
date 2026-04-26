import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { cwd } from "node:process";
import { isAddress } from "ethers";
import type { GlobalOptions, NetworkName } from "../cli/types.ts";
import { CliUserError } from "../cli/parser.ts";

type DeploymentFile = {
  network: string;
  chainId: number;
  rpc: string;
  contracts: Record<string, string>;
  external?: Record<string, string>;
};

export type RuntimeConfig = {
  network: NetworkName;
  networkLabel: string;
  chainId: number;
  rpcUrl: string;
  factory: string;
  allowlist: string;
  defaultCollateral?: string;
  deploymentPath: string;
  repoRoot: string;
  contracts: Record<string, string>;
  external: Record<string, string>;
};

export function resolveRuntimeConfig(global: GlobalOptions): RuntimeConfig {
  const repoRoot = findRepoRoot(cwd());
  const deploymentPath = join(repoRoot, "frontend", "config", deploymentFileName(global.network));
  const deployment = readJson<DeploymentFile>(deploymentPath);
  const factory = normalizeAddress(
    global.factory ?? process.env.BACKED_FACTORY ?? deployment.contracts.AgentRaiseFactory,
    "AgentRaiseFactory",
  );
  const allowlist = normalizeAddress(
    global.allowlist ?? process.env.BACKED_ALLOWLIST ?? deployment.contracts.ContractAllowlist,
    "ContractAllowlist",
  );
  const defaultCollateral = deployment.external?.USDM
    ? normalizeAddress(deployment.external.USDM, "USDM")
    : undefined;

  return {
    network: global.network,
    networkLabel: deployment.network,
    chainId: deployment.chainId,
    rpcUrl: global.rpcUrl ?? process.env.BACKED_RPC_URL ?? deployment.rpc,
    factory,
    allowlist,
    deploymentPath,
    repoRoot,
    contracts: deployment.contracts,
    external: deployment.external ?? {},
    ...(defaultCollateral ? { defaultCollateral } : {}),
  };
}

function deploymentFileName(network: NetworkName): string {
  return network === "testnet" ? "deployment.testnet.json" : "deployment.mainnet.json";
}

function findRepoRoot(startPath: string): string {
  let current = startPath;
  while (true) {
    try {
      readFileSync(join(current, "frontend", "config", "deployment.testnet.json"));
      return current;
    } catch {}

    const parent = dirname(current);
    if (parent === current) {
      throw new CliUserError("cannot locate repo root containing frontend/config deployment files");
    }
    current = parent;
  }
}

function readJson<T>(path: string): T {
  try {
    return JSON.parse(readFileSync(path, "utf8")) as T;
  } catch (error) {
    throw new CliUserError(`failed to read deployment file: ${path}\n${String(error)}`);
  }
}

function normalizeAddress(value: string | undefined, label: string): string {
  if (!value || !isAddress(value)) {
    throw new CliUserError(`invalid address for ${label}`);
  }
  return value;
}
