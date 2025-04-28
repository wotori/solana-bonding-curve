import { BN, Program } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { BondingCurve, IDL } from '../../target/types/bonding_curve';

export { BondingCurve, IDL };
export type BondingCurveProgram = Program<BondingCurve>;

export interface BondingCurveParams {
    aTotalTokens: BN;
    kVirtualPoolOffset: BN;
    cBondingScaleFactor: BN;
}

export interface UpdateCoreParams {
    admin?: PublicKey;
    gradThreshold: BN;
    totalSupply: BN;
    bondingCurve: BondingCurveParams;
    acceptedBaseMint: PublicKey;
}

export interface MintSupplyParams {
    name: string;
    symbol: string;
    uri: string;
    totalChains: number;
}

export type XyberCoreAccount = Awaited<ReturnType<BondingCurveProgram['account']['xyberCore']['fetch']>>;
export type XyberTokenAccount = Awaited<ReturnType<BondingCurveProgram['account']['xyberToken']['fetch']>>;
