import type { FactoryCommand, GlobalOptions } from "../cli/types.ts";
import type { RuntimeConfig } from "../config/runtime.ts";
import {
  createFactoryContract,
  createReadClient,
  createWriteClient,
  normalizeAddress,
} from "../chain/client.ts";
import {
  readFactoryProject,
  readTokenMetadata,
  sendFactoryGlobalConfig,
} from "../chain/contracts.ts";
import { printGlobalConfig, printProject, printReceipt } from "../lib/output.ts";
import {
  minutesToSeconds,
  parseAmountUnits,
  sendAndWait,
  unixNow,
} from "../lib/utils.ts";

const STATUS_CODES: Record<string, number> = {
  raising: 0,
  deploying: 1,
  operating: 2,
  paused: 3,
  closed: 4,
};

export async function runFactory(
  global: GlobalOptions,
  config: RuntimeConfig,
  command: FactoryCommand,
): Promise<void> {
  const readClient = createReadClient(config);
  const factory = createFactoryContract(config.factory, readClient);

  switch (command.kind) {
    case "info": {
      console.log(`factory: ${config.factory}`);
      console.log(`projects_total: ${String(await factory.projectCount())}`);
      printGlobal(await factory.globalConfig());
      await printDefaultCollateral(config, readClient, factory);
      return;
    }
    case "global": {
      printGlobal(await factory.globalConfig());
      return;
    }
    case "collateral": {
      const collateral = normalizeAddress(command.collateral, "collateral");
      const allowed = Boolean(await factory.allowedCollateral(collateral));
      console.log(`collateral: ${collateral}`);
      console.log(`allowed: ${allowed}`);
      if (allowed) {
        const metadata = await readTokenMetadata(collateral, readClient);
        console.log(`symbol: ${metadata.symbol}`);
        console.log(
          `min_raise: ${String(await factory.minRaiseForCollateral(collateral))}`,
        );
        console.log(
          `max_raise: ${String(await factory.maxRaiseForCollateral(collateral))}`,
        );
      }
      return;
    }
    case "list": {
      const total = Number(await factory.projectCount());
      const start = Math.min(command.from, total);
      const end = Math.min(start + command.limit, total);
      console.log(`projects_total: ${total}`);
      console.log(`range: ${start}..${end}`);
      for (let projectId = start; projectId < end; projectId += 1) {
        const project = await readFactoryProject(config.factory, readClient, projectId);
        const approved = Boolean(await factory.isProjectApproved(BigInt(projectId)));
        printProject(projectId, project, approved);
      }
      return;
    }
    case "project": {
      const project = await readFactoryProject(config.factory, readClient, command.projectId);
      const approved = Boolean(await factory.isProjectApproved(BigInt(command.projectId)));
      printProject(command.projectId, project, approved);
      return;
    }
    case "snapshot": {
      const snapshot = (await factory.getProjectRaiseSnapshot(BigInt(command.projectId))) as readonly [
        boolean,
        bigint,
        bigint,
        boolean,
        boolean,
        boolean,
        bigint,
        bigint,
        string,
      ];
      console.log(`project_id: ${command.projectId}`);
      console.log(`approved: ${snapshot[0]}`);
      console.log(`total_committed: ${snapshot[1]}`);
      console.log(`accepted_amount: ${snapshot[2]}`);
      console.log(`finalized: ${snapshot[3]}`);
      console.log(`failed: ${snapshot[4]}`);
      console.log(`active: ${snapshot[5]}`);
      console.log(`start_time: ${snapshot[6]}`);
      console.log(`end_time: ${snapshot[7]}`);
      console.log(`share_token: ${snapshot[8]}`);
      return;
    }
    case "commitment": {
      console.log(`project_id: ${command.projectId}`);
      console.log(`user: ${normalizeAddress(command.user, "user")}`);
      console.log(
        `committed: ${String(
          await factory.getProjectCommitment(
            BigInt(command.projectId),
            normalizeAddress(command.user, "user"),
          ),
        )}`,
      );
      return;
    }
    case "agent-projects": {
      console.log(`agent_id: ${command.agentId}`);
      console.log(`project_ids: ${JSON.stringify(await factory.getAgentProjects(BigInt(command.agentId)))}`);
      return;
    }
    case "create": {
      const writeClient = createWriteClient(global, config);
      const writeFactory = createFactoryContract(config.factory, writeClient);
      const collateral = command.collateral
        ? normalizeAddress(command.collateral, "collateral")
        : config.defaultCollateral;
      if (!collateral) {
        throw new Error("collateral not set: pass --collateral or configure USDM in deployment file");
      }
      const launchTimestamp =
        unixNow() +
        Number(minutesToSeconds(command.launchInMinutes, "launch-in-minutes"));
      const transaction = writeFactory.createAgentRaise(
        BigInt(command.agentId),
        command.name,
        command.description,
        command.categories,
        normalizeAddress(command.agentAddress ?? writeClient.address, "agent-address"),
        collateral,
        minutesToSeconds(command.durationMinutes, "duration-minutes"),
        BigInt(launchTimestamp),
        command.tokenName,
        command.tokenSymbol,
      );
      printReceipt(await sendAndWait(transaction));
      return;
    }
    case "approve": {
      const writeFactory = createFactoryContract(config.factory, createWriteClient(global, config));
      printReceipt(await sendAndWait(writeFactory.approveProject(BigInt(command.projectId))));
      return;
    }
    case "revoke": {
      const writeFactory = createFactoryContract(config.factory, createWriteClient(global, config));
      printReceipt(await sendAndWait(writeFactory.revokeProject(BigInt(command.projectId))));
      return;
    }
    case "update-metadata": {
      const writeFactory = createFactoryContract(config.factory, createWriteClient(global, config));
      printReceipt(
        await sendAndWait(
          writeFactory.updateProjectMetadata(
            BigInt(command.projectId),
            command.description,
            command.categories,
          ),
        ),
      );
      return;
    }
    case "set-status": {
      const writeFactory = createFactoryContract(config.factory, createWriteClient(global, config));
      printReceipt(
        await sendAndWait(
          writeFactory.updateProjectOperationalStatus(
            BigInt(command.projectId),
            STATUS_CODES[command.status],
            command.statusNote,
          ),
        ),
      );
      return;
    }
    case "set-collateral": {
      const writeFactory = createFactoryContract(config.factory, createWriteClient(global, config));
      printReceipt(
        await sendAndWait(
          writeFactory.setAllowedCollateral(
            normalizeAddress(command.collateral, "collateral"),
            command.allowed,
          ),
        ),
      );
      return;
    }
    case "set-global": {
      const writeClient = createWriteClient(global, config);
      printReceipt(
        await sendAndWait(
          sendFactoryGlobalConfig(config.factory, writeClient, [
            parseAmountUnits(command.minRaise, 18),
            parseAmountUnits(command.maxRaise, 18),
            command.platformFeeBps,
            normalizeAddress(command.platformFeeRecipient, "platform-fee-recipient"),
            minutesToSeconds(command.minDurationMinutes, "min-duration-minutes"),
            minutesToSeconds(command.maxDurationMinutes, "max-duration-minutes"),
            minutesToSeconds(command.minLaunchDelayMinutes, "min-launch-delay-minutes"),
            minutesToSeconds(command.maxLaunchDelayMinutes, "max-launch-delay-minutes"),
          ]),
        ),
      );
      return;
    }
  }
}

