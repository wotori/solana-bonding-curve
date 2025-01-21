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

const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Load a keypair to use as the "creator"
const keypath = path.join(process.env.HOME!, ".config", "solana", "devnet-owner.json");
const secretKeyArray = JSON.parse(fs.readFileSync(keypath, "utf-8"));
const devnetKeypair = Keypair.fromSecretKey(new Uint8Array(secretKeyArray));

const connection = new anchor.web3.Connection("https://api.devnet.solana.com");
const wallet = new anchor.Wallet(devnetKeypair);
const provider = new anchor.AnchorProvider(connection, wallet, {});
anchor.setProvider(provider);

const program = anchor.workspace.BondingCurve as Program<BondingCurve>;
const creator = devnetKeypair;

const decimals = 9;
const totalSupply = new BN(
  (BigInt(1_000_000_000) * BigInt(10 ** decimals)).toString()
);
const initialMintAmount = new BN(
  (BigInt(500_000_000) * BigInt(10 ** decimals)).toString()
);
const tokenName = "Hello World";
const tokenSymbol = "HWD";
const tokenUri = "https://ipfs.io/ipfs/QmVjBTRsbAM96BnNtZKrR8i3hGRbkjnQ3kugwXn6BVFu2k";

// We set a naive price: 0.001 SOL (1_000_000 lamports)
const priceLamports = new BN(1_000_000);

describe("Bonding Curve (Devnet): Create, Buy, Sell", () => {
  it("Creates a new token, mints supply, sets metadata, then performs buy/sell", async () => {
    //
    // 0) GENERATE SEEDS/PDA
    //
    const tokenSeedKeypair = Keypair.generate();
    const mintKeypair = Keypair.generate();

    const [ownedTokenPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("owned_token"),
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


    //
    // 1) CREATE TOKEN
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
    // 2) CREATE METADATA
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

    const tx = new Transaction().add(ixCreate).add(ixMetadata);
    const txSignature = await provider.sendAndConfirm(tx, [creator, mintKeypair]);
    console.log("Create + Metadata Transaction:", txSignature);

    // Fetch ATA info & OwnedToken data
    const creatorAtaInfo = await getAccount(provider.connection, creatorTokenAccount);
    const ownedTokenDataBefore = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Creator ATA amount:", creatorAtaInfo.amount.toString());
    console.log("OwnedToken supply:", ownedTokenDataBefore.supply.toString());
    console.log("OwnedToken price:", ownedTokenDataBefore.priceLamports.toString());

    // Assertions
    assert.strictEqual(creatorAtaInfo.amount.toString(), initialMintAmount.toString());
    assert.strictEqual(ownedTokenDataBefore.supply.toString(), totalSupply.toString());
    assert.strictEqual(ownedTokenDataBefore.priceLamports.toString(), priceLamports.toString());

    //
    // 3) BUY LOGIC
    //

    const buyerKeypath = path.join(process.env.HOME!, ".config", "solana", "devnet-buyer.json");
    const secretBuyerKeyArray = JSON.parse(fs.readFileSync(buyerKeypath, "utf-8"));
    const buyerKeypair = Keypair.fromSecretKey(new Uint8Array(secretBuyerKeyArray));

    // The buyer’s associated token account:
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      buyerKeypair.publicKey
    );

    // Let's define a buy amount of e.g. 100 tokens (in base units, i.e. 100 * 10^decimals)
    const buyAmount = new BN((BigInt(100) * BigInt(10 ** decimals)).toString());
    const ixBuy = await program.methods
      .buyInstruction(buyAmount)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        buyerTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .instruction();

    const txBuySig = await provider.sendAndConfirm(new Transaction().add(ixBuy), [buyerKeypair]);
    console.log("Buy transaction:", txBuySig);

    // Fetch updated ATA & OwnedToken data
    const buyerAtaInfoAfterBuy = await getAccount(provider.connection, buyerTokenAccount);
    const ownedTokenDataAfterBuy = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Buyer ATA after buy:", buyerAtaInfoAfterBuy.amount.toString());
    console.log("OwnedToken supply after buy:", ownedTokenDataAfterBuy.supply.toString());

    // Supply should have decreased by `buyAmount`
    const expectedSupplyAfterBuy = totalSupply.sub(buyAmount);
    assert.strictEqual(
      ownedTokenDataAfterBuy.supply.toString(),
      expectedSupplyAfterBuy.toString(),
      "Supply did not decrease as expected"
    );

    // The buyer’s ATA should have exactly `buyAmount`
    assert.strictEqual(
      buyerAtaInfoAfterBuy.amount.toString(),
      buyAmount.toString(),
      "Buyer token account mismatch"
    );

    //
    // 4) SELL LOGIC
    //
    // Let’s sell half of what was bought: 50 tokens
    const sellAmount = new BN((BigInt(50) * BigInt(10 ** decimals)).toString());
    const ixSell = await program.methods
      .sellInstruction(sellAmount)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        userTokenAccount: buyerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .instruction();

    const txSellSig = await provider.sendAndConfirm(new Transaction().add(ixSell), [buyerKeypair]);
    console.log("Sell transaction:", txSellSig);

    // Check updated supply, buyer’s ATA
    const buyerAtaInfoAfterSell = await getAccount(provider.connection, buyerTokenAccount);
    const ownedTokenDataAfterSell = await program.account.ownedToken.fetch(ownedTokenPda);

    console.log("Buyer ATA after sell:", buyerAtaInfoAfterSell.amount.toString());
    console.log("OwnedToken supply after sell:", ownedTokenDataAfterSell.supply.toString());

    // Supply should be originalSupply - buyAmount + sellAmount
    // i.e. (1_000_000_000 - 100) + 50 = 999_999_950
    const expectedSupplyAfterSell = expectedSupplyAfterBuy.add(sellAmount);
    assert.strictEqual(
      ownedTokenDataAfterSell.supply.toString(),
      expectedSupplyAfterSell.toString(),
      "Supply did not increment as expected after sell"
    );

    // Buyer should now have buyAmount - sellAmount = 50 tokens
    const expectedBuyerTokens = buyAmount.sub(sellAmount);
    assert.strictEqual(
      buyerAtaInfoAfterSell.amount.toString(),
      expectedBuyerTokens.toString(),
      "Buyer token account not decremented by sellAmount"
    );

    console.log("==== TEST PASSED: CREATE, BUY, SELL ====");
  });
});