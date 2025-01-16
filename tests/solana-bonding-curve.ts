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

const METAPLEX_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

const keypath = path.join(process.env.HOME, ".config", "solana", "devnet-owner.json");
const secretKeyArray = JSON.parse(fs.readFileSync(keypath, "utf-8"));
const devnetKeypair = Keypair.fromSecretKey(new Uint8Array(secretKeyArray));

const connection = new anchor.web3.Connection("https://api.devnet.solana.com");
const wallet = new anchor.Wallet(devnetKeypair);
const provider = new anchor.AnchorProvider(connection, wallet, {});
anchor.setProvider(provider);

const program = anchor.workspace.BondingCurve as Program<BondingCurve>;
const creator = devnetKeypair;

const decimals = 9;

// Convert BigInt to string before passing to BN
const totalSupply = new BN((BigInt(1_000_000_000) * BigInt(10 ** decimals)).toString());
const initialMintAmount = new BN((BigInt(500_000_000) * BigInt(10 ** decimals)).toString());
const tokenName = "Hello World";
const tokenSymbol = "HWD";
const tokenUri = "https://ipfs.io/ipfs/QmVjBTRsbAM96BnNtZKrR8i3hGRbkjnQ3kugwXn6BVFu2k";

describe("Bonding Curve (Devnet): Multiple Tokens with Unique Seeds", () => {
  it("Creates a new token mint, mints supply, and sets metadata", async () => {
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

    const ixCreate = await program.methods
      .createTokenInstruction(totalSupply, initialMintAmount)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        creatorTokenAccount,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([mintKeypair])
      .instruction();

    const ixMetadata = await program.methods
      .setMetadataInstruction(tokenName, tokenSymbol, tokenUri)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creator.publicKey,
        ownedToken: ownedTokenPda,
        mint: mintKeypair.publicKey,
        metadata: metadataPda,
        tokenMetadataProgram: METAPLEX_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .instruction();

    const tx = new Transaction().add(ixCreate).add(ixMetadata);
    const txSignature = await provider.sendAndConfirm(tx, [creator, mintKeypair]);
    console.log("Transaction signature:", txSignature);

    const creatorAtaInfo = await getAccount(provider.connection, creatorTokenAccount);
    assert.strictEqual(
      creatorAtaInfo.amount.toString(),
      initialMintAmount.toString(),
    );

    console.log("User's ATA amount:", creatorAtaInfo.amount.toString());

    const ownedTokenData = await program.account.ownedToken.fetch(ownedTokenPda);
    assert.strictEqual(
      ownedTokenData.supply.toString(),
      totalSupply.toString(),
    );
    console.log("OwnedToken supply:", ownedTokenData.supply.toString());
  });
});