import { renderHelp } from "./help.ts";
import type {
  AllowlistCommand,
  Command,
  FactoryCommand,
  GlobalOptions,
  ParsedCli,
  SaleCommand,
  SaleTarget,
} from "./types.ts";

export class CliUserError extends Error {
  readonly showHelp: boolean;

  constructor(message: string, showHelp = false) {
    super(message);
    this.name = "CliUserError";
    this.showHelp = showHelp;
  }
}

export function parseCli(argv: string[]): ParsedCli {
  const cursor = new Cursor(argv);
  const global = parseGlobalOptions(cursor);

  if (global.help && cursor.done()) {
    throw new CliUserError(renderHelp(), false);
  }

  const scope = cursor.take();
  if (!scope) {
    throw new CliUserError(renderHelp(), false);
  }

  const command = parseCommand(scope, cursor);

  if (!cursor.done()) {
    throw new CliUserError(`unexpected argument: ${cursor.take()}`, true);
  }

  return { global, command };
}

class Cursor {
  readonly tokens: string[];
  index = 0;

  constructor(tokens: string[]) {
    this.tokens = tokens;
  }

  peek(): string | undefined {
    return this.tokens[this.index];
  }

  take(): string | undefined {
    const value = this.tokens[this.index];
    if (value !== undefined) {
      this.index += 1;
    }
    return value;
  }

  done(): boolean {
    return this.index >= this.tokens.length;
  }
}

function parseGlobalOptions(cursor: Cursor): GlobalOptions {
  const global: GlobalOptions = {
    network: "testnet",
    help: false,
  };

  while (cursor.peek()?.startsWith("--")) {
    const flag = cursor.peek();
    if (
      flag !== "--network" &&
      flag !== "--rpc-url" &&
      flag !== "--factory" &&
      flag !== "--allowlist" &&
      flag !== "--private-key" &&
      flag !== "--help"
    ) {
      break;
    }

    cursor.take();

    switch (flag) {
      case "--network":
        global.network = expectOneOf(cursor, ["testnet", "mainnet"], "--network");
        break;
      case "--rpc-url":
        global.rpcUrl = expectValue(cursor, "--rpc-url");
        break;
      case "--factory":
        global.factory = expectValue(cursor, "--factory");
        break;
      case "--allowlist":
        global.allowlist = expectValue(cursor, "--allowlist");
        break;
      case "--private-key":
        global.privateKey = expectValue(cursor, "--private-key");
        break;
      case "--help":
        global.help = true;
        break;
      default:
        break;
    }
  }

  return global;
}

function parseCommand(scope: string, cursor: Cursor): Command {
  switch (scope) {
    case "network":
      return { scope: "network" };
    case "factory":
      return { scope: "factory", command: parseFactoryCommand(cursor) };
    case "sale":
      return { scope: "sale", command: parseSaleCommand(cursor) };
    case "allowlist":
      return { scope: "allowlist", command: parseAllowlistCommand(cursor) };
    default:
      throw new CliUserError(`unknown command: ${scope}`, true);
  }
}

