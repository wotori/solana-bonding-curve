import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert } from "chai";

import { BondingCurve } from "../target/types/bonding_curve";

const METAPLEX_TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("bonding_curve", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const program = anchor.workspace.BondingCurve as Program<BondingCurve>;

  // We'll generate a new Keypair to be the owner.
  const owner = Keypair.generate();

  // We'll store PDAs or addresses here once created.
  let ownedTokenPda: PublicKey;
  let mintKey: PublicKey;
  let ownerTokenAccount: PublicKey;

  // Example test parameters
  const tokenName = "My Test Token";
  const tokenSymbol = "TTT";
  const totalSupply = new BN(1_000_000_000); // 1,000,000,000 tokens (9 decimals => 1,000 real tokens)
  const initialBuyAmount = new BN(500_000_000); // half minted to owner
  const initialBuyPrice = new BN(1000);
  const metadataUri = "https://example.com/metadata.json";

  before(async () => {
    // Airdrop SOL to the owner so they can pay for transactions
    console.log("Requesting 2 SOL airdrop for owner:", owner.publicKey.toBase58());
    const sig = await provider.connection.requestAirdrop(
      owner.publicKey,
      2_000_000_000 // 2 SOL
    );
    await provider.connection.confirmTransaction(sig);
    console.log("Airdrop confirmed. Owner now has SOL for test transactions.\n");
  });

  it("Launches a new token with metadata", async () => {
    // 1. Generate PDA for OwnedToken
    const ownedTokenKeypair = Keypair.generate();
    ownedTokenPda = ownedTokenKeypair.publicKey;
    console.log("Owned Token PDA:", ownedTokenPda.toBase58());

    // 2. Create Mint keypair
    const mintKeypair = Keypair.generate();
    mintKey = mintKeypair.publicKey;
    console.log("Mint Key:", mintKey.toBase58());

    // 3. Derive the associated token account for the owner
    ownerTokenAccount = await getAssociatedTokenAddress(mintKey, owner.publicKey);
    console.log("Owner Token Account (ATA):", ownerTokenAccount.toBase58());

    // 4. Build the params for token creation
    const params = {
      tokenName,
      ticker: tokenSymbol,
      metadataUri,
      mediaUrl: "",
      description: "",
      website: null,
      twitter: null,
      telegram: null,
      supply: new BN(totalSupply),
      initialBuyAmount: new BN(initialBuyAmount),
      initialBuyPrice: initialBuyPrice.toNumber(),
      targetChains: [{ solana: {} }, { base: {} }],
      bondingCurveCoefficients: {
        coefficientA: new anchor.BN(10),
        coefficientB: new anchor.BN(5),
        coefficientC: new anchor.BN(2),
      },
    };
    console.log("Launch Token Params:", params, "\n");

    // 5. Call the 'launch_token' instruction
    await program.methods
      .launchToken(params)
      .accounts({
        ownedToken: ownedTokenPda,
        owner: owner.publicKey,
        mint: mintKey,
        ownerTokenAccount,
        tokenMetadataProgram: METAPLEX_TOKEN_METADATA_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([owner, ownedTokenKeypair, mintKeypair])
      .rpc();

    console.log("launch_token instruction executed successfully.\n");

    // 6. Verify minted tokens
    const ownerAtaInfo = await getAccount(provider.connection, ownerTokenAccount);
    console.log("Owner ATA info amount:", Number(ownerAtaInfo.amount));
    assert.strictEqual(Number(ownerAtaInfo.amount), initialBuyAmount.toNumber());

    // 7. Fetch OwnedToken account data to confirm stored fields
    const ownedTokenAccountData = await program.account.ownedToken.fetch(ownedTokenPda);
    console.log("OwnedToken Account Data:\n", ownedTokenAccountData);

    // Verify fields
    assert.strictEqual(ownedTokenAccountData.tokenName, tokenName);
    assert.strictEqual(ownedTokenAccountData.ticker, tokenSymbol);
    assert.strictEqual(
      ownedTokenAccountData.supply.toNumber(),
      totalSupply.toNumber()
    );
    assert.strictEqual(
      ownedTokenAccountData.initialBuyAmount.toNumber(),
      initialBuyAmount.toNumber()
    );
    assert.strictEqual(
      ownedTokenAccountData.initialBuyPrice,
      initialBuyPrice.toNumber()
    );

    // Confirm targetChains: ["SOL", "BASE"]
    assert.deepEqual(
      ownedTokenAccountData.targetChains.map((chain) => {
        if (chain.solana) return "SOL";
        if (chain.base) return "BASE";
        throw new Error("Unexpected target chain enum value");
      }),
      ["SOL", "BASE"]
    );

    // Confirm bonding curve coefficients: [10, 5, 2]
    assert.deepEqual(
      [
        ownedTokenAccountData.bondingCurveCoefficients.coefficientA.toNumber(),
        ownedTokenAccountData.bondingCurveCoefficients.coefficientB.toNumber(),
        ownedTokenAccountData.bondingCurveCoefficients.coefficientC.toNumber(),
      ],
      [10, 5, 2]
    );

    assert.strictEqual(ownedTokenAccountData.metadataUri, metadataUri);

    console.log("Token launched successfully. Mint:", mintKey.toBase58());
  });
});
