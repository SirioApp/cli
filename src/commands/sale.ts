import { type Contract, type ContractRunner, type ContractTransactionResponse } from "ethers";
import type { GlobalOptions, SaleCommand, SaleTarget } from "../cli/types.ts";
import type { RuntimeConfig } from "../config/runtime.ts";
import {
  createErc20Contract,
  createFactoryContract,
  createReadClient,
  createSaleContract,
  createShareTokenContract,
  createWriteClient,
  ensureSaleTarget,
  normalizeAddress,
} from "../chain/client.ts";
import {
  readFactoryProject,
  readSaleCollateral,
  readSaleProjectId,
  readTokenMetadata,
} from "../chain/contracts.ts";
import { printReceipt } from "../lib/output.ts";
import { formatTokenAmount, parseAmountRaw, parseAmountUnits, sendAndWait } from "../lib/utils.ts";

export async function runSale(
  global: GlobalOptions,
  config: RuntimeConfig,
  command: SaleCommand,
): Promise<void> {
  const readClient = createReadClient(config);

  switch (command.kind) {
    case "status":
      await showStatus(config, readClient, command.target);
      return;
    case "claimable":
      await showClaimable(config, readClient, command.target, command.user);
      return;
    case "refundable":
      await showRefundable(config, readClient, command.target, command.user);
      return;
    case "commitment":
      await showCommitment(config, readClient, command.target, command.user);
      return;
    case "approve-collateral":
      await approveCollateral(global, config, command.target, command.amount, command.raw);
      return;
    case "commit":
      await commit(global, config, command.target, command.amount, command.raw);
      return;
    case "finalize":
      await runSaleTransaction(global, config, command.target, (sale) => sale.finalize());
      return;
    case "claim":
      await runSaleTransaction(global, config, command.target, (sale) => sale.claim());
      return;
    case "refund":
      await runSaleTransaction(global, config, command.target, (sale) => sale.refund());
      return;
    case "emergency-refund":
      await runSaleTransaction(global, config, command.target, (sale) => sale.emergencyRefund());
      return;
  }
}

async function showStatus(
  config: RuntimeConfig,
  readClient: ReturnType<typeof createReadClient>,
  target: SaleTarget,
): Promise<void> {
  const saleAddress = await resolveSaleAddress(config, readClient, target);
  const sale = createSaleContract(saleAddress, readClient);
  const [totalCommitted, acceptedAmount, finalized, failed] = (await sale.getStatus()) as readonly [
    bigint,
    bigint,
    boolean,
    boolean,
  ];
  const collateral = await readSaleCollateral(saleAddress, readClient);
  const { decimals, symbol } = await readTokenMetadata(collateral, readClient);

  console.log(`sale: ${saleAddress}`);
  try {
    const projectId = await readSaleProjectId(saleAddress, readClient);
    console.log(`project_id: ${projectId}`);
    console.log(
      `project_approved: ${Boolean(
        await createFactoryContract(config.factory, readClient).isProjectApproved(projectId),
      )}`,
    );
  } catch {}
  console.log(`collateral: ${collateral} (${symbol})`);
  console.log(`start_time: ${String(await sale.startTime())}`);
  console.log(`end_time: ${String(await sale.endTime())}`);
  console.log(`is_active: ${Boolean(await sale.isActive())}`);
  console.log(`time_remaining_seconds: ${String(await sale.timeRemaining())}`);
  console.log(
    `total_committed: ${formatTokenAmount(totalCommitted, decimals)} (${totalCommitted})`,
  );
  console.log(
    `accepted_amount: ${formatTokenAmount(acceptedAmount, decimals)} (${acceptedAmount})`,
  );
  console.log(`finalized: ${finalized}`);
  console.log(`failed: ${failed}`);
  const shareTokenAddress = String(await sale.token());
  console.log(`share_token: ${shareTokenAddress}`);
  if (shareTokenAddress !== "0x0000000000000000000000000000000000000000") {
    const shareToken = createShareTokenContract(shareTokenAddress, readClient);
    try {
      const poolAssets = (await shareToken.totalAssets()) as bigint;
      console.log(`fund_term_end: ${String(await shareToken.LOCKUP_END_TIME())}`);
      console.log(`settled: ${Boolean(await shareToken.settled())}`);
      console.log(`redeem_pool: ${formatTokenAmount(poolAssets, decimals)} (${poolAssets})`);
    } catch {}
  }
}

async function showClaimable(
  config: RuntimeConfig,
  readClient: ReturnType<typeof createReadClient>,
  target: SaleTarget,
  user: string,
): Promise<void> {
  const saleAddress = await resolveSaleAddress(config, readClient, target);
  const sale = createSaleContract(saleAddress, readClient);
  const normalizedUser = normalizeAddress(user, "user");
  const collateral = await readSaleCollateral(saleAddress, readClient);
  const { decimals: collateralDecimals, symbol: collateralSymbol } =
    await readTokenMetadata(collateral, readClient);
  const [payout, refund] = (await sale.getClaimable(normalizedUser)) as readonly [bigint, bigint];
  const shareTokenAddress = String(await sale.token());
  const hasShareToken = shareTokenAddress !== "0x0000000000000000000000000000000000000000";
  const { decimals: shareDecimals, symbol: shareSymbol } = hasShareToken
    ? await readTokenMetadata(shareTokenAddress, readClient)
    : { decimals: 18, symbol: "SHARE" };

  console.log(`sale: ${saleAddress}`);
  console.log(`user: ${normalizedUser}`);
  console.log(`share_token: ${shareSymbol}`);
  console.log(
    `claimable_shares: ${formatTokenAmount(payout, shareDecimals)} ${shareSymbol} (${payout})`,
  );
  console.log(
    `overflow_refund: ${formatTokenAmount(refund, collateralDecimals)} ${collateralSymbol} (${refund})`,
  );
}

