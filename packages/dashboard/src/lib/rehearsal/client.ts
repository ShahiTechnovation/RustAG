import { keccak256, toHex, type Address } from 'viem';
import type { RehearsalClient, RehearsalRequest, RehearsalResult } from './types';

export const demoAddresses = { safe: '0x1111111111111111111111111111111111111111', proxy: '0x2222222222222222222222222222222222222222', implementation: '0x3333333333333333333333333333333333333333', token: '0x4444444444444444444444444444444444444444' } as const;
export const demoRequest: RehearsalRequest = { chainId: 46630, from: demoAddresses.safe, to: demoAddresses.proxy, value: '0', data: '0x3659cfe6' };
// This fixture deliberately does not derive execution outcomes from user input.
// A future live client must implement RehearsalClient and validate its wire response.
export const demoClient: RehearsalClient = { async rehearse(request) {
  const payload = { ...request, blockNumber: request.blockNumber?.toString() };
  const call = (address: Address, label: string, type: 'CALL' | 'DELEGATECALL', fn: string, gas: number) => ({ address, label, type, function: fn, value: '0 ETH', gas, success: true, data: '0x' as const, returnData: '0x' as const, children: [] });
  return {
    mode: 'demo', request: payload, success: true, gas: 148392,
    calls: [{ ...call(demoAddresses.safe, 'Treasury Safe', 'CALL', 'execTransaction(…)', 148392), children: [{ ...call(demoAddresses.proxy, 'Treasury proxy', 'CALL', 'upgradeToAndCall(…)', 112640), children: [{ ...call(demoAddresses.implementation, 'Implementation v2', 'DELEGATECALL', 'initialize(…)', 42810), children: [call(demoAddresses.token, 'Treasury token', 'CALL', 'approve(…)', 22400)] }] }] }],
    changes: [
      { address: demoAddresses.proxy, field: 'Implementation', before: '0x5555555555555555555555555555555555555555', after: demoAddresses.implementation, kind: 'storage' },
      { address: demoAddresses.proxy, field: 'Owner', before: demoAddresses.safe, after: '0x6666666666666666666666666666666666666666', kind: 'permission' },
      { address: demoAddresses.token, field: 'ERC-20 allowance', before: '0 TOKEN', after: 'Unlimited TOKEN', kind: 'approval' },
      { address: demoAddresses.safe, field: 'ETH balance', before: '24.8000 ETH', after: '24.7997 ETH', kind: 'balance' },
      { address: demoAddresses.safe, field: 'Token balance', before: '125,000 TOKEN', after: '120,000 TOKEN', kind: 'balance' },
      { address: demoAddresses.safe, field: 'Nonce', before: '41', after: '42', kind: 'nonce' },
    ],
    findings: [{ severity: 'high', title: 'Unlimited token approval', detail: 'Illustrative finding: the spender can move the entire token balance. Review the spender and consider a bounded allowance.' }, { severity: 'medium', title: 'Upgrade authority changes', detail: 'Illustrative finding: ownership moves to a different executor after the proxy upgrade.' }, { severity: 'info', title: 'Delegatecall in execution path', detail: 'Illustrative finding: implementation code executes in the proxy storage context.' }],
    revert: null,
    evidence: { chainId: request.chainId, blockNumber: null, blockHash: null, payloadHash: keccak256(toHex(JSON.stringify(payload))), preStateCommitment: null, postStateCommitment: null, timestamp: new Date().toISOString(), attestation: null, rpcProvenance: null },
  };
} };
export const rehearsalClient: RehearsalClient = demoClient;
export function downloadEvidence(result: RehearsalResult) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(result, null, 2)], { type: 'application/json' }));
  const a = document.createElement('a'); a.href = url; a.download = 'rustag-demo-evidence.json'; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
}
