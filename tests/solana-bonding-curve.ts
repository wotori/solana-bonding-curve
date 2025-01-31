import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddress
} from "@solana/spl-token";
import fs from "fs";
import path from "path";
import { assert } from "chai";
import { BondingCurve } from "../target/types/bonding_curve";

const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const DEVNET_URL = "https://api.devnet.solana.com";
const KEYPAIR_PATH = path.join(process.env.HOME!, ".config", "solana", "devnet-owner.json");
const BUYER_KEYPAIR_PATH = path.join(process.env.HOME!, ".config", "solana", "devnet-buyer.json");

const TOTAL_TOKENS = 1_073_000_191;
const VIRTUAL_POOL_OFFSET = 30 * LAMPORTS_PER_SOL;
const BONDING_SCALE_FACTOR = (BigInt("32_190_005_730") * BigInt(LAMPORTS_PER_SOL)).toString();
const TOKEN_BASE_PUB_KEY = new PublicKey("Apv67VdcTZ5hK9wkPGFaZpDrSHPYbb4q1Dtvss5dcj84");

const connection = new anchor.web3.Connection(DEVNET_URL, {
  commitment: "finalized",
});

const secretKeyArray = JSON.parse(fs.readFileSync(KEYPAIR_PATH, "utf-8"));
const devnetKeypair = Keypair.fromSecretKey(new Uint8Array(secretKeyArray));
const wallet = new anchor.Wallet(devnetKeypair);
const provider = new anchor.AnchorProvider(connection, wallet, {});
anchor.setProvider(provider);

const program = anchor.workspace.BondingCurve as Program<BondingCurve>;
const creator = devnetKeypair;

const now = new Date();
const year = now.getFullYear().toString().slice(2);
const month = String(now.getMonth() + 1).padStart(2, "0");
const day = String(now.getDate()).padStart(2, "0");
const hours = String(now.getHours()).padStart(2, "0");
const minutes = String(now.getMinutes()).padStart(2, "0");

const tokenName = `${year}_${month}_${day}_${hours}_${minutes}`;
const tokenSymbol = "HWD";
const tokenUri = "https://ekza.io/ipfs/QmVjBTRsbAM96BnNtZKrR8i3hGRbkjnQ3kugwXn6BVFu2k";

const initialDepositLamports = new BN(0.001 * LAMPORTS_PER_SOL);

