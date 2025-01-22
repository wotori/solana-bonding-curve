import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import fs from "fs";
import path from "path";
import { assert } from "chai";
import { BondingCurve } from "../target/types/bonding_curve";

// Metaplex program ID:
const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Load "creator" keypair
const keypath = path.join(
  process.env.HOME!,
  ".config",
  "solana",
  "devnet-owner.json"
);
const secretKeyArray = JSON.parse(fs.readFileSync(keypath, "utf-8"));
const devnetKeypair = Keypair.fromSecretKey(new Uint8Array(secretKeyArray));

// Setup Anchor provider
const connection = new anchor.web3.Connection("https://api.devnet.solana.com");
const wallet = new anchor.Wallet(devnetKeypair);
const provider = new anchor.AnchorProvider(connection, wallet, {});
anchor.setProvider(provider);

// Anchor program
const program = anchor.workspace.BondingCurve as Program<BondingCurve>;
const creator = devnetKeypair;

// Token parameters
const decimals = 9;
const totalSupply = new BN(
  (BigInt(1_000_000_000) * BigInt(10 ** decimals)).toString()
);
const initialMintAmount = new BN(
  (BigInt(500_000_000) * BigInt(10 ** decimals)).toString()
);
const tokenName = "Hello World";
const tokenSymbol = "HWD";
const tokenUri =
  "https://ipfs.io/ipfs/QmVjBTRsbAM96BnNtZKrR8i3hGRbkjnQ3kugwXn6BVFu2k";

// Price = 0.001 SOL = 1_000_000 lamports
const priceLamports = new BN(1_000_000);

