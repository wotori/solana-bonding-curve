import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { assert } from "chai";

import {
    DEVNET_URL,
    CREATOR_KEYPAIR_PATH,
} from "./constants";

import { BondingCurve } from "../target/types/bonding_curve";

describe("XyberCore Stats Only", () => {
    let connection: Connection;
    let provider: anchor.AnchorProvider;
    let program: Program<BondingCurve>;
    let creatorKeypair: Keypair;

    let xyberCorePda: PublicKey;

    before("Setup Anchor + PDA", async () => {
        connection = new anchor.web3.Connection(DEVNET_URL, { commitment: "processed" });

        creatorKeypair = Keypair.fromSecretKey(
            new Uint8Array(require(CREATOR_KEYPAIR_PATH))
        );

        const wallet = new anchor.Wallet(creatorKeypair);
        provider = new anchor.AnchorProvider(connection, wallet, {});
        anchor.setProvider(provider);

        program = anchor.workspace.BondingCurve as Program<BondingCurve>;

        // Derive the XyberCore PDA (using the same seeds as in your main test)
        [xyberCorePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("xyber_core")],
            program.programId
        );
    });

    it("Fetches XyberCore and displays info (human-readable)", async () => {
        const xyberCoreState = await program.account.xyberCore.fetch(xyberCorePda);

        // Convert each of the BN fields from hex to decimal
        const aTotalTokensDecimal = new anchor.BN(xyberCoreState.bondingCurve.aTotalTokens, 16).toString(10);
        const kVirtualPoolOffsetDecimal = new anchor.BN(xyberCoreState.bondingCurve.kVirtualPoolOffset, 16).toString(10);
        const cBondingScaleFactorDecimal = new anchor.BN(xyberCoreState.bondingCurve.cBondingScaleFactor, 16).toString(10);

        console.log("=== XyberCore State (human-readable) ===");
        console.log("admin =", xyberCoreState.admin.toBase58 ? xyberCoreState.admin.toBase58() : xyberCoreState.admin);
        console.log("gradThreshold =", xyberCoreState.gradThreshold.toString());
        console.log("acceptedBaseMint =", xyberCoreState.acceptedBaseMint.toBase58 ? xyberCoreState.acceptedBaseMint.toBase58() : xyberCoreState.acceptedBaseMint);

        // Bonding Curve fields (raw decimal integers)
        console.log("\n-- Bonding Curve --");
        console.log("aTotalTokens =", aTotalTokensDecimal);
        console.log("kVirtualPoolOffset =", kVirtualPoolOffsetDecimal);
        console.log("cBondingScaleFactor =", cBondingScaleFactorDecimal);

        assert(xyberCoreState, "Failed to fetch XyberCore state");
    });
});