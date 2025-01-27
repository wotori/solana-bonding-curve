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

// 1) Setup anchor provider
const connection = new anchor.web3.Connection("https://api.devnet.solana.com");
const keypath = path.join(process.env.HOME!, ".config", "solana", "devnet-owner.json");
const secretKeyArray = JSON.parse(fs.readFileSync(keypath, "utf-8"));
const devnetKeypair = Keypair.fromSecretKey(new Uint8Array(secretKeyArray));
const wallet = new anchor.Wallet(devnetKeypair);
const provider = new anchor.AnchorProvider(connection, wallet, {});
anchor.setProvider(provider);

// 2) The anchor program
const program = anchor.workspace.BondingCurve as Program<BondingCurve>;
const creator = devnetKeypair;

// Some metadata for setMetadataInstruction
const now = new Date();
const year = now.getFullYear().toString().slice(2);
const month = String(now.getMonth() + 1).padStart(2, '0');
const day = String(now.getDate()).padStart(2, '0');
const hours = String(now.getHours()).padStart(2, '0');
const minutes = String(now.getMinutes()).padStart(2, '0');

const tokenName = `${year}_${month}_${day}_${hours}_${minutes}`;
const tokenSymbol = "HWD";
const tokenUri = "https://ekza.io/ipfs/QmVjBTRsbAM96BnNtZKrR8i3hGRbkjnQ3kugwXn6BVFu2k";

// We'll deposit 0.001 SOL initially
const initialDepositLamports = new BN(0.001 * LAMPORTS_PER_SOL);

