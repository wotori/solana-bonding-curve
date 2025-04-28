import * as anchor from "@project-serum/anchor";
import { Program, BN, Idl } from "@project-serum/anchor";
import {
    PublicKey,
    Keypair,
    SystemProgram,
    Connection,
    Signer,
    ConfirmOptions
} from "@solana/web3.js";
import {
    getAssociatedTokenAddress,
    getAccount,
    Account,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getMint,
    Mint
} from "@solana/spl-token";

import {
    BondingCurve,
    BondingCurveProgram,
    IDL,
    UpdateCoreParams,
    MintSupplyParams,
    XyberCoreAccount,
    XyberTokenAccount
} from './types';

import {
    TOKEN_FACTORY_PROGRAM_ID,
    METAPLEX_PROGRAM_ID,
} from './constants';

import {
    findXyberCorePda,
    findXyberTokenPda,
    findMintPda,
    findMetadataPda,
    findEscrowAta,
    findVaultAta,
    findCoreEscrowAta,
} from './pda';

export interface XyberClientConfig {
    connection: Connection;
    wallet?: anchor.Wallet;
    payer?: Keypair;
    programId: PublicKey;
    tokenFactoryProgramId?: PublicKey;
    metaplexProgramId?: PublicKey;
    confirmOpts?: ConfirmOptions;
}

export class XyberClient {
    readonly connection: Connection;
    readonly programId: PublicKey;
    readonly tokenFactoryProgramId: PublicKey;
    readonly metaplexProgramId: PublicKey;
    readonly provider: anchor.AnchorProvider;
    readonly program: BondingCurveProgram;
    readonly payer: Keypair;

    constructor(config: XyberClientConfig) {
        this.connection = config.connection;
        this.programId = config.programId;
        this.tokenFactoryProgramId = config.tokenFactoryProgramId ?? TOKEN_FACTORY_PROGRAM_ID;
        this.metaplexProgramId = config.metaplexProgramId ?? METAPLEX_PROGRAM_ID;
        if (config.wallet && config.wallet.payer) {
            this.payer = config.wallet.payer;
        } else if (config.payer) {
            this.payer = config.payer;
        } else {
            throw new Error("Wallet or Payer must be provided in config");
        }
        const walletForProvider = config.wallet ?? new anchor.Wallet(this.payer);
        this.provider = new anchor.AnchorProvider(
            this.connection,
            walletForProvider,
            config.confirmOpts ?? anchor.AnchorProvider.defaultOptions()
        );
        this.program = new Program<BondingCurve>(
            IDL as BondingCurve,
            this.programId,
            this.provider
        );
    }

    getXyberCorePda(): [PublicKey, number] {
        return findXyberCorePda(this.programId);
    }

    getXyberTokenPda(tokenSeed: PublicKey): [PublicKey, number] {
        return findXyberTokenPda(tokenSeed, this.programId);
    }

    getMintPda(tokenSeed: PublicKey): [PublicKey, number] {
        return findMintPda(tokenSeed, this.tokenFactoryProgramId);
    }

    getMetadataPda(mintPda: PublicKey): [PublicKey, number] {
        return findMetadataPda(mintPda, this.metaplexProgramId);
    }

    async getEscrowAta(paymentMint: PublicKey, tokenSeed: PublicKey): Promise<PublicKey> {
        const [xyberTokenPda] = this.getXyberTokenPda(tokenSeed);
        return findEscrowAta(paymentMint, xyberTokenPda);
    }

    async getVaultAta(tokenSeed: PublicKey): Promise<PublicKey> {
        const [mintPda] = this.getMintPda(tokenSeed);
        const [xyberTokenPda] = this.getXyberTokenPda(tokenSeed);
        return findVaultAta(mintPda, xyberTokenPda);
    }

    async getCoreEscrowAta(paymentMint: PublicKey): Promise<PublicKey> {
        const [xyberCorePda] = this.getXyberCorePda();
        return findCoreEscrowAta(paymentMint, xyberCorePda);
    }

    async getCoreState(xyberCorePda?: PublicKey): Promise<XyberCoreAccount | null> {
        const pda = xyberCorePda ?? this.getXyberCorePda()[0];
        try {
            return await this.program.account.xyberCore.fetch(pda);
        } catch (error) {
            if (error.message.includes("Account does not exist")) {
                return null;
            }
            console.error("Error fetching core state:", error);
            throw error;
        }
    }

