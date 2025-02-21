
import { WalletContextState } from '@solana/wallet-adapter-react';
import {
    Keypair,
    PublicKey,
} from '@solana/web3.js';
import { Buffer } from "buffer";
import {
    getAssociatedTokenAddress,
} from '@solana/spl-token';

const METAPLEX_PROGRAM_ID = new PublicKey(
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);
const TOKEN_FACTORY_PROGRAM_ID = new PublicKey(
    'TF5AoQEG87r1gpWsNzADMxYean6tfdGVUouQQ5LbYPP'
);
const PAYMENT_MINT_PUBKEY = new PublicKey(
    '6WQQPDXsBxkgMwuApkXbV2bUf3CZAJmGBDqk62aMpmKR'
);

const XYBER_CORE_PROGRAM_ID = new PublicKey(
    '7TtWm2z8uixrGbxhkT1SYZfWfbiAJEg7zRaozUh46v2C'
);

export const deriveAddresses = async (
    wallet: WalletContextState,
    tokenSeedKeypair: Keypair
) => {
    const publicKey = wallet.publicKey as PublicKey; // Safe after validateWallet

    // Derive PDAs
    const [xyberCorePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('xyber_core')],
        XYBER_CORE_PROGRAM_ID
    );

    const [xyberTokenPda] = PublicKey.findProgramAddressSync(
        [
            Buffer.from('xyber_token'),
            publicKey.toBuffer(),
            tokenSeedKeypair.publicKey.toBuffer()
        ],
        XYBER_CORE_PROGRAM_ID
    );

    const [mintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('MINT'), tokenSeedKeypair.publicKey.toBuffer()],
        TOKEN_FACTORY_PROGRAM_ID
    );

    const [metadataPda] = PublicKey.findProgramAddressSync(
        [
            Buffer.from('metadata'),
            METAPLEX_PROGRAM_ID.toBuffer(),
            mintPda.toBuffer()
        ],
        METAPLEX_PROGRAM_ID
    );

    return { xyberCorePda, xyberTokenPda, mintPda, metadataPda };
};

export const getAssociatedAccounts = async (
    wallet: WalletContextState,
    mintPda: PublicKey,
    xyberTokenPda: PublicKey
) => {
    const publicKey = wallet.publicKey as PublicKey; // Safe after validateWallet

    const creatorTokenAccount = await getAssociatedTokenAddress(
        mintPda,
        publicKey
    );

    const creatorPaymentAccount = await getAssociatedTokenAddress(
        PAYMENT_MINT_PUBKEY,
        publicKey
    );

    const escrowTokenAccount = await getAssociatedTokenAddress(
        PAYMENT_MINT_PUBKEY,
        xyberTokenPda,
        true
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
        mintPda,
        xyberTokenPda,
        true
    );

    return {
        escrowTokenAccount,
        creatorTokenAccount,
        creatorPaymentAccount,
        vaultTokenAccount
    };
};