function parseFactoryCommand(cursor: Cursor): FactoryCommand {
  const subcommand = cursor.take();
  switch (subcommand) {
    case "info":
      return { kind: "info" };
    case "global":
      return { kind: "global" };
    case "collateral":
      return { kind: "collateral", collateral: expectPositional(cursor, "collateral") };
    case "list": {
      const options = parseNamedOptions(cursor);
      return {
        kind: "list",
        from: readIntOption(options, "--from", 0),
        limit: readIntOption(options, "--limit", 10),
      };
    }
    case "project":
      return { kind: "project", projectId: parseIntArg(expectPositional(cursor, "project-id"), "project-id") };
    case "snapshot":
      return { kind: "snapshot", projectId: parseIntArg(expectPositional(cursor, "project-id"), "project-id") };
    case "commitment":
      return {
        kind: "commitment",
        projectId: parseIntArg(expectPositional(cursor, "project-id"), "project-id"),
        user: expectPositional(cursor, "user"),
      };
    case "agent-projects":
      return { kind: "agent-projects", agentId: parseIntArg(expectPositional(cursor, "agent-id"), "agent-id") };
    case "create": {
      const options = parseNamedOptions(cursor);
      return {
        kind: "create",
        agentId: readRequiredIntOption(options, "--agent-id"),
        name: readRequiredOption(options, "--name"),
        description: readRequiredOption(options, "--description"),
        categories: readOptionalOption(options, "--categories", ""),
        tokenName: readRequiredOption(options, "--token-name"),
        tokenSymbol: readRequiredOption(options, "--token-symbol"),
        durationMinutes: readRequiredIntOption(options, "--duration-minutes"),
        lockupMinutes: readIntOption(
          options,
          "--lockup-minutes",
          readIntOption(options, "--redeem-delay-minutes", 0),
        ),
        launchInMinutes: readIntOption(options, "--launch-in-minutes", 0),
        agentAddress: readOptionalOption(options, "--agent-address"),
        collateral: readOptionalOption(options, "--collateral"),
      };
    }
    case "approve":
      return { kind: "approve", projectId: parseIntArg(expectPositional(cursor, "project-id"), "project-id") };
    case "revoke":
      return { kind: "revoke", projectId: parseIntArg(expectPositional(cursor, "project-id"), "project-id") };
    case "update-metadata": {
      const options = parseNamedOptions(cursor);
      return {
        kind: "update-metadata",
        projectId: readRequiredIntOption(options, "--project-id"),
        description: readRequiredOption(options, "--description"),
        categories: readOptionalOption(options, "--categories", ""),
      };
    }
    case "set-status": {
      const options = parseNamedOptions(cursor);
      return {
        kind: "set-status",
        projectId: readRequiredIntOption(options, "--project-id"),
        status: readRequiredOneOfOption(options, "--status", [
          "raising",
          "deploying",
          "operating",
          "paused",
          "closed",
        ]),
        statusNote: readOptionalOption(options, "--status-note", ""),
      };
    }
    case "set-collateral":
      return {
        kind: "set-collateral",
        collateral: expectPositional(cursor, "collateral"),
        allowed: parseBoolean(expectPositional(cursor, "allowed")),
      };
    case "set-global": {
      const options = parseNamedOptions(cursor);
      return {
        kind: "set-global",
        minRaise: readRequiredOption(options, "--min-raise"),
        maxRaise: readRequiredOption(options, "--max-raise"),
        platformFeeBps: readRequiredIntOption(options, "--platform-fee-bps"),
        platformFeeRecipient: readRequiredOption(options, "--platform-fee-recipient"),
      };
    }
    default:
      throw new CliUserError("missing or invalid factory subcommand", true);
  }
}

function parseSaleCommand(cursor: Cursor): SaleCommand {
  const subcommand = cursor.take();
  switch (subcommand) {
    case "status":
      return { kind: "status", target: parseSaleTarget(cursor) };
    case "claimable":
      return { kind: "claimable", target: parseSaleTarget(cursor), user: expectPositional(cursor, "user") };
    case "refundable":
      return { kind: "refundable", target: parseSaleTarget(cursor), user: expectPositional(cursor, "user") };
    case "commitment":
      return { kind: "commitment", target: parseSaleTarget(cursor), user: expectPositional(cursor, "user") };
    case "approve-collateral":
      return {
        kind: "approve-collateral",
        target: parseSaleTarget(cursor),
        amount: expectPositional(cursor, "amount"),
        raw: readBooleanFlag(cursor, "--raw"),
      };
    case "commit":
      return {
        kind: "commit",
        target: parseSaleTarget(cursor),
        amount: expectPositional(cursor, "amount"),
        raw: readBooleanFlag(cursor, "--raw"),
      };
    case "finalize":
      return { kind: "finalize", target: parseSaleTarget(cursor) };
    case "claim":
      return { kind: "claim", target: parseSaleTarget(cursor) };
    case "refund":
      return { kind: "refund", target: parseSaleTarget(cursor) };
    case "emergency-refund":
      return { kind: "emergency-refund", target: parseSaleTarget(cursor) };
    default:
      throw new CliUserError("missing or invalid sale subcommand", true);
  }
}

