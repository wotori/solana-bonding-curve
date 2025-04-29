import { PublicKey } from '@solana/web3.js';
import { getAssociatedTokenAddress } from '@solana/spl-token';
import { TOKEN_FACTORY_PROGRAM_ID, METAPLEX_PROGRAM_ID } from './constants';

export function findXyberCorePda(programId: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [Buffer.from("xyber_core")],
        programId
    );
}

export function findXyberTokenPda(tokenSeed: PublicKey, programId: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [Buffer.from("xyber_token"), tokenSeed.toBuffer()],
        programId
    );
}

export function findMintPda(tokenSeed: PublicKey, tokenFactoryProgramId: PublicKey = TOKEN_FACTORY_PROGRAM_ID): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [Buffer.from("MINT"), tokenSeed.toBuffer()],
        tokenFactoryProgramId
    );
}

export function findMetadataPda(mintPda: PublicKey, metaplexProgramId: PublicKey = METAPLEX_PROGRAM_ID): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [
            Buffer.from("metadata"),
            metaplexProgramId.toBuffer(),
            mintPda.toBuffer(),
        ],
        metaplexProgramId
    );
}

export async function findEscrowAta(paymentMint: PublicKey, xyberTokenPda: PublicKey): Promise<PublicKey> {
    return getAssociatedTokenAddress(
        paymentMint,
        xyberTokenPda,
        true
    );
}

export async function findVaultAta(projectMint: PublicKey, xyberTokenPda: PublicKey): Promise<PublicKey> {
    return getAssociatedTokenAddress(
        projectMint,
        xyberTokenPda,
        true
    );
}

export async function findCoreEscrowAta(paymentMint: PublicKey, xyberCorePda: PublicKey): Promise<PublicKey> {
    return getAssociatedTokenAddress(
        paymentMint,
        xyberCorePda,
        true
    );
}