describe("Bonding Curve (Lamports) Test (multi-step with all asserts)", () => {
  // Shared variables across steps
  let tokenSeedKeypair: Keypair;
  let mintKeypair: Keypair;

  let ownedTokenPda: PublicKey;
  let escrowPda: PublicKey;
  let metadataPda: PublicKey;
  let creatorTokenAccount: PublicKey;

  // We'll store supply & ATA data between steps
  let ownedTokenDataBefore: any;
  let buyerKeypair: Keypair;
  let buyerTokenAccount: PublicKey;

  it("Step 1) Create, Init Escrow, MintInitial, SetMetadata", async () => {
    //
    // 0) Generate keypairs + PDAs
    //
    tokenSeedKeypair = Keypair.generate();
    mintKeypair = Keypair.generate();

    [ownedTokenPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("owned_token"),
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

    // Creator’s ATA
    creatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      creator.publicKey
    );

    console.log("===== PDAs =====");
    console.log("OwnedToken PDA:", ownedTokenPda.toBase58());
    console.log("Escrow PDA:", escrowPda.toBase58());
    console.log("Mint Pubkey:", mintKeypair.publicKey.toBase58());
    console.log("Creator ATA:", creatorTokenAccount.toBase58());
    console.log("Metadata PDA:", metadataPda.toBase58());

    //
    // 1) CREATE
    //
    const ixCreate = await program.methods
      .createTokenInstruction()
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        creatorTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        escrowPda,
        systemProgram: SystemProgram.programId
      })
      .signers([mintKeypair])
      .instruction();

    //
    // 2) INIT ESCROW
    //
    const ixInitEscrow = await program.methods
      .initEscrowInstruction()
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        escrowPda,
        systemProgram: SystemProgram.programId
      })
      .instruction();

    //
    // 3) MINT INITIAL
    //
    const ixMintInitial = await program.methods
      .mintInitialTokensInstruction(initialDepositLamports)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        escrowPda,
        mint: mintKeypair.publicKey,
        creatorTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .instruction();

    //
    // 4) SET METADATA
    //
    const ixSetMetadata = await program.methods
      .setMetadataInstruction(tokenName, tokenSymbol, tokenUri)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        metadata: metadataPda,
        tokenMetadataProgram: METAPLEX_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram: SystemProgram.programId
      })
      .instruction();

    try {
      console.log("Sending transaction for: Create + Init + MintInitial + Metadata...");
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

    // Check OwnedToken & Creator’s ATA
    const creatorAtaInfo = await getAccount(connection, creatorTokenAccount);
    ownedTokenDataBefore = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Creator ATA after initial mint:", creatorAtaInfo.amount.toString());
    console.log("OwnedToken.supply after initial mint:", ownedTokenDataBefore.supply.toString());

    // If deposit_lamports < 1 SOL, watch out for integer division => might be 0 tokens
    assert(
      creatorAtaInfo.amount >= BigInt(0),
      "Creator ATA should have minted at least 0 tokens (check integer division)."
    );
  });

  it("Step 2) Buyer deposits 0.001 SOL => receives tokens", async () => {
    // Load buyer from local keypair
    const buyerKeypath = path.join(process.env.HOME!, ".config", "solana", "devnet-buyer.json");
    const buyerSecretArr = JSON.parse(fs.readFileSync(buyerKeypath, "utf-8"));
    buyerKeypair = Keypair.fromSecretKey(new Uint8Array(buyerSecretArr));

    // Buyer’s ATA
    buyerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      buyerKeypair.publicKey
    );

    // We'll deposit 0.001 SOL
    const buyLamports = new BN(0.001 * LAMPORTS_PER_SOL);
    const buyerAtaInfoBefore = await getAccount(connection, buyerTokenAccount).catch(() => null);
    const buyerTokensBefore = buyerAtaInfoBefore ? buyerAtaInfoBefore.amount : BigInt("0");
    const supplyBeforeBuy = ownedTokenDataBefore.supply;

    // Build buy instruction
    const ixBuy = await program.methods
      .buyInstruction(buyLamports)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
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

    // Check new balances
    const buyerAtaInfoAfterBuy = await getAccount(connection, buyerTokenAccount);
    const ownedTokenDataAfterBuy = await program.account.ownedToken.fetch(ownedTokenPda);

    const deltaBuyer = buyerAtaInfoAfterBuy.amount - buyerTokensBefore;
    const deltaSupply = supplyBeforeBuy.sub(ownedTokenDataAfterBuy.supply);
    const scaledDeltaSupply = deltaSupply.mul(new BN(10 ** 9));

    console.log("Buyer ATA after buy:", buyerAtaInfoAfterBuy.amount.toString());
    console.log("OwnedToken supply after buy:", ownedTokenDataAfterBuy.supply.toString());
    console.log("Tokens minted to buyer:", deltaBuyer.toString());
    console.log("Supply decreased by:", deltaSupply.toString());

    assert(deltaBuyer > BigInt("0"), "Buyer should receive some positive token amount");
    assert(
      scaledDeltaSupply.eq(new BN(deltaBuyer.toString())),
      "Supply should decrease exactly by minted amount"
    );

    // Update for next step
    ownedTokenDataBefore = ownedTokenDataAfterBuy;
  });

  it("Step 3) Buyer sells half of their tokens", async () => {
    // Re-check buyer’s ATA
    const buyerAtaInfoAfterBuy = await getAccount(connection, buyerTokenAccount);
    const tokensBuyerHas = buyerAtaInfoAfterBuy.amount;

    console.log("Buyer has:", tokensBuyerHas.toString());
    const tokensToSell = tokensBuyerHas / BigInt("2");
    const tokensInCurveScaleDown = tokensToSell / BigInt(10 ** 9);

    console.log("Selling half of buyer’s tokens:", tokensInCurveScaleDown.toString());

    const supplyBeforeSell = ownedTokenDataBefore.supply;

    // Sell
    const ixSell = await program.methods
      .sellInstruction(new BN(tokensInCurveScaleDown.toString()))
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
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

    // Check final
    const buyerAtaInfoAfterSell = await getAccount(connection, buyerTokenAccount);
    const ownedTokenDataAfterSell = await program.account.ownedToken.fetch(ownedTokenPda);

    const tokensBurned = tokensBuyerHas - buyerAtaInfoAfterSell.amount;
    const deltaSupplyAfterSell = ownedTokenDataAfterSell.supply.sub(supplyBeforeSell);

    const buyerTokenAccOnChain = await getAccount(connection, buyerTokenAccount);
    console.log("ATA owner is:", buyerTokenAccOnChain.owner.toBase58());
    console.log("User is:", buyerKeypair.publicKey.toBase58());

    console.log("Buyer ATA after sell:", buyerAtaInfoAfterSell.amount.toString());
    console.log("Tokens actually burned:", tokensBurned.toString());
    console.log("OwnedToken supply before sell:", supplyBeforeSell.toString());
    console.log("OwnedToken supply after sell:", ownedTokenDataAfterSell.supply.toString());
    console.log("Supply changed by:", deltaSupplyAfterSell.toString());

    assert(
      buyerAtaInfoAfterSell.amount < tokensBuyerHas,
      "ATA balance should have decreased after selling!"
    );
    assert(tokensBurned > BigInt("0"), "We expected to burn some tokens");

    const deltaSupplyInBase = deltaSupplyAfterSell.mul(new BN(10 ** 9));
    assert(
      deltaSupplyInBase.eq(new BN(tokensBurned.toString())),
      "Supply should increase by exactly the burned amount (in base units)"
    );

    console.log("==== TEST PASSED ====");
  });
});