import {
  formatUnits,
  parseUnits,
  type ContractTransactionResponse,
  type TransactionReceipt,
} from "ethers";
import { CliUserError } from "../cli/parser.ts";

export function parseAmountUnits(value: string, decimals: number): bigint {
  try {
    return parseUnits(value, decimals);
  } catch {
    throw new CliUserError(`invalid amount \`${value}\` for ${decimals} decimals`);
  }
}

export function parseAmountRaw(value: string): bigint {
  try {
    return BigInt(value);
  } catch {
    throw new CliUserError(`invalid raw uint256 amount \`${value}\``);
  }
}

export function formatTokenAmount(value: bigint, decimals: number): string {
  try {
    return formatUnits(value, decimals);
  } catch {
    return value.toString();
  }
}

export function minutesToSeconds(minutes: number, label: string): bigint {
  if (!Number.isSafeInteger(minutes) || minutes < 0) {
    throw new CliUserError(`invalid integer for ${label}: ${minutes}`);
  }
  return BigInt(minutes * 60);
}

export function unixNow(): number {
  return Math.floor(Date.now() / 1000);
}

export async function sendAndWait(
  transactionPromise: Promise<ContractTransactionResponse>,
): Promise<TransactionReceipt> {
  const transaction = await transactionPromise;
  const receipt = await transaction.wait();
  if (!receipt) {
    throw new CliUserError("transaction dropped before inclusion");
  }
  return receipt;
}

export function formatBps(bps: number): string {
  return `${bps} (${(bps / 100).toFixed(2)}%)`;
}

export function formatDurationHuman(seconds: bigint): string {
  if (seconds > BigInt(Number.MAX_SAFE_INTEGER)) {
    return `${seconds.toString()} sec`;
  }

  let remaining = Number(seconds);
  const days = Math.floor(remaining / 86400);
  remaining %= 86400;
  const hours = Math.floor(remaining / 3600);
  remaining %= 3600;
  const minutes = Math.floor(remaining / 60);
  const secs = remaining % 60;

  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);
  if (secs > 0 || parts.length === 0) parts.push(`${secs}s`);

  return `${seconds.toString()} sec (${parts.join(" ")})`;
}

export function formatStatus(code: number): string {
  switch (code) {
    case 0:
      return "raising";
    case 1:
      return "deploying";
    case 2:
      return "operating";
    case 3:
      return "paused";
    case 4:
      return "closed";
    default:
      return "unknown";
  }
}