async function showRefundable(
  config: RuntimeConfig,
  readClient: ReturnType<typeof createReadClient>,
  target: SaleTarget,
  user: string,
): Promise<void> {
  const saleAddress = await resolveSaleAddress(config, readClient, target);
  const sale = createSaleContract(saleAddress, readClient);
  const normalizedUser = normalizeAddress(user, "user");
  const collateral = await readSaleCollateral(saleAddress, readClient);
  const { decimals, symbol } = await readTokenMetadata(collateral, readClient);
  const refundable = (await sale.getRefundable(normalizedUser)) as bigint;

  console.log(`sale: ${saleAddress}`);
  console.log(`user: ${normalizedUser}`);
  console.log(`token: ${symbol}`);
  console.log(`refundable: ${formatTokenAmount(refundable, decimals)} (${refundable})`);
}

async function showCommitment(
  config: RuntimeConfig,
  readClient: ReturnType<typeof createReadClient>,
  target: SaleTarget,
  user: string,
): Promise<void> {
  const saleAddress = await resolveSaleAddress(config, readClient, target);
  const sale = createSaleContract(saleAddress, readClient);
  const normalizedUser = normalizeAddress(user, "user");
  const collateral = await readSaleCollateral(saleAddress, readClient);
  const { decimals, symbol } = await readTokenMetadata(collateral, readClient);
  const committed = (await sale.commitments(normalizedUser)) as bigint;

  console.log(`sale: ${saleAddress}`);
  console.log(`user: ${normalizedUser}`);
  console.log(`token: ${symbol}`);
  console.log(`committed: ${formatTokenAmount(committed, decimals)} (${committed})`);
}

async function approveCollateral(
  global: GlobalOptions,
  config: RuntimeConfig,
  target: SaleTarget,
  amount: string,
  raw: boolean,
): Promise<void> {
  const writeClient = createWriteClient(global, config);
  const saleAddress = await resolveSaleAddress(config, writeClient, target);
  const collateral = await readSaleCollateral(saleAddress, writeClient);
  const token = createErc20Contract(collateral, writeClient);
  const { decimals, symbol } = await readTokenMetadata(collateral, writeClient);
  const value = raw ? parseAmountRaw(amount) : parseAmountUnits(amount, decimals);

  const receipt = await sendAndWait(token.approve(saleAddress, value));
  console.log(`owner: ${writeClient.address}`);
  console.log(`approved: ${formatTokenAmount(value, decimals)} ${symbol} (${value})`);
  printReceipt(receipt);
}

async function commit(
  global: GlobalOptions,
  config: RuntimeConfig,
  target: SaleTarget,
  amount: string,
  raw: boolean,
): Promise<void> {
  const writeClient = createWriteClient(global, config);
  const saleAddress = await resolveSaleAddress(config, writeClient, target);
  const sale = createSaleContract(saleAddress, writeClient);
  const collateral = await readSaleCollateral(saleAddress, writeClient);
  const token = createErc20Contract(collateral, writeClient);
  const { decimals, symbol } = await readTokenMetadata(collateral, writeClient);
  const value = raw ? parseAmountRaw(amount) : parseAmountUnits(amount, decimals);
  const owner = writeClient.address;
  const allowance = (await token.allowance(owner, saleAddress)) as bigint;
  const balance = (await token.balanceOf(owner)) as bigint;

  if (allowance < value) {
    throw new Error(
      `insufficient allowance: allowance=${formatTokenAmount(allowance, decimals)} required=${formatTokenAmount(value, decimals)}`,
    );
  }
  if (balance < value) {
    throw new Error(
      `insufficient balance: balance=${formatTokenAmount(balance, decimals)} required=${formatTokenAmount(value, decimals)}`,
    );
  }

  const receipt = await sendAndWait(sale.commit(value));
  console.log(`sender: ${owner}`);
  console.log(`committed: ${formatTokenAmount(value, decimals)} ${symbol} (${value})`);
  printReceipt(receipt);
}

async function runSaleTransaction(
  global: GlobalOptions,
  config: RuntimeConfig,
  target: SaleTarget,
  build: (sale: Contract) => Promise<ContractTransactionResponse>,
): Promise<void> {
  const writeClient = createWriteClient(global, config);
  const saleAddress = await resolveSaleAddress(config, writeClient, target);
  const sale = createSaleContract(saleAddress, writeClient);
  printReceipt(await sendAndWait(build(sale)));
}

async function resolveSaleAddress(
  config: RuntimeConfig,
  runner: ContractRunner,
  target: SaleTarget,
): Promise<string> {
  const normalizedTarget = ensureSaleTarget(target);
  if (normalizedTarget.sale) {
    return normalizeAddress(normalizedTarget.sale, "sale");
  }
  return (await readFactoryProject(config.factory, runner, normalizedTarget.projectId!)).sale;
}
