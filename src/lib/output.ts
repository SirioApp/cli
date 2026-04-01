import type { TransactionReceipt } from "ethers";
import type { RuntimeConfig } from "../config/runtime.ts";
import type { ProjectView } from "../types/project.ts";
import { formatStatus, formatTokenAmount } from "./utils.ts";

export function printNetwork(config: RuntimeConfig): void {
  console.log(`network: ${config.network}`);
  console.log(`label: ${config.networkLabel}`);
  console.log(`chain_id: ${config.chainId}`);
  console.log(`rpc: ${config.rpcUrl}`);
  console.log(`factory: ${config.factory}`);
  console.log(`allowlist: ${config.allowlist}`);
  console.log(`default_collateral: ${config.defaultCollateral ?? "<none>"}`);
  console.log(`deployment_file: ${config.deploymentPath}`);
}

export function printReceipt(receipt: TransactionReceipt): void {
  console.log(`tx_hash: ${receipt.hash}`);
  console.log(`block: ${receipt.blockNumber ?? 0}`);
  console.log(`status: ${receipt.status ?? 0}`);
}

export function printProject(projectId: number, project: ProjectView, approved: boolean): void {
  console.log("---");
  console.log(`project_id: ${projectId}`);
  console.log(`name: ${project.name}`);
  console.log(`description: ${project.description}`);
  if (project.categories) {
    console.log(`categories: ${project.categories}`);
  }
  console.log(`agent_id: ${project.agentId.toString()}`);
  console.log(`agent: ${project.agent}`);
  console.log(`treasury: ${project.treasury}`);
  console.log(`sale: ${project.sale}`);
  console.log(`agent_executor: ${project.agentExecutor}`);
  console.log(`collateral: ${project.collateral}`);
  console.log(
    `operational_status: ${project.operationalStatus} (${formatStatus(project.operationalStatus)})`,
  );
  if (project.statusNote) {
    console.log(`status_note: ${project.statusNote}`);
  }
  console.log(`approved: ${approved}`);
  console.log(`created_at: ${project.createdAt.toString()}`);
  console.log(`updated_at: ${project.updatedAt.toString()}`);
}

export function printGlobalConfig(config: {
  minRaise: bigint;
  maxRaise: bigint;
  feeBps: number;
  feeRecipient: string;
  minDuration: bigint;
  maxDuration: bigint;
  minLaunchDelay: bigint;
  maxLaunchDelay: bigint;
}): void {
  console.log("global_config:");
  console.log(`  min_raise_18: ${formatTokenAmount(config.minRaise, 18)} [raw ${config.minRaise}]`);
  console.log(`  max_raise_18: ${formatTokenAmount(config.maxRaise, 18)} [raw ${config.maxRaise}]`);
  console.log(`  platform_fee_bps: ${config.feeBps}`);
  console.log(`  fee_recipient: ${config.feeRecipient}`);
  console.log(`  min_duration_seconds: ${config.minDuration}`);
  console.log(`  max_duration_seconds: ${config.maxDuration}`);
  console.log(`  min_launch_delay_seconds: ${config.minLaunchDelay}`);
  console.log(`  max_launch_delay_seconds: ${config.maxLaunchDelay}`);
}
