import { BN, web3 } from "@project-serum/anchor";
import {
    PublicKey,
    SystemProgram,
    Connection,
    Transaction,
    Keypair
} from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress
} from "@solana/spl-token";
import { getProgram } from "./setup";
import { WalletContextState } from "@solana/wallet-adapter-react";
import { deriveAddresses, getAssociatedAccounts } from "./utls";


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

const METAPLEX_PROGRAM_ID = new PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const TOKEN_FACTORY_PROGRAM_ID = new PublicKey(
    "TF5AoQEG87r1gpWsNzADMxYean6tfdGVUouQQ5LbYPP"
);

export const SYSVAR_RENT_PUBKEY = new PublicKey(
    "SysvarRent111111111111111111111111111111111"
);

const PAYMENT_MINT_PUBKEY = new PublicKey(
    '6WQQPDXsBxkgMwuApkXbV2bUf3CZAJmGBDqk62aMpmKR'
);

// Fetch core state
export async function fetchCoreState(wallet: WalletContextState) {
    try {
        const program = getProgram(wallet);
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
    wallet: WalletContextState,
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
        const program = getProgram(wallet);

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
    expectedOut: number,
    wallet: WalletContextState,
) {
    try {
        const program = getProgram(wallet);

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
    tokenAmount: number,
    wallet: WalletContextState,
) {
    let program = getProgram(wallet)
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
export async function mintFullSupplyTx(
    tokenName: string,
    tokenSymbol: string,
    tokenUri: string,
    wallet: WalletContextState,
    accounts: {
        xyberTokenPda: PublicKey;
        xyberCorePda: PublicKey;
        tokenSeedKeypair: Keypair;
        mintPda: PublicKey;
        vaultTokenAccount: PublicKey;
        metadataPda: PublicKey;
    }
): Promise<Transaction> {
    const program = getProgram(wallet);

    // Let Anchor build the transaction for us (same style as exactBuy)
    const transaction = program.methods
        .mintFullSupplyInstruction({
            name: tokenName,
            symbol: tokenSymbol,
            uri: tokenUri,
        })
        .accounts({
            payer: wallet.publicKey,
            xyberToken: accounts.xyberTokenPda,
            xyberCore: accounts.xyberCorePda,
            tokenSeed: accounts.tokenSeedKeypair.publicKey,
            creator: wallet.publicKey,
            mint: accounts.mintPda,
            vaultTokenAccount: accounts.vaultTokenAccount,
            metadataAccount: accounts.metadataPda,
            rent: web3.SYSVAR_RENT_PUBKEY,
            tokenMetadataProgram: METAPLEX_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            tokenFactoryProgram: TOKEN_FACTORY_PROGRAM_ID
        })
        .transaction();

    return transaction;
}

// 2) Build transaction for the "initialBuyTokensInstruction"
export async function initialBuyTx(
    depositLamports: number, // how many lamports to deposit
    wallet: WalletContextState,
    accounts: {
        xyberCorePda: PublicKey;
        tokenSeedKeypair: Keypair;
        wallet: WalletContextState;
        xyberTokenPda: PublicKey;
        escrowTokenAccount: PublicKey;
        creatorPaymentAccount: PublicKey;
        mintPda: PublicKey;
        vaultTokenAccount: PublicKey;
        creatorTokenAccount: PublicKey;
    }
): Promise<web3.TransactionInstruction> {
    const program = getProgram(wallet);

    const { } =
        await deriveAddresses(wallet, accounts.tokenSeedKeypair);

    const {
        escrowTokenAccount,
        creatorTokenAccount,
        creatorPaymentAccount,
        vaultTokenAccount
    } = await getAssociatedAccounts(wallet, accounts.mintPda, accounts.xyberTokenPda);

    const transaction = program.methods
        .initialBuyTokensInstruction(new BN(depositLamports))
        .accounts({
            xyberCore: accounts.xyberCorePda,
            tokenSeed: accounts.tokenSeedKeypair.publicKey,
            creator: wallet.publicKey,
            xyberToken: accounts.xyberTokenPda,
            escrowTokenAccount: escrowTokenAccount,
            paymentMint: PAYMENT_MINT_PUBKEY,
            creatorPaymentAccount: creatorPaymentAccount,
            mint: accounts.mintPda,
            vaultTokenAccount: vaultTokenAccount,
            creatorTokenAccount: creatorTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
        })
        .instruction();

    return transaction;
}

// 3) Combine the two into one transaction
//    We build the two separate TX objects, then merge instructions.
export async function mintFullSupplyAndInitialBuyInOneTx(
    sendTransaction: (tx: Transaction, conn: Connection) => Promise<string>,
    connection: Connection,
    wallet: WalletContextState,
    {
        tokenName,
        tokenSymbol,
        tokenUri,
        depositLamports
    }: {
        tokenName: string;
        tokenSymbol: string;
        tokenUri: string;
        depositLamports: number;
    }
) {
    try {
        const tokenSeedKeypair = Keypair.generate();
        const { xyberCorePda, xyberTokenPda, mintPda, metadataPda } =
            await deriveAddresses(wallet, tokenSeedKeypair);

        const {
            escrowTokenAccount,
            creatorTokenAccount,
            creatorPaymentAccount,
            vaultTokenAccount
        } = await getAssociatedAccounts(wallet, mintPda, xyberTokenPda);

        // a) Build the first transaction
        const txMint = await mintFullSupplyTx(tokenName, tokenSymbol, tokenUri, wallet, {
            xyberTokenPda,
            xyberCorePda,
            tokenSeedKeypair,
            mintPda,
            vaultTokenAccount,
            metadataPda,
        });

        // b) Build the second transaction
        const txBuy = await initialBuyTx(depositLamports, wallet, {
            xyberCorePda,
            tokenSeedKeypair,
            wallet,
            xyberTokenPda,
            escrowTokenAccount,
            creatorPaymentAccount,
            mintPda,
            vaultTokenAccount,
            creatorTokenAccount,
        });

        // c) Combine instructions (and signers if needed) into one
        const combinedTx = new Transaction()
            .add(txMint)
            .add(txBuy);

        // d) Send + confirm
        const txSig = await sendTransaction(combinedTx, connection);
        console.log("mintFullSupplyAndInitialBuyInOneTx =>", txSig);
        return txSig;
    } catch (err) {
        console.error("mintFullSupplyAndInitialBuyInOneTx error:", err);
        throw err;
    }
}