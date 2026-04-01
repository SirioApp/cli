export const FACTORY_ABI = [
  "function globalConfig() view returns (uint256,uint256,uint16,address,uint256,uint256,uint256,uint256)",
  "function projectCount() view returns (uint256)",
  "function getAgentProjects(uint256 agentId) view returns (uint256[])",
  "function isProjectApproved(uint256 id) view returns (bool)",
  "function allowedCollateral(address collateral) view returns (bool)",
  "function minRaiseForCollateral(address collateral) view returns (uint256)",
  "function maxRaiseForCollateral(address collateral) view returns (uint256)",
  "function createAgentRaise(uint256,string,string,string,address,address,uint256,uint256,string,string) returns (uint256)",
  "function approveProject(uint256 projectId)",
  "function revokeProject(uint256 projectId)",
  "function setAllowedCollateral(address collateral, bool allowed)",
  "function updateProjectMetadata(uint256 projectId, string description, string categories)",
  "function updateProjectOperationalStatus(uint256 projectId, uint8 status, string statusNote)",
  "function getProjectRaiseSnapshot(uint256 projectId) view returns (bool,uint256,uint256,bool,bool,bool,uint256,uint256,address)",
  "function getProjectCommitment(uint256 projectId, address user) view returns (uint256)",
  "function getProject(uint256 id) view returns (uint256,string,string,string,address,address,address,address,address,uint8,string,uint256,uint256)",
] as const;

export const SALE_ABI = [
  "function startTime() view returns (uint256)",
  "function endTime() view returns (uint256)",
  "function getStatus() view returns (uint256,uint256,bool,bool)",
  "function getClaimable(address user) view returns (uint256,uint256)",
  "function getRefundable(address user) view returns (uint256)",
  "function isActive() view returns (bool)",
  "function timeRemaining() view returns (uint256)",
  "function token() view returns (address)",
  "function commitments(address user) view returns (uint256)",
  "function claim()",
  "function refund()",
  "function commit(uint256 amount)",
  "function finalize()",
  "function emergencyRefund()",
  "function COLLATERAL() view returns (address)",
  "function PROJECT_ID() view returns (uint256)",
] as const;

export const ERC20_ABI = [
  "function decimals() view returns (uint8)",
  "function symbol() view returns (string)",
  "function balanceOf(address owner) view returns (uint256)",
  "function allowance(address owner, address spender) view returns (uint256)",
  "function approve(address spender, uint256 amount) returns (bool)",
] as const;

export const ALLOWLIST_ABI = [
  "function admin() view returns (address)",
  "function isAllowed(address target) view returns (bool)",
  "function addContract(address target)",
  "function removeContract(address target)",
  "function transferAdmin(address newAdmin)",
] as const;

export const FACTORY_ADMIN_ABI = [
  "function setGlobalConfig((uint256,uint256,uint16,address,uint256,uint256,uint256,uint256) config)",
] as const;
