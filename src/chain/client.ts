import { Contract, JsonRpcProvider, Wallet, getAddress, isAddress, type ContractRunner } from "ethers";
import type { GlobalOptions, SaleTarget } from "../cli/types.ts";
import { CliUserError } from "../cli/parser.ts";
import type { RuntimeConfig } from "../config/runtime.ts";
import { ALLOWLIST_ABI, ERC20_ABI, FACTORY_ABI, FACTORY_ADMIN_ABI, SALE_ABI } from "./abis.ts";

export function createReadClient(config: RuntimeConfig): JsonRpcProvider {
  return new JsonRpcProvider(config.rpcUrl, config.chainId);
}

export function createWriteClient(global: GlobalOptions, config: RuntimeConfig): Wallet {
  const privateKey = global.privateKey ?? process.env.BACKED_PRIVATE_KEY;
  if (!privateKey) {
    throw new CliUserError("write command requires --private-key or BACKED_PRIVATE_KEY");
  }
  return new Wallet(privateKey, createReadClient(config));
}

export function createFactoryContract(address: string, runner: ContractRunner): Contract {
  return new Contract(getAddress(address), FACTORY_ABI, runner);
}

export function createFactoryAdminContract(address: string, runner: ContractRunner): Contract {
  return new Contract(getAddress(address), FACTORY_ADMIN_ABI, runner);
}

export function createSaleContract(address: string, runner: ContractRunner): Contract {
  return new Contract(getAddress(address), SALE_ABI, runner);
}

export function createAllowlistContract(address: string, runner: ContractRunner): Contract {
  return new Contract(getAddress(address), ALLOWLIST_ABI, runner);
}

export function createErc20Contract(address: string, runner: ContractRunner): Contract {
  return new Contract(getAddress(address), ERC20_ABI, runner);
}

export function normalizeAddress(value: string, label: string): string {
  if (!isAddress(value)) {
    throw new CliUserError(`invalid address for ${label}: ${value}`);
  }
  return getAddress(value);
}

export function ensureSaleTarget(target: SaleTarget): SaleTarget {
  if (!target.sale && target.projectId === undefined) {
    throw new CliUserError("sale target required: use --sale or --project-id");
  }
  if (target.sale && target.projectId !== undefined) {
    throw new CliUserError("use either --sale or --project-id, not both");
  }
  return target;
}
