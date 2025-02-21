// src/actions.ts
import { BN } from "@project-serum/anchor";
import {
    PublicKey,
    SystemProgram,
    Connection,
    Transaction
} from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress
} from "@solana/spl-token";
import { program } from "./setup";

// PDAs & Mints
export const TOKEN_SEED_PUBKEY = new PublicKey("NoMZLvywWv7x3KyVHbMwp6kgMzjjyMuUQGgt1sh4VUH");
export const XYBER_CORE_PDA = new PublicKey("HFrD8J1m9zc5qCzEmLDRmZQJ5vwMGfZ8LaYEHAKi8Bym");
export const XYBER_TOKEN_PDA = new PublicKey("962ownUvqXXqArg6sgC9Kk6PPQ8uDSZiEbREByc21yq8");
export const MINT_PDA = new PublicKey("sxkSLFk6tbfTcucsuSe2tuEJ1FGGgNhMbkujicWetBh");
export const VAULT_TOKEN_ACCOUNT_PDA = new PublicKey("5u2vPR5nZgZGe2ppwgrZUyB4BXSeeFdTmZZRnAbQBJLF");
export const ESCROW_TOKEN_ACCOUNT_PDA = new PublicKey("2joSWicyUq2mFhrTADoNNg4kjR2yQw59zqrwR5DuCBqW");
export const CREATOR_PUBKEY = new PublicKey("FCMPSxbmyMugTRyfdGPNx4mdeAaVDcSnVaN3p82zBcT8");
export const BUYER_PUBKEY = new PublicKey("CW97fy6bRvkuyTXkunu4A5Qi8VPMidncPm79EpgHtqZF");
export const PAYMENT_MINT = new PublicKey("6WQQPDXsBxkgMwuApkXbV2bUf3CZAJmGBDqk62aMpmKR");

// Fetch core state
export async function fetchCoreState() {
    try {
        const state = await program.account.xyberCore.fetch(XYBER_CORE_PDA);
        console.log("XYBER CORE state:", state);
        return state;
    } catch (err) {
        console.error("fetchCoreState error:", err);
        throw err;
    }
}

// Update core params
export async function updateCoreParams(
    publicKey: PublicKey,
    sendTransaction: (tx: Transaction, conn: Connection) => Promise<string>,
    connection: Connection,
    {
        gradThreshold,
        graduateDollarsAmount,
        aTotalTokens,
        kVirtualPoolOffset,
        cBondingScaleFactor
    }: {
        gradThreshold: number;
        graduateDollarsAmount: number;
        aTotalTokens: string;
        kVirtualPoolOffset: string;
        cBondingScaleFactor: string;
    }
) {
    try {
        const bc = {
            aTotalTokens: new BN(aTotalTokens),
            kVirtualPoolOffset: new BN(kVirtualPoolOffset).mul(new BN(10 ** 9)),
            cBondingScaleFactor: new BN(cBondingScaleFactor).mul(new BN(10 ** 9)),
            xTotalBaseDeposit: new BN(0),
        };

        const params = {
            admin: publicKey,
            gradThreshold,
            bondingCurve: bc,
            acceptedBaseMint: PAYMENT_MINT,
            graduateDollarsAmount,
        };

        const transaction = await program.methods
            .updateXyberCoreInstruction(params)
            .accounts({
                admin: publicKey,
                xyberCore: XYBER_CORE_PDA,
            })
            .transaction();

        const txSig = await sendTransaction(transaction, connection);
        console.log("updateCoreParams =>", txSig);
        return txSig;
    } catch (err) {
        console.error("updateCoreParams error:", err);
        throw err;
    }
}

// Exact Buy
export async function exactBuy(
    publicKey: PublicKey,
    sendTransaction: (tx: Transaction, conn: Connection) => Promise<string>,
    connection: Connection,
    baseIn: number,
    expectedOut: number
) {
    try {
        const baseInLamports = new BN(baseIn).mul(new BN(10 ** 9));
        const expectedOutLamports = new BN(expectedOut).mul(new BN(10 ** 9));

        const buyerPaymentAccount = await getAssociatedTokenAddress(PAYMENT_MINT, publicKey);
        const buyerTokenAccount = await getAssociatedTokenAddress(MINT_PDA, publicKey);

        const transaction = await program.methods
            .buyExactInputInstruction(baseInLamports, expectedOutLamports)
            .accounts({
                xyberCore: XYBER_CORE_PDA,
                tokenSeed: TOKEN_SEED_PUBKEY,
                buyer: publicKey,
                creator: CREATOR_PUBKEY,
                xyberToken: XYBER_TOKEN_PDA,
                escrowTokenAccount: ESCROW_TOKEN_ACCOUNT_PDA,
                paymentMint: PAYMENT_MINT,
                mint: MINT_PDA,
                vaultTokenAccount: VAULT_TOKEN_ACCOUNT_PDA,
                buyerTokenAccount,
                buyerPaymentAccount,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .transaction();

        const txSig = await sendTransaction(transaction, connection);
        console.log("exactBuy =>", txSig);
        return txSig;
    } catch (err) {
        console.error("exactBuy error:", err);
        throw err;
    }
}

// Exact Sell
export async function exactSell(
    publicKey: PublicKey,
    sendTransaction: (tx: Transaction, conn: Connection) => Promise<string>,
    connection: Connection,
    tokenAmount: number
) {
    try {
        // If user is typing unscaled tokens, do not multiply again:
        const tokensToSell = new BN(tokenAmount);

        const userTokenAccount = await getAssociatedTokenAddress(MINT_PDA, publicKey);
        const userPaymentAccount = await getAssociatedTokenAddress(PAYMENT_MINT, publicKey);

        const transaction = await program.methods
            .sellExactInputInstruction(tokensToSell)
            .accounts({
                xyberCore: XYBER_CORE_PDA,
                tokenSeed: TOKEN_SEED_PUBKEY,
                user: publicKey,
                creator: CREATOR_PUBKEY,
                xyberToken: XYBER_TOKEN_PDA,
                escrowTokenAccount: ESCROW_TOKEN_ACCOUNT_PDA,
                paymentMint: PAYMENT_MINT,
                mint: MINT_PDA,
                vaultTokenAccount: VAULT_TOKEN_ACCOUNT_PDA,
                userTokenAccount,
                userPaymentAccount,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .transaction();

        const txSig = await sendTransaction(transaction, connection);
        console.log("exactSell =>", txSig);
        return txSig;
    } catch (err) {
        console.error("exactSell error:", err);
        throw err;
    }
}