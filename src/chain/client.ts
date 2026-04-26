import {
  Contract,
  JsonRpcProvider,
  Wallet,
  getAddress,
  isAddress,
  type ContractRunner,
  type ContractTransactionResponse,
} from "ethers";
import type { GlobalOptions, SaleTarget } from "../cli/types.ts";
import { CliUserError } from "../cli/parser.ts";
import type { RuntimeConfig } from "../config/runtime.ts";
import {
  ALLOWLIST_ABI,
  ERC20_ABI,
  FACTORY_ABI,
  FACTORY_ADMIN_ABI,
  SALE_ABI,
  SHARE_TOKEN_ABI,
} from "./abis.ts";

export type FactoryContract = Contract & {
  projectCount(): Promise<bigint>;
  globalConfig(): Promise<readonly [bigint, bigint, number, string, bigint, bigint, bigint, bigint]>;
  allowedCollateral(collateral: string): Promise<boolean>;
  minRaiseForCollateral(collateral: string): Promise<bigint>;
  maxRaiseForCollateral(collateral: string): Promise<bigint>;
  isProjectApproved(projectId: bigint): Promise<boolean>;
  getProjectRaiseSnapshot(projectId: bigint): Promise<readonly [boolean, bigint, bigint, boolean, boolean, boolean, bigint, bigint, string]>;
  getProjectCommitment(projectId: bigint, user: string): Promise<bigint>;
  getAgentProjects(agentId: bigint): Promise<readonly bigint[]>;
  getProject(projectId: bigint): Promise<
    readonly [bigint, string, string, string, string, string, string, string, string, number, string, bigint, bigint]
  >;
  approveProject(projectId: bigint): Promise<ContractTransactionResponse>;
  revokeProject(projectId: bigint): Promise<ContractTransactionResponse>;
  updateProjectMetadata(projectId: bigint, description: string, categories: string): Promise<ContractTransactionResponse>;
  updateProjectOperationalStatus(projectId: bigint, status: number, statusNote: string): Promise<ContractTransactionResponse>;
  setAllowedCollateral(collateral: string, allowed: boolean): Promise<ContractTransactionResponse>;
  setGlobalConfig(
    config: readonly [bigint, bigint, number, string, bigint, bigint, bigint, bigint],
  ): Promise<ContractTransactionResponse>;
  "createAgentRaise(uint256,string,string,string,address,address,uint256,uint256,uint256,string,string)"(
    agentId: bigint,
    name: string,
    description: string,
    categories: string,
    agent: string,
    collateral: string,
    durationSeconds: bigint,
    launchTimestamp: bigint,
    lockupMinutes: bigint,
    tokenName: string,
    tokenSymbol: string,
  ): Promise<ContractTransactionResponse>;
};

export type SaleContract = Contract & {
  COLLATERAL(): Promise<string>;
  PROJECT_ID(): Promise<bigint>;
  getStatus(): Promise<readonly [bigint, bigint, boolean, boolean]>;
  startTime(): Promise<bigint>;
  endTime(): Promise<bigint>;
  isActive(): Promise<boolean>;
  timeRemaining(): Promise<bigint>;
  token(): Promise<string>;
  getClaimable(user: string): Promise<readonly [bigint, bigint]>;
  getRefundable(user: string): Promise<bigint>;
  commitments(user: string): Promise<bigint>;
  finalize(): Promise<ContractTransactionResponse>;
  claim(): Promise<ContractTransactionResponse>;
  refund(): Promise<ContractTransactionResponse>;
  emergencyRefund(): Promise<ContractTransactionResponse>;
  commit(amount: bigint): Promise<ContractTransactionResponse>;
};

export type AllowlistContract = Contract & {
  admin(): Promise<string>;
  isAllowed(target: string): Promise<boolean>;
  addContract(target: string): Promise<ContractTransactionResponse>;
  removeContract(target: string): Promise<ContractTransactionResponse>;
  transferAdmin(newAdmin: string): Promise<ContractTransactionResponse>;
};

export type Erc20Contract = Contract & {
  decimals(): Promise<bigint>;
  symbol(): Promise<string>;
  allowance(owner: string, spender: string): Promise<bigint>;
  balanceOf(owner: string): Promise<bigint>;
  approve(spender: string, amount: bigint): Promise<ContractTransactionResponse>;
};

export type ShareTokenContract = Contract & {
  totalAssets(): Promise<bigint>;
  LOCKUP_END_TIME(): Promise<bigint>;
  settled(): Promise<boolean>;
};

export function createReadClient(config: RuntimeConfig): JsonRpcProvider {
  return new JsonRpcProvider(config.rpcUrl, config.chainId);
}

export function createWriteClient(global: GlobalOptions, config: RuntimeConfig): Wallet {
  const privateKey = global.privateKey ?? process.env.BACKED_PRIVATE_KEY ?? process.env.PRIVATE_KEY;
  if (!privateKey) {
    throw new CliUserError("write command requires --private-key, BACKED_PRIVATE_KEY, or PRIVATE_KEY");
  }
  return new Wallet(privateKey, createReadClient(config));
}

export function createFactoryContract(address: string, runner: ContractRunner): FactoryContract {
  return new Contract(getAddress(address), FACTORY_ABI, runner) as FactoryContract;
}

export function createFactoryAdminContract(address: string, runner: ContractRunner): FactoryContract {
  return new Contract(getAddress(address), FACTORY_ADMIN_ABI, runner) as FactoryContract;
}

export function createSaleContract(address: string, runner: ContractRunner): SaleContract {
  return new Contract(getAddress(address), SALE_ABI, runner) as SaleContract;
}

export function createAllowlistContract(address: string, runner: ContractRunner): AllowlistContract {
  return new Contract(getAddress(address), ALLOWLIST_ABI, runner) as AllowlistContract;
}

export function createErc20Contract(address: string, runner: ContractRunner): Erc20Contract {
  return new Contract(getAddress(address), ERC20_ABI, runner) as Erc20Contract;
}

export function createShareTokenContract(address: string, runner: ContractRunner): ShareTokenContract {
  return new Contract(getAddress(address), SHARE_TOKEN_ABI, runner) as ShareTokenContract;
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
