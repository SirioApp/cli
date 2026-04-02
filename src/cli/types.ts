export type NetworkName = "testnet" | "mainnet";

export type GlobalOptions = {
  network: NetworkName;
  rpcUrl?: string;
  factory?: string;
  allowlist?: string;
  privateKey?: string;
  help: boolean;
};

export type SaleTarget = {
  sale?: string;
  projectId?: number;
};

export type FactoryCommand =
  | { kind: "info" }
  | { kind: "global" }
  | { kind: "collateral"; collateral: string }
  | { kind: "list"; from: number; limit: number }
  | { kind: "project"; projectId: number }
  | { kind: "snapshot"; projectId: number }
  | { kind: "commitment"; projectId: number; user: string }
  | { kind: "agent-projects"; agentId: number }
  | {
      kind: "create";
      agentId: number;
      name: string;
      description: string;
      categories: string;
      tokenName: string;
      tokenSymbol: string;
      durationMinutes: number;
      lockupMinutes: number;
      launchInMinutes: number;
      agentAddress?: string;
      collateral?: string;
    }
  | { kind: "approve"; projectId: number }
  | { kind: "revoke"; projectId: number }
  | { kind: "update-metadata"; projectId: number; description: string; categories: string }
  | {
      kind: "set-status";
      projectId: number;
      status: "raising" | "deploying" | "operating" | "paused" | "closed";
      statusNote: string;
    }
  | { kind: "set-collateral"; collateral: string; allowed: boolean }
  | {
      kind: "set-global";
      minRaise: string;
      maxRaise: string;
      platformFeeBps: number;
      platformFeeRecipient: string;
    };

export type SaleCommand =
  | { kind: "status"; target: SaleTarget }
  | { kind: "claimable"; target: SaleTarget; user: string }
  | { kind: "refundable"; target: SaleTarget; user: string }
  | { kind: "commitment"; target: SaleTarget; user: string }
  | { kind: "approve-collateral"; target: SaleTarget; amount: string; raw: boolean }
  | { kind: "commit"; target: SaleTarget; amount: string; raw: boolean }
  | { kind: "finalize"; target: SaleTarget }
  | { kind: "claim"; target: SaleTarget }
  | { kind: "refund"; target: SaleTarget }
  | { kind: "emergency-refund"; target: SaleTarget };

export type AllowlistCommand =
  | { kind: "info" }
  | { kind: "is-allowed"; target: string }
  | { kind: "add"; target: string }
  | { kind: "remove"; target: string }
  | { kind: "transfer-admin"; newAdmin: string };

export type Command =
  | { scope: "network" }
  | { scope: "factory"; command: FactoryCommand }
  | { scope: "sale"; command: SaleCommand }
  | { scope: "allowlist"; command: AllowlistCommand };

export type ParsedCli = {
  global: GlobalOptions;
  command: Command;
};
