import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
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

// Metaplex program ID
const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Load "creator" keypair from devnet
const keypath = path.join(process.env.HOME!, ".config", "solana", "devnet-owner.json");
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

// We'll deposit 0.001 SOL initially
const initialDepositLamports = new BN(100_000);

// Some metadata
const tokenName = "Hello World";
const tokenSymbol = "HWD";
const tokenUri =
  "https://ipfs.io/ipfs/QmVjBTRsbAM96BnNtZKrR8i3hGRbkjnQ3kugwXn6BVFu2k";

describe("Bonding Curve (Devnet): Buy(0.01 SOL) and Sell(0.01 SOL)", () => {
  it("Creates a new token, inits escrow, mints initial tokens, then buys and sells with lamports", async () => {
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

    // Creator's ATA
    const creatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      creator.publicKey
    );

    // Metaplex metadata PDA
    const [metadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METAPLEX_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      METAPLEX_PROGRAM_ID
    );

    console.log("=== PDAs ===");
    console.log("OwnedToken PDA:", ownedTokenPda.toBase58());
    console.log("Escrow PDA:", escrowPda.toBase58());
    console.log("Mint Pubkey:", mintKeypair.publicKey.toBase58());
    console.log("Creator ATA:", creatorTokenAccount.toBase58());

    //
    // 1) CREATE TOKEN INSTRUCTION
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
        systemProgram: SystemProgram.programId,
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
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    //
    // 3) MINT INITIAL TOKENS (0.001 SOL -> minted tokens for creator)
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
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    //
    // 4) SET METADATA
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
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    // Combine the first four instructions in one TX
    const tx1 = new Transaction()
      .add(ixCreate)
      .add(ixInitEscrow)
      .add(ixMintInitial)
      .add(ixMetadata);

    const tx1Sig = await provider.sendAndConfirm(tx1, [creator, mintKeypair]);
    console.log("Create + InitEscrow + MintInitial + Metadata TX:", tx1Sig);

    // Check OwnedToken & Creator's ATA
    let creatorAtaInfo = await getAccount(provider.connection, creatorTokenAccount);
    let ownedTokenData = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Creator ATA after initial mint:", creatorAtaInfo.amount.toString());
    console.log("OwnedToken.supply after initial mint:", ownedTokenData.supply.toString());

    // Just a sanity check: we expect some tokens minted to the creator
    assert(
      creatorAtaInfo.amount > BigInt(0),
      "Creator ATA should have received some tokens from initial deposit"
    );

    //
    // 5) BUY LOGIC: user deposits 0.01 SOL and receives tokens
    //
    const buyerKeypath = path.join(
      process.env.HOME!,
      ".config",
      "solana",
      "devnet-buyer.json"
    );
    const secretBuyerKeyArray = JSON.parse(fs.readFileSync(buyerKeypath, "utf-8"));
    const buyerKeypair = Keypair.fromSecretKey(new Uint8Array(secretBuyerKeyArray));

    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      buyerKeypair.publicKey
    );

    // We'll deposit 0.01 SOL from the buyer
    const buyLamports = new BN(100_000); // 0.0001 SOL
    const buyerAtaInfoBeforeBuy = await getAccount(provider.connection, buyerTokenAccount).catch(
      () => null
    );
    const buyerTokensBeforeBuy = buyerAtaInfoBeforeBuy ? buyerAtaInfoBeforeBuy.amount : 0;
    const supplyBeforeBuy = ownedTokenData.supply;

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
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .instruction();

    const txBuySig = await provider.sendAndConfirm(new Transaction().add(ixBuy), [buyerKeypair]);
    console.log("Buy transaction (0.01 SOL):", txBuySig);

    // Refresh buyer ATA & OwnedToken
    const buyerAtaInfoAfterBuy = await getAccount(provider.connection, buyerTokenAccount);
    const ownedTokenDataAfterBuy = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Buyer ATA balance after buy:", buyerAtaInfoAfterBuy.amount.toString());
    console.log("OwnedToken supply after buy:", ownedTokenDataAfterBuy.supply.toString());

    // Asserts:
    const deltaBuyer = buyerAtaInfoAfterBuy.amount - buyerTokensBeforeBuy;
    assert(
      deltaBuyer > 0,
      "Buyer ATA should have increased by some positive token amount"
    );

    const deltaSupply = supplyBeforeBuy.sub(ownedTokenDataAfterBuy.supply);
    assert(
      deltaSupply.eq(new anchor.BN(deltaBuyer.toString())),
      `Supply didn't decrease by minted amount. 
       Expected ${deltaBuyer}, got ${deltaSupply}`
    );

    //
    // 6) SELL LOGIC: user wants to withdraw 0.01 SOL
    //
    // Program calculates how many tokens must be burned to free exactly 0.01 SOL.
    const sellLamports = new BN(100_000); // 0.0001 SOL
    const buyerTokensBeforeSell = buyerAtaInfoAfterBuy.amount;
    const supplyBeforeSell = ownedTokenDataAfterBuy.supply;

    const ixSell = await program.methods
      .sellInstruction(sellLamports)
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

    const txSellSig = await provider.sendAndConfirm(new Transaction().add(ixSell), [buyerKeypair]);
    console.log("Sell transaction (get 0.01 SOL):", txSellSig);

    // Check final state
    const buyerAtaInfoAfterSell = await getAccount(provider.connection, buyerTokenAccount);
    const ownedTokenDataAfterSell = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Buyer ATA after sell:", buyerAtaInfoAfterSell.amount.toString());
    console.log("OwnedToken supply after sell:", ownedTokenDataAfterSell.supply.toString());

    // Asserts:
    const tokensBurned = buyerTokensBeforeSell - buyerAtaInfoAfterSell.amount; // how many were burned
    assert(
      tokensBurned > 0,
      "We expected to burn some tokens in order to withdraw 0.01 SOL"
    );

    const deltaSupplyAfterSell = ownedTokenDataAfterSell.supply.sub(supplyBeforeSell);
    assert(
      deltaSupplyAfterSell.eq(new anchor.BN(tokensBurned.toString())),
      `Supply didn't increase by the burned amount. 
       Expected supply to go up by ${tokensBurned}, got ${deltaSupplyAfterSell}`
    );

    console.log("==== TEST PASSED: CREATE, INIT ESCROW, MINT INITIAL, BUY(0.01 SOL), SELL(0.01 SOL) ====");
  });
});