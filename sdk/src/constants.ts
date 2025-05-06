import { PublicKey, SystemProgram } from '@solana/web3.js';


export const TOKEN_FACTORY_PROGRAM_ID = new PublicKey(
    process.env.TOKEN_FACTORY_PROGRAM_ID || "851Ez1PDMZY4yGYahRba87g7CYtmCfD8v5TP85cGj95p"
);

export const METAPLEX_PROGRAM_ID = new PublicKey(
    process.env.METAPLEX_PROGRAM_ID || "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// System program
export { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
export const SYSTEM_PROGRAM_ID = SystemProgram.programId;

// Other
export const DEFAULT_DECIMALS = 9;
export const LAMPORTS_PER_TOKEN_BASE = 10 ** DEFAULT_DECIMALS;