    async getTokenState(tokenSeed: PublicKey, xyberTokenPda?: PublicKey): Promise<XyberTokenAccount | null> {
        const pda = xyberTokenPda ?? this.getXyberTokenPda(tokenSeed)[0];
        try {
            return await this.program.account.xyberToken.fetch(pda);
        } catch (error) {
            if (error.message.includes("Account does not exist")) {
                return null;
            }
            console.error("Error fetching token state:", error);
            throw error;
        }
    }

    async getSplAccountInfo(accountPublicKey: PublicKey): Promise<Account | null> {
        try {
            return await getAccount(this.connection, accountPublicKey, this.provider.opts.commitment);
        } catch (error: any) {
            if (error.message.includes('could not find account') || error.message.includes('Account does not exist')) {
                return null;
            }
            throw error;
        }
    }

    async getSplMintInfo(mintPublicKey: PublicKey): Promise<Mint | null> {
        try {
            return await getMint(this.connection, mintPublicKey, this.provider.opts.commitment);
        } catch (error: any) {
            if (error.message.includes('could not find account') || error.message.includes('Account does not exist')) {
                return null;
            }
            throw error;
        }
    }

    async updateCore(
        params: UpdateCoreParams,
        admin?: Signer
    ): Promise<string> {
        const signer = admin ?? this.payer;
        const [xyberCorePda] = this.getXyberCorePda();
        const coreEscrowAta = await this.getCoreEscrowAta(params.acceptedBaseMint);

        return this.program.methods
            .updateXyberCoreInstruction(params as any)
            .accounts({
                admin: signer.publicKey,
                xyberCore: xyberCorePda,
                newAcceptedBaseMint: params.acceptedBaseMint,
                escrowTokenAccount: coreEscrowAta,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([signer])
            .rpc();
    }

    async mintFullSupply(
        params: MintSupplyParams,
        tokenSeed: PublicKey,
        creator?: Signer,
        paymentMint?: PublicKey
    ): Promise<string> {
        const signer = creator ?? this.payer;
        const [xyberCorePda] = this.getXyberCorePda();
        const [xyberTokenPda] = this.getXyberTokenPda(tokenSeed);
        const [mintPda] = this.getMintPda(tokenSeed);
        const [metadataPda] = this.getMetadataPda(mintPda);
        const vaultAta = await this.getVaultAta(tokenSeed);

        let finalPaymentMint = paymentMint;
        if (!finalPaymentMint) {
            const coreState = await this.getCoreState(xyberCorePda);
            if (!coreState || !coreState.acceptedBaseMint) {
                throw new Error("Payment mint not provided and could not be fetched from core state.");
            }
            finalPaymentMint = coreState.acceptedBaseMint;
        }
        const escrowAta = await this.getEscrowAta(finalPaymentMint, tokenSeed);

        return this.program.methods
            .mintFullSupplyInstruction(params)
            .accounts({
                xyberCore: xyberCorePda,
                xyberToken: xyberTokenPda,
                tokenSeed: tokenSeed,
                creator: signer.publicKey,
                mint: mintPda,
                vaultTokenAccount: vaultAta,
                metadataAccount: metadataPda,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                tokenMetadataProgram: this.metaplexProgramId,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                tokenFactoryProgram: this.tokenFactoryProgramId,
                escrowTokenAccount: escrowAta,
                paymentMint: finalPaymentMint,
            })
            .signers([signer])
            .rpc();
    }

    async buyExactInput(
        tokenSeed: PublicKey,
        amountBaseIn: BN,
        minAmountOut: BN,
        buyer: Signer,
        paymentMint?: PublicKey
    ): Promise<string> {
        const [xyberCorePda] = this.getXyberCorePda();
        const [xyberTokenPda] = this.getXyberTokenPda(tokenSeed);
        const [mintPda] = this.getMintPda(tokenSeed);

        let finalPaymentMint = paymentMint;
        if (!finalPaymentMint) {
            const coreState = await this.getCoreState(xyberCorePda);
            if (!coreState || !coreState.acceptedBaseMint) {
                throw new Error("Payment mint not provided and could not be fetched from core state.");
            }
            finalPaymentMint = coreState.acceptedBaseMint;
        }

        const escrowAta = await this.getEscrowAta(finalPaymentMint, tokenSeed);
        const vaultAta = await this.getVaultAta(tokenSeed);
        const buyerTokenAccount = await getAssociatedTokenAddress(mintPda, buyer.publicKey);
        const buyerPaymentAccount = await getAssociatedTokenAddress(finalPaymentMint, buyer.publicKey);

        return this.program.methods
            .buyExactInputInstruction(amountBaseIn, minAmountOut)
            .accounts({
                xyberCore: xyberCorePda,
                tokenSeed: tokenSeed,
                buyer: buyer.publicKey,
                xyberToken: xyberTokenPda,
                escrowTokenAccount: escrowAta,
                paymentMint: finalPaymentMint,
                mint: mintPda,
                vaultTokenAccount: vaultAta,
                buyerTokenAccount: buyerTokenAccount,
                buyerPaymentAccount: buyerPaymentAccount,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([buyer])
            .rpc();
    }

    async sellExactInput(
        tokenSeed: PublicKey,
        amountTokensIn: BN,
        minAmountOut: BN,
        user: Signer,
        paymentMint?: PublicKey
    ): Promise<string> {
        const [xyberCorePda] = this.getXyberCorePda();
        const [xyberTokenPda] = this.getXyberTokenPda(tokenSeed);
        const [mintPda] = this.getMintPda(tokenSeed);

        let finalPaymentMint = paymentMint;
        if (!finalPaymentMint) {
            const coreState = await this.getCoreState(xyberCorePda);
            if (!coreState || !coreState.acceptedBaseMint) {
                throw new Error("Payment mint not provided and could not be fetched from core state.");
            }
            finalPaymentMint = coreState.acceptedBaseMint;
        }

        const escrowAta = await this.getEscrowAta(finalPaymentMint, tokenSeed);
        const vaultAta = await this.getVaultAta(tokenSeed);
        const userTokenAccount = await getAssociatedTokenAddress(mintPda, user.publicKey);
        const userPaymentAccount = await getAssociatedTokenAddress(finalPaymentMint, user.publicKey);

        return this.program.methods
            .sellExactInputInstruction(amountTokensIn, minAmountOut)
            .accounts({
                xyberCore: xyberCorePda,
                tokenSeed: tokenSeed,
                user: user.publicKey,
                xyberToken: xyberTokenPda,
                escrowTokenAccount: escrowAta,
                paymentMint: finalPaymentMint,
                mint: mintPda,
                vaultTokenAccount: vaultAta,
                userTokenAccount: userTokenAccount,
                userPaymentAccount: userPaymentAccount,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();
    }

    async withdrawLiquidity(
        tokenSeed: PublicKey,
        admin?: Signer,
        paymentMint?: PublicKey
    ): Promise<string> {
        const signer = admin ?? this.payer;
        const [xyberCorePda] = this.getXyberCorePda();
        const [xyberTokenPda] = this.getXyberTokenPda(tokenSeed);
        const [mintPda] = this.getMintPda(tokenSeed);

        let finalPaymentMint = paymentMint;
        if (!finalPaymentMint) {
            const coreState = await this.getCoreState(xyberCorePda);
            if (!coreState || !coreState.acceptedBaseMint) {
                throw new Error("Payment mint not provided and could not be fetched from core state.");
            }
            finalPaymentMint = coreState.acceptedBaseMint;
        }

        const escrowAta = await this.getEscrowAta(finalPaymentMint, tokenSeed);
        const vaultAta = await this.getVaultAta(tokenSeed);
        const adminBaseAta = await getAssociatedTokenAddress(finalPaymentMint, signer.publicKey);
        const adminVaultAta = await getAssociatedTokenAddress(mintPda, signer.publicKey);

        return this.program.methods
            .withdrawLiquidity()
            .accounts({
                admin: signer.publicKey,
                xyberCore: xyberCorePda,
                xyberToken: xyberTokenPda,
                tokenSeed: tokenSeed,
                creator: signer.publicKey,
                escrowTokenAccount: escrowAta,
                baseTokenMint: finalPaymentMint,
                mint: mintPda,
                vaultTokenAccount: vaultAta,
                adminTokenAccount: adminBaseAta,
                adminVaultAccount: adminVaultAta,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([signer])
            .rpc();
    }
}