function printGlobal(tuple: readonly [bigint, bigint, number, string, bigint, bigint, bigint, bigint]): void {
  printGlobalConfig({
    minRaise: tuple[0],
    maxRaise: tuple[1],
    feeBps: tuple[2],
    feeRecipient: tuple[3],
    minDuration: tuple[4],
    maxDuration: tuple[5],
    minLaunchDelay: tuple[6],
    maxLaunchDelay: tuple[7],
  });
}

async function printDefaultCollateral(
  config: RuntimeConfig,
  readClient: ReturnType<typeof createReadClient>,
  factory: ReturnType<typeof createFactoryContract>,
): Promise<void> {
  if (!config.defaultCollateral) {
    return;
  }

  const collateral = config.defaultCollateral;
  try {
    const allowed = Boolean(await factory.allowedCollateral(collateral));
    if (!allowed) {
      console.log(`default_collateral_allowed: false`);
      return;
    }
    const metadata = await readTokenMetadata(collateral, readClient);
    console.log(`default_collateral: ${collateral}`);
    console.log(`default_collateral_symbol: ${metadata.symbol}`);
    console.log(
      `default_collateral_min_raise: ${String(await factory.minRaiseForCollateral(collateral))}`,
    );
    console.log(
      `default_collateral_max_raise: ${String(await factory.maxRaiseForCollateral(collateral))}`,
    );
  } catch {
    console.log("default_collateral: unavailable");
  }
}