function parseAllowlistCommand(cursor: Cursor): AllowlistCommand {
  const subcommand = cursor.take();
  switch (subcommand) {
    case "info":
      return { kind: "info" };
    case "is-allowed":
      return { kind: "is-allowed", target: expectPositional(cursor, "target") };
    case "add":
      return { kind: "add", target: expectPositional(cursor, "target") };
    case "remove":
      return { kind: "remove", target: expectPositional(cursor, "target") };
    case "transfer-admin":
      return { kind: "transfer-admin", newAdmin: expectPositional(cursor, "new-admin") };
    default:
      throw new CliUserError("missing or invalid allowlist subcommand", true);
  }
}

function parseSaleTarget(cursor: Cursor): SaleTarget {
  const target: SaleTarget = {};

  while (cursor.peek()?.startsWith("--")) {
    const flag = cursor.take();
    switch (flag) {
      case "--sale":
        target.sale = expectValue(cursor, "--sale");
        break;
      case "--project-id":
        target.projectId = parseIntArg(expectValue(cursor, "--project-id"), "--project-id");
        break;
      case "--help":
        throw new CliUserError(renderHelp(), false);
      default:
        cursor.index -= 1;
        break;
    }

    if (flag !== "--sale" && flag !== "--project-id" && flag !== "--help") {
      break;
    }
  }

  if (!target.sale && target.projectId === undefined) {
    throw new CliUserError("sale target required: use --sale or --project-id", true);
  }
  if (target.sale && target.projectId !== undefined) {
    throw new CliUserError("use either --sale or --project-id, not both", true);
  }

  return target;
}

function readBooleanFlag(cursor: Cursor, flag: string): boolean {
  const options = parseNamedOptions(cursor);
  return options.has(flag);
}

function parseNamedOptions(cursor: Cursor): Map<string, string> {
  const options = new Map<string, string>();
  const booleans = new Set<string>(["--raw"]);

  while (cursor.peek()?.startsWith("--")) {
    const flag = cursor.take()!;
    if (booleans.has(flag)) {
      options.set(flag, "true");
      continue;
    }
    options.set(flag, expectValue(cursor, flag));
  }

  return options;
}

function readRequiredOption(options: Map<string, string>, flag: string): string {
  const value = options.get(flag);
  if (!value) {
    throw new CliUserError(`missing required option: ${flag}`, true);
  }
  return value;
}

function readOptionalOption(
  options: Map<string, string>,
  flag: string,
  fallback?: string,
): string | undefined {
  return options.get(flag) ?? fallback;
}

function readRequiredIntOption(options: Map<string, string>, flag: string): number {
  return parseIntArg(readRequiredOption(options, flag), flag);
}

function readIntOption(options: Map<string, string>, flag: string, fallback: number): number {
  const value = options.get(flag);
  return value === undefined ? fallback : parseIntArg(value, flag);
}

function readRequiredOneOfOption<T extends string>(
  options: Map<string, string>,
  flag: string,
  values: readonly T[],
): T {
  return expectOneOfValue(readRequiredOption(options, flag), values, flag);
}

function expectPositional(cursor: Cursor, label: string): string {
  const value = cursor.take();
  if (!value || value.startsWith("--")) {
    throw new CliUserError(`missing required argument: ${label}`, true);
  }
  return value;
}

function expectValue(cursor: Cursor, flag: string): string {
  const value = cursor.take();
  if (!value || value.startsWith("--")) {
    throw new CliUserError(`missing value for ${flag}`, true);
  }
  return value;
}

function expectOneOf(cursor: Cursor, values: readonly string[], flag: string): string {
  return expectOneOfValue(expectValue(cursor, flag), values, flag);
}

function expectOneOfValue<T extends string>(value: string, values: readonly T[], flag: string): T {
  if (!values.includes(value as T)) {
    throw new CliUserError(`invalid value for ${flag}: ${value}`, true);
  }
  return value as T;
}

function parseBoolean(value: string): boolean {
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  throw new CliUserError(`invalid boolean value: ${value}`, true);
}

function parseIntArg(value: string, label: string): number {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isInteger(parsed) || parsed < 0) {
    throw new CliUserError(`invalid integer for ${label}: ${value}`, true);
  }
  return parsed;
}
