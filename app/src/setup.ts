import { AnchorProvider, Program, Idl } from "@coral-xyz/anchor";
import { clusterApiUrl, Connection } from "@solana/web3.js";
import idl from "../../target/idl/bonding_curve.json";
import { BondingCurve } from "../../target/types/bonding_curve";
import { WalletContextState } from '@solana/wallet-adapter-react';

console.log('IDL metadata.address =', (idl as any).metadata?.address);

const connection = new Connection(clusterApiUrl('devnet'), {
    commitment: 'confirmed',
    confirmTransactionInitialTimeout: 60000,
    wsEndpoint: clusterApiUrl('devnet').replace('https', 'wss'),
    disableRetryOnRateLimit: false,
    httpHeaders: {
        'Content-Type': 'application/json'
    }
});

// Create a wallet adapter that implements the Wallet interface
const createWalletAdapter = (wallet: WalletContextState) => {
    if (
        !wallet.publicKey ||
        !wallet.signTransaction ||
        !wallet.signAllTransactions ||
        !wallet.signMessage
    ) {
        throw new Error('Wallet not connected');
    }

    return {
        publicKey: wallet.publicKey,
        signTransaction: wallet.signTransaction,
        signAllTransactions: wallet.signAllTransactions,
        signMessage: wallet.signMessage
    };
};

// We'll create the program instance when we have the wallet
export const getProgram = (wallet: WalletContextState) => {
    // Create the provider with the wallet adapter
    const provider = new AnchorProvider(
        connection,
        createWalletAdapter(wallet),
        AnchorProvider.defaultOptions()
    );

    // Create and return the program instance
    return new Program<BondingCurve>(
        idl as unknown as BondingCurve,
        '8FydojysL5DJ8M3s15JLFEbsKzyQ1BcFgSMVDvJetEEq',
        provider
    );
};
