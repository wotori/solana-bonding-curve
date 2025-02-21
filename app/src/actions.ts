// src/actions.ts
import { BN } from "@project-serum/anchor";
import { PublicKey, SystemProgram, Connection, Transaction } from "@solana/web3.js";
import { getAssociatedTokenAddress } from "@solana/spl-token";
import { program } from "./setup";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

// === Из вывода теста ("Dumping Key Info") ===
export const XYBER_CORE_PDA = new PublicKey("HFrD8J1m9zc5qCzEmLDRmZQJ5vwMGfZ8LaYEHAKi8Bym");
export const XYBER_TOKEN_PDA = new PublicKey("HskcNqNBC8CxcxbWy8Bh2aUBXHtfmAwbCtytM6UFqMUh");
export const ESCROW_TOKEN_ACCOUNT_PDA = new PublicKey("iEnzy2CbmPrErU7ZJPRcwTeDqo9JPrVQ7cMa9wqktqR");
export const VAULT_TOKEN_ACCOUNT_PDA = new PublicKey("9RqTjeKC84zAoHszb8Vzkr9UB9xEXddHchPJ7fuYfi3y");
export const PAYMENT_MINT = new PublicKey("6WQQPDXsBxkgMwuApkXbV2bUf3CZAJmGBDqk62aMpmKR");

// Из "Dump Info": mintPda =
const MINT_PDA = new PublicKey("C7rkMEvztRWr9qkd8ZbpWkfNjsdPvxbL34NbTMxpEyr7");

// Если инструкции где-то требуют "creator", это "FCMPSxbmyMugTRyfdGPNx4mdeAaVDcSnVaN3p82zBcT8" (админ)
const CREATOR_PUBKEY = new PublicKey("FCMPSxbmyMugTRyfdGPNx4mdeAaVDcSnVaN3p82zBcT8");

// ------------------------------------------------------------------
// 1) fetchCoreState
// ------------------------------------------------------------------
export async function fetchCoreState() {
    try {
        const state = await program.account.xyberCore.fetch(XYBER_CORE_PDA);
        console.log("Current XYBER CORE state:", state);
        return state;
    } catch (err) {
        console.error("Failed to fetchCoreState:", err);
        throw err;
    }
}

// ------------------------------------------------------------------
// 2) updateCoreParams
// ------------------------------------------------------------------
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
        // Собираем структуру bondingCurve
        const bc = {
            aTotalTokens: new BN(aTotalTokens), // строка -> BN
            kVirtualPoolOffset: new BN(kVirtualPoolOffset).mul(new BN(10 ** 9)),
            cBondingScaleFactor: new BN(cBondingScaleFactor).mul(new BN(10 ** 9)),
            xTotalBaseDeposit: new BN(0),
        };

        const params = {
            admin: publicKey, // кто вызывает
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
        console.log("updateCoreParams => txSig:", txSig);
        return txSig;
    } catch (err) {
        console.error("updateCoreParams error:", err);
        throw err;
    }
}

// ------------------------------------------------------------------
// 3) exactBuy
// ------------------------------------------------------------------
export async function exactBuy(
    publicKey: PublicKey,
    sendTransaction: (tx: Transaction, conn: Connection) => Promise<string>,
    connection: Connection,
    baseIn: number,
    expectedOut: number
) {
    try {
        // baseIn, expectedOut в "целых" токенах, умножаем
        const baseInLamports = new BN(baseIn).mul(new BN(10 ** 9));
        const expectedOutBN = new BN(expectedOut).mul(new BN(10 ** 9));

        // Buyer Payment ATA
        const buyerPaymentAccount = await getAssociatedTokenAddress(PAYMENT_MINT, publicKey);

        // Buyer project-token ATA
        const buyerTokenAccount = await getAssociatedTokenAddress(MINT_PDA, publicKey);

        // Формируем Transaction
        const transaction = await program.methods
            .buyExactInputInstruction(baseInLamports, expectedOutBN)
            .accounts({
                xyberCore: XYBER_CORE_PDA,
                tokenSeed: XYBER_TOKEN_PDA,       // "seed" в коде
                buyer: publicKey,
                creator: CREATOR_PUBKEY,         // admin key (если нужно)
                xyberToken: XYBER_TOKEN_PDA,
                escrowTokenAccount: ESCROW_TOKEN_ACCOUNT_PDA,
                paymentMint: PAYMENT_MINT,
                mint: MINT_PDA,
                vaultTokenAccount: VAULT_TOKEN_ACCOUNT_PDA,
                buyerTokenAccount,
                buyerPaymentAccount,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .transaction();

        const txSig = await sendTransaction(transaction, connection);
        console.log("exactBuy success, txSig =", txSig);
        return txSig;
    } catch (err) {
        console.error("exactBuy error:", err);
        throw err;
    }
}

// ------------------------------------------------------------------
// 4) exactSell
// ------------------------------------------------------------------
export async function exactSell(
    publicKey: PublicKey,
    sendTransaction: (tx: Transaction, conn: Connection) => Promise<string>,
    connection: Connection,
    tokenAmount: number
) {
    try {
        // продаём tokenAmount "целых" проект-токенов
        const tokensToSell = new BN(tokenAmount).mul(new BN(10 ** 9));

        const userTokenAccount = await getAssociatedTokenAddress(MINT_PDA, publicKey);
        const userPaymentAccount = await getAssociatedTokenAddress(PAYMENT_MINT, publicKey);

        const transaction = await program.methods
            .sellExactInputInstruction(tokensToSell)
            .accounts({
                xyberCore: XYBER_CORE_PDA,
                tokenSeed: XYBER_TOKEN_PDA,
                user: publicKey,
                creator: CREATOR_PUBKEY, // admin key
                xyberToken: XYBER_TOKEN_PDA,
                escrowTokenAccount: ESCROW_TOKEN_ACCOUNT_PDA,
                paymentMint: PAYMENT_MINT,
                mint: MINT_PDA,
                vaultTokenAccount: VAULT_TOKEN_ACCOUNT_PDA,
                userTokenAccount,
                userPaymentAccount,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .transaction();

        const txSig = await sendTransaction(transaction, connection);
        console.log("exactSell success, txSig =", txSig);
        return txSig;
    } catch (err) {
        console.error("exactSell error:", err);
        throw err;
    }
}