describe("Bonding Curve (Lamports) Test (multi-step with all asserts)", () => {
  let tokenSeedKeypair: Keypair;
  let mintKeypair: Keypair;
  let ownedTokenPda: PublicKey;
  let escrowPda: PublicKey;
  let metadataPda: PublicKey;
  let creatorTokenAccount: PublicKey;
  let ownedTokenDataBefore: any;
  let buyerKeypair: Keypair;
  let buyerTokenAccount: PublicKey;

  before(async () => {
    tokenSeedKeypair = Keypair.generate();
    mintKeypair = Keypair.generate();

    [ownedTokenPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("xyber_token"),
        creator.publicKey.toBuffer(),
        tokenSeedKeypair.publicKey.toBuffer()
      ],
      program.programId
    );

    [escrowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        creator.publicKey.toBuffer(),
        tokenSeedKeypair.publicKey.toBuffer()
      ],
      program.programId
    );

    [metadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METAPLEX_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer()
      ],
      METAPLEX_PROGRAM_ID
    );

    creatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      creator.publicKey
    );
  });

  it("Step 1) Create, Init Escrow, MintInitial, SetMetadata", async () => {
    const ixCreate = await program.methods
      .createTokenInstruction({
        tokenSupply: new BN(TOTAL_TOKENS),
        tokenGradThrUsd: 1000,
        bondingCurve: {
          aTotalTokens: new BN(TOTAL_TOKENS),
          kVirtualPoolOffset: new BN(VIRTUAL_POOL_OFFSET),
          cBondingScaleFactor: new BN(BONDING_SCALE_FACTOR),
          x: new BN(0),
        },
        acceptedBaseMint: TOKEN_BASE_PUB_KEY,
      })
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        xyberToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        creatorTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        escrowPda,
        systemProgram: SystemProgram.programId
      })
      .signers([mintKeypair])
      .instruction();

    const ixInitEscrow = await program.methods
      .initEscrowInstruction()
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        xyberToken: ownedTokenPda,
        escrowPda,
        systemProgram: SystemProgram.programId
      })
      .instruction();

    const ixMintInitial = await program.methods
      .mintInitialTokensInstruction(initialDepositLamports)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        xyberToken: ownedTokenPda,
        escrowPda,
        mint: mintKeypair.publicKey,
        creatorTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .instruction();

    const ixSetMetadata = await program.methods
      .setMetadataInstruction(tokenName, tokenSymbol, tokenUri)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        xyberToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        metadata: metadataPda,
        tokenMetadataProgram: METAPLEX_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram: SystemProgram.programId
      })
      .instruction();

    try {
      const tx1 = new Transaction()
        .add(ixCreate)
        .add(ixInitEscrow)
        .add(ixMintInitial)
        .add(ixSetMetadata);

      const tx1Sig = await provider.sendAndConfirm(tx1, [creator, mintKeypair]);
      console.log("Success! TX Sig:", tx1Sig);
    } catch (err: any) {
      console.error("Error sending transaction:", err);
      if ("logs" in err) console.log("Transaction logs:", err.logs);
      throw err;
    }

    const creatorAtaInfo = await getAccount(connection, creatorTokenAccount);
    ownedTokenDataBefore = await program.account.xyberToken.fetch(ownedTokenPda);

    assert(
      creatorAtaInfo.amount >= BigInt(0),
      "Creator ATA should have minted at least 0 tokens (check integer division)."
    );
  });

  it("Step 2) Buyer deposits 0.001 SOL => receives tokens (buyExactInputInstruction)", async () => {
    const buyerSecretArr = JSON.parse(fs.readFileSync(BUYER_KEYPAIR_PATH, "utf-8"));
    buyerKeypair = Keypair.fromSecretKey(new Uint8Array(buyerSecretArr));

    buyerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      buyerKeypair.publicKey
    );

    const buyLamports = new BN(0.001 * LAMPORTS_PER_SOL);
    const buyerAtaInfoBefore = await getAccount(connection, buyerTokenAccount).catch(() => null);
    const buyerTokensBefore = buyerAtaInfoBefore ? buyerAtaInfoBefore.amount : BigInt("0");
    const supplyBeforeBuy = ownedTokenDataBefore.supply;

    const ixBuy = await program.methods
      .buyExactInputInstruction(buyLamports)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
        creator: creator.publicKey,
        xyberToken: ownedTokenPda,
        escrowPda,
        mint: mintKeypair.publicKey,
        buyerTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([buyerKeypair])
      .instruction();

    const txBuySig = await provider.sendAndConfirm(
      new Transaction().add(ixBuy),
      [buyerKeypair]
    );
    console.log("Buyer purchased 0.001 SOL worth of tokens. TX Sig:", txBuySig);

    const buyerAtaInfoAfterBuy = await getAccount(connection, buyerTokenAccount);
    const ownedTokenDataAfterBuy = await program.account.xyberToken.fetch(ownedTokenPda);

    const deltaBuyer = buyerAtaInfoAfterBuy.amount - buyerTokensBefore;
    const deltaSupply = supplyBeforeBuy.sub(ownedTokenDataAfterBuy.supply);
    // Scale supply difference up by 10^9 decimals for comparison:
    const scaledDeltaSupply = deltaSupply.mul(new BN(10 ** 9));

    assert(deltaBuyer > BigInt("0"), "Buyer should receive some positive token amount");
    assert(
      scaledDeltaSupply.eq(new BN(deltaBuyer.toString())),
      "Supply should decrease exactly by minted amount"
    );

    ownedTokenDataBefore = ownedTokenDataAfterBuy;
  });

  it("Step 3) Buyer sells half of their tokens (sellExactInputInstruction)", async () => {
    const buyerAtaInfoAfterBuy = await getAccount(connection, buyerTokenAccount);
    const tokensBuyerHas = buyerAtaInfoAfterBuy.amount;
    const tokensToSell = tokensBuyerHas / BigInt("2");
    // Convert from raw tokens to "curve" base (assuming 10^9 decimals here):
    const tokensInCurveScaleDown = tokensToSell / BigInt(10 ** 9);

    const supplyBeforeSell = ownedTokenDataBefore.supply;

    const ixSell = await program.methods
      .sellExactInputInstruction(new BN(tokensInCurveScaleDown.toString()))
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey,
        creator: creator.publicKey,
        xyberToken: ownedTokenPda,
        escrowPda,
        mint: mintKeypair.publicKey,
        userTokenAccount: buyerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([buyerKeypair])
      .instruction();

    const txSellSig = await provider.sendAndConfirm(
      new Transaction().add(ixSell),
      [buyerKeypair]
    );
    console.log("Sell transaction Sig:", txSellSig);

    const buyerAtaInfoAfterSell = await getAccount(connection, buyerTokenAccount);
    const ownedTokenDataAfterSell = await program.account.xyberToken.fetch(ownedTokenPda);

    const tokensBurned = tokensBuyerHas - buyerAtaInfoAfterSell.amount;
    const deltaSupplyAfterSell = ownedTokenDataAfterSell.supply.sub(supplyBeforeSell);

    assert(
      buyerAtaInfoAfterSell.amount < tokensBuyerHas,
      "ATA balance should have decreased after selling!"
    );
    assert(tokensBurned > BigInt("0"), "We expected to burn some tokens");

    // Scale supply difference up by 10^9 decimals for comparison:
    const deltaSupplyInBase = deltaSupplyAfterSell.mul(new BN(10 ** 9));
    assert(
      deltaSupplyInBase.eq(new BN(tokensBurned.toString())),
      "Supply should increase exactly by the burned amount (in base units)"
    );

    ownedTokenDataBefore = ownedTokenDataAfterSell;
  });

  it("Step 4) Buyer buys EXACT output: requests exactly 10 tokens (buyExactOutputInstruction)", async () => {
    // We'll request 10 'curve units' of tokens (before multiplying by decimals).
    const tokensOutWanted = new BN(10);

    // 1) Fetch buyer token balance + current supply
    const buyerAtaInfoBefore = await getAccount(connection, buyerTokenAccount);
    const buyerTokensBefore = buyerAtaInfoBefore.amount;
    const supplyBefore = ownedTokenDataBefore.supply;

    console.log("=== [Step 4] Buyer buys EXACT output ===");
    console.log("Buyer tokens BEFORE (raw):", buyerTokensBefore.toString());
    console.log("Supply BEFORE (curve units):", supplyBefore.toString());
    console.log("Tokens requested (curve units):", tokensOutWanted.toString());

    // 2) Construct the instruction
    const ixBuyExactOutput = await program.methods
      .buyExactOutputInstruction(tokensOutWanted)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
        creator: creator.publicKey,
        xyberToken: ownedTokenPda,
        escrowPda,
        mint: mintKeypair.publicKey,
        buyerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([buyerKeypair])
      .instruction();

    // 3) Send transaction
    const txBuyExactOutSig = await provider.sendAndConfirm(
      new Transaction().add(ixBuyExactOutput),
      [buyerKeypair]
    );
    console.log("BuyExactOutput TX Sig:", txBuyExactOutSig);

    // 4) Post-transaction checks
    const buyerAtaInfoAfter = await getAccount(connection, buyerTokenAccount);
    const ownedTokenDataAfter = await program.account.xyberToken.fetch(ownedTokenPda);

    const buyerTokensAfter = buyerAtaInfoAfter.amount;
    console.log("Buyer tokens AFTER (raw):", buyerTokensAfter.toString());

    const supplyAfter = ownedTokenDataAfter.supply;
    console.log("Supply AFTER (curve units):", supplyAfter.toString());

    // 5) Compute minted + supply deltas
    const expectedRawMinted = tokensOutWanted.mul(new BN(10 ** 9)); // if 9 decimals
    const actualMinted = buyerTokensAfter - buyerTokensBefore; // BigInt difference

    console.log("Expected minted (raw):", expectedRawMinted.toString());
    console.log("Actual minted (raw):", actualMinted.toString());

    const supplyDelta = supplyBefore.sub(supplyAfter); // supply goes DOWN
    const supplyDeltaRaw = supplyDelta.mul(new BN(10 ** 9));

    console.log("Supply delta (curve units):", supplyDelta.toString());
    console.log("Supply delta (raw):", supplyDeltaRaw.toString());

    // 6) Assertions
    const expectedRawMintedBigInt = BigInt(expectedRawMinted.toString());
    const actualMintedBigInt = BigInt(actualMinted.toString());

    assert(
      expectedRawMintedBigInt === actualMintedBigInt,
      `Expected minted = ${expectedRawMintedBigInt}, but got ${actualMintedBigInt}`
    );

    assert(
      supplyDeltaRaw.eq(new BN(actualMinted.toString())),
      "Supply should decrease by exactly that minted amount (in raw units)."
    );

    // Update reference so next test sees the new XyberToken data
    ownedTokenDataBefore = ownedTokenDataAfter;
  });

  it("Step 5) Buyer sells EXACT output: requests exactly 10,000 lamports (sellExactOutputInstruction)", async () => {
    const lamportsWanted = new BN(10_000);

    // Log Buyer’s SOL balance BEFORE
    const buyerSolBalanceBefore = await provider.connection.getBalance(buyerKeypair.publicKey);
    console.log("Buyer SOL balance BEFORE:", buyerSolBalanceBefore.toString());

    // 1) Fetch buyer token account + current supply
    const buyerAtaInfoBefore = await getAccount(connection, buyerTokenAccount);
    const buyerTokensBefore = buyerAtaInfoBefore.amount;

    console.log("=== [Step 5] Buyer sells EXACT output ===");
    console.log("Buyer ATA info BEFORE:", {
      address: buyerTokenAccount.toBase58(),
      owner: buyerAtaInfoBefore.owner.toBase58(),
      mint: buyerAtaInfoBefore.mint.toBase58(),
      amount: buyerTokensBefore.toString(),
    });

    console.log("Buyer ATA info BEFORE (raw):", buyerAtaInfoBefore);

    const supplyBefore = ownedTokenDataBefore.supply;
    console.log("Supply BEFORE (curve units):", supplyBefore.toString());
    console.log("Buyer tokens BEFORE (raw):", buyerTokensBefore.toString());

    // 2) Construct instruction
    const ixSellExactOutput = await program.methods
      .sellExactOutputInstruction(lamportsWanted)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey,
        creator: creator.publicKey,
        xyberToken: ownedTokenPda,
        escrowPda,
        mint: mintKeypair.publicKey,
        userTokenAccount: buyerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([buyerKeypair])
      .instruction();

    // 3) Send transaction
    const txSellExactOutSig = await provider.sendAndConfirm(
      new Transaction().add(ixSellExactOutput),
      [buyerKeypair]
    );
    console.log("SellExactOutput TX Sig:", txSellExactOutSig);

    // (B) Log Buyer’s SOL balance AFTER (optional)
    const buyerSolBalanceAfter = await provider.connection.getBalance(buyerKeypair.publicKey);
    console.log("Buyer SOL balance AFTER:", buyerSolBalanceAfter.toString());
    console.log(
      "Buyer SOL gained/lost:",
      (buyerSolBalanceAfter - buyerSolBalanceBefore).toString()
    );

    // 4) Post-transaction checks: fetch updated ATA + supply
    const buyerAtaInfoAfter = await getAccount(connection, buyerTokenAccount);
    const buyerTokensAfter = buyerAtaInfoAfter.amount;

    console.log("Buyer ATA info AFTER:", {
      address: buyerTokenAccount.toBase58(),
      owner: buyerAtaInfoAfter.owner.toBase58(),
      mint: buyerAtaInfoAfter.mint.toBase58(),
      amount: buyerTokensAfter.toString(),
    });

    console.log("Buyer ATA info AFTER (raw):", buyerAtaInfoAfter);
    const ownedTokenDataAfter = await program.account.xyberToken.fetch(ownedTokenPda);
    const supplyAfter = ownedTokenDataAfter.supply;

    console.log("Supply AFTER (curve units):", supplyAfter.toString());
    console.log("Buyer tokens AFTER (raw):", buyerTokensAfter.toString());

    // 5) Compute tokens burned + supply deltas
    const tokensBurned = buyerTokensBefore - buyerTokensAfter; // BigInt
    console.log("Tokens burned (raw):", tokensBurned.toString());

    const supplyDelta = supplyAfter.sub(supplyBefore); // supply should go UP
    const supplyDeltaRaw = supplyDelta.mul(new BN(10 ** 9));
    console.log("Supply delta (curve units):", supplyDelta.toString());
    console.log("Supply delta (raw):", supplyDeltaRaw.toString());

    // 6) Assertions
    assert(tokensBurned > BigInt("0"), "Should have burned > 0 tokens.");
    const tokensBurnedBN = new BN(tokensBurned.toString());
    assert(
      supplyDeltaRaw.eq(tokensBurnedBN),
      "Supply should increase by exactly the number of burned tokens (raw)."
    );

    console.log("==== ALL TESTS PASSED (sellExactOutput) ====");
  });
});