import type { Address, Hex } from 'viem';
export interface RehearsalRequest { chainId: number; from?: Address; to: Address; value: string; data: Hex; blockNumber?: bigint }
export type Severity = 'critical' | 'high' | 'medium' | 'low' | 'info';
export interface Finding { severity: Severity; title: string; detail: string }
export interface StateChange { address: Address; field: string; before: string; after: string; kind: 'balance' | 'permission' | 'approval' | 'storage' | 'nonce' }
export interface Call { address: Address; label: string; type: 'CALL' | 'DELEGATECALL'; function: string; value: string; gas: number; success: boolean; data: Hex; returnData: Hex; children: Call[] }
export interface Evidence { chainId: number; blockNumber: string | null; blockHash: Hex | null; payloadHash: Hex; preStateCommitment: Hex | null; postStateCommitment: Hex | null; timestamp: string; attestation: string | null; rpcProvenance: string | null }
export interface RehearsalResult { mode: 'demo' | 'live'; request: Omit<RehearsalRequest, 'blockNumber'> & { blockNumber?: string }; success: boolean; gas: number; calls: Call[]; changes: StateChange[]; findings: Finding[]; revert: string | null; evidence: Evidence }
export interface RehearsalClient { rehearse(request: RehearsalRequest): Promise<RehearsalResult> }
