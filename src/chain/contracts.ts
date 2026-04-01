import { getAddress, type ContractRunner } from "ethers";
import { CliUserError } from "../cli/parser.ts";
import type { ProjectView } from "../types/project.ts";
import {
  createErc20Contract,
  createFactoryAdminContract,
  createFactoryContract,
  createSaleContract,
} from "./client.ts";

export async function readFactoryProject(
  factoryAddress: string,
  runner: ContractRunner,
  projectId: number,
): Promise<ProjectView> {
  const tuple = (await createFactoryContract(factoryAddress, runner).getProject(BigInt(projectId))) as readonly [
    bigint,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    number,
    string,
    bigint,
    bigint,
  ];

  return {
    agentId: tuple[0],
    name: tuple[1],
    description: tuple[2],
    categories: tuple[3],
    agent: getAddress(tuple[4]),
    treasury: getAddress(tuple[5]),
    sale: getAddress(tuple[6]),
    agentExecutor: getAddress(tuple[7]),
    collateral: getAddress(tuple[8]),
    operationalStatus: tuple[9],
    statusNote: tuple[10],
    createdAt: tuple[11],
    updatedAt: tuple[12],
  };
}

export async function readSaleCollateral(
  saleAddress: string,
  runner: ContractRunner,
): Promise<string> {
  return getAddress((await createSaleContract(saleAddress, runner).COLLATERAL()) as string);
}

export async function readSaleProjectId(
  saleAddress: string,
  runner: ContractRunner,
): Promise<bigint> {
  return (await createSaleContract(saleAddress, runner).PROJECT_ID()) as bigint;
}

export async function readTokenMetadata(
  tokenAddress: string,
  runner: ContractRunner,
): Promise<{ decimals: number; symbol: string }> {
  const token = createErc20Contract(tokenAddress, runner);
  const decimals = Number(await token.decimals());
  const symbol = String(await token.symbol().catch(() => "TOKEN"));
  return { decimals, symbol };
}

export async function sendFactoryGlobalConfig(
  factoryAddress: string,
  runner: ContractRunner,
  config: readonly [bigint, bigint, number, string, bigint, bigint, bigint, bigint],
) {
  const contract = createFactoryAdminContract(factoryAddress, runner);
  try {
    return await contract.setGlobalConfig(config);
  } catch (error) {
    throw new CliUserError(`failed to send setGlobalConfig transaction: ${String(error)}`);
  }
}