describe("Bonding Curve (Devnet): Create, Buy, Sell", () => {
  it("Creates a new token, initializes escrow, sets metadata, then performs buy/sell", async () => {
    //
    // 0) Generate keypairs + PDAs
    //
    const tokenSeedKeypair = Keypair.generate();
    const mintKeypair = Keypair.generate();

    // PDAs
    const [ownedTokenPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("owned_token"),
        creator.publicKey.toBuffer(),
        tokenSeedKeypair.publicKey.toBuffer(),
      ],
      program.programId
    );
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        creator.publicKey.toBuffer(),
        tokenSeedKeypair.publicKey.toBuffer(),
      ],
      program.programId
    );
    const creatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      creator.publicKey
    );
    const [metadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METAPLEX_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      METAPLEX_PROGRAM_ID
    );

    console.log(
      "JS associatedTokenProgram in test:",
      ASSOCIATED_TOKEN_PROGRAM_ID.toBase58()
    );

    //
    // 1) CREATE TOKEN INSTRUCTION
    //
    const ixCreate = await program.methods
      .createTokenInstruction(totalSupply, initialMintAmount, priceLamports)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        creatorTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([mintKeypair])
      .instruction();

    //
    // 2) INIT ESCROW INSTRUCTION
    //
    const ixInitEscrow = await program.methods
      .initEscrowInstruction()
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        escrowPda,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    //
    // 3) SET METADATA INSTRUCTION
    //
    const ixMetadata = await program.methods
      .setMetadataInstruction(tokenName, tokenSymbol, tokenUri)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        metadata: metadataPda,
        tokenMetadataProgram: METAPLEX_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    //
    // COMBINE ALL 3 INSTRUCTIONS INTO 1 TRANSACTION
    //
    const tx1 = new Transaction()
      .add(ixCreate)
      .add(ixInitEscrow)
      .add(ixMetadata);

    const tx1Sig = await provider.sendAndConfirm(tx1, [creator, mintKeypair]);
    console.log("Create + InitEscrow + Metadata Transaction:", tx1Sig);

    //
    // Now we check OwnedToken & Creator ATA
    //
    const creatorAtaInfo = await getAccount(provider.connection, creatorTokenAccount);
    const ownedTokenDataBefore = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Creator ATA amount:", creatorAtaInfo.amount.toString());
    console.log("OwnedToken supply:", ownedTokenDataBefore.supply.toString());
    console.log("OwnedToken price:", ownedTokenDataBefore.priceLamports.toString());
    console.log("OwnedToken escrow_pda:", ownedTokenDataBefore.escrowPda.toBase58());

    // Assertions
    assert.strictEqual(
      creatorAtaInfo.amount.toString(),
      initialMintAmount.toString(),
      "Creator should have initial mint amount"
    );
    assert.strictEqual(
      ownedTokenDataBefore.supply.toString(),
      totalSupply.toString(),
      "OwnedToken.supply mismatch"
    );
    assert.strictEqual(
      ownedTokenDataBefore.priceLamports.toString(),
      priceLamports.toString(),
      "OwnedToken.price mismatch"
    );
    assert.strictEqual(
      ownedTokenDataBefore.escrowPda.toBase58(),
      escrowPda.toBase58(),
      "EscrowPDA mismatch in OwnedToken"
    );

    //
    // 4) BUY LOGIC
    //
    const buyerKeypath = path.join(
      process.env.HOME!,
      ".config",
      "solana",
      "devnet-buyer.json"
    );
    const secretBuyerKeyArray = JSON.parse(fs.readFileSync(buyerKeypath, "utf-8"));
    const buyerKeypair = Keypair.fromSecretKey(new Uint8Array(secretBuyerKeyArray));

    // Buyer ATA
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      buyerKeypair.publicKey
    );

    const buyAmount = new BN((BigInt(100) * BigInt(10 ** decimals)).toString());
    const ixBuy = await program.methods
      .buyInstruction(buyAmount)
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
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .instruction();

    const txBuySig = await provider.sendAndConfirm(
      new Transaction().add(ixBuy),
      [buyerKeypair]
    );
    console.log("Buy transaction:", txBuySig);

    // Check buyer's ATA & OwnedToken
    const buyerAtaInfoAfterBuy = await getAccount(provider.connection, buyerTokenAccount);
    const ownedTokenDataAfterBuy = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Buyer ATA after buy:", buyerAtaInfoAfterBuy.amount.toString());
    console.log("OwnedToken supply after buy:", ownedTokenDataAfterBuy.supply.toString());

    const expectedSupplyAfterBuy = totalSupply.sub(buyAmount);
    assert.strictEqual(
      ownedTokenDataAfterBuy.supply.toString(),
      expectedSupplyAfterBuy.toString(),
      "Supply did not decrease as expected"
    );
    assert.strictEqual(
      buyerAtaInfoAfterBuy.amount.toString(),
      buyAmount.toString(),
      "Buyer token account mismatch"
    );

    //
    // 5) SELL LOGIC
    //
    const sellAmount = new BN((BigInt(50) * BigInt(10 ** decimals)).toString());
    const ixSell = await program.methods
      .sellInstruction(sellAmount)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        escrowPda,
        mint: mintKeypair.publicKey,
        userTokenAccount: buyerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .instruction();

    const txSellSig = await provider.sendAndConfirm(
      new Transaction().add(ixSell),
      [buyerKeypair]
    );
    console.log("Sell transaction:", txSellSig);

    // Check updated supply & buyer's ATA
    const buyerAtaInfoAfterSell = await getAccount(provider.connection, buyerTokenAccount);
    const ownedTokenDataAfterSell = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Buyer ATA after sell:", buyerAtaInfoAfterSell.amount.toString());
    console.log("OwnedToken supply after sell:", ownedTokenDataAfterSell.supply.toString());

    // Supply should be totalSupply - buyAmount + sellAmount
    const expectedSupplyAfterSell = expectedSupplyAfterBuy.add(sellAmount);
    assert.strictEqual(
      ownedTokenDataAfterSell.supply.toString(),
      expectedSupplyAfterSell.toString(),
      "Supply did not increment as expected after sell"
    );

    // Buyer should have (buyAmount - sellAmount) tokens left
    const expectedBuyerTokens = buyAmount.sub(sellAmount);
    assert.strictEqual(
      buyerAtaInfoAfterSell.amount.toString(),
      expectedBuyerTokens.toString(),
      "Buyer token account not decremented by sellAmount"
    );

    console.log("==== TEST PASSED: CREATE, INIT ESCROW, METADATA, BUY, SELL ====");
  });
});