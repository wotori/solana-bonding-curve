import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  getAssociatedTokenAddress,
  getAccount, // for checking balances
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import path from "path";
import { BondingCurve } from "../target/types/bonding_curve";

// Metaplex Metadata program ID
const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Token factory program ID
const TOKEN_FACTORY_PROGRAM_ID = new PublicKey(
  "TF5AoQEG87r1gpWsNzADMxYean6tfdGVUouQQ5LbYPP"
);

// Adjust these paths/keys to suit environment:
const DEVNET_URL = "https://api.devnet.solana.com";
const CREATOR_KEYPAIR_PATH = path.join(
  process.env.HOME!,
  ".config",
  "solana",
  "devnet-owner.json"
);
const BUYER_KEYPAIR_PATH = path.join(
  process.env.HOME!,
  ".config",
  "solana",
  "devnet-buyer.json"
);

// Base (payment) token mint
const PAYMENT_MINT_PUBKEY = new PublicKey(
  "6WQQPDXsBxkgMwuApkXbV2bUf3CZAJmGBDqk62aMpmKR"
);

// Test constants
const TOTAL_TOKENS = new BN("1073000191");
const DECIMALS = 9;
const LAMPORTS_PER_TOKEN = 10 ** DECIMALS;
const BONDING_K_VIRTUAL = new BN("32190005730").mul(new BN(LAMPORTS_PER_TOKEN));
const VIRTUAL_POOL_OFFSET = new BN(30 * LAMPORTS_PER_TOKEN);
const GRADUATE_DOLLARS_AMOUNT = 10000;
const XBT_PRICE_DOLLARS = 0.05;

// Metadata parameters for the project token
const tokenName = "TEST_TOKEN";
const tokenSymbol = "TST";
const tokenUri =
  "https://ekza.io/ipfs/QmVjBTRsbAM96BnNtZKrR8i3hGRbkjnQ3kugwXn6BVFu2k";

describe("Bonding Curve Program (Token Init + Buyer/Seller Flow)", () => {
  // 1) Setup & Keypairs
  const connection = new anchor.web3.Connection(DEVNET_URL, {
    commitment: "processed",
  });

  // Creator (admin) keypair
  const creatorSecret = require(CREATOR_KEYPAIR_PATH);
  const creatorKeypair = Keypair.fromSecretKey(new Uint8Array(creatorSecret));

  // Buyer keypair
  const buyerSecret = require(BUYER_KEYPAIR_PATH);
  const buyerKeypair = Keypair.fromSecretKey(new Uint8Array(buyerSecret));

  // Anchor provider, program, etc.
  const wallet = new anchor.Wallet(creatorKeypair);
  const provider = new anchor.AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  const program = anchor.workspace.BondingCurve as Program<BondingCurve>;

  // 2) Variables / PDAs
  let tokenSeedKeypair: Keypair;
  let xyberTokenPda: PublicKey;
  let mintPda: PublicKey;
  let vaultTokenAccount: PublicKey;
  let creatorTokenAccount: PublicKey;
  let escrowTokenAccount: PublicKey;
  let metadataPda: PublicKey;

  // Derive PDAs in before() hook
  before("Derive all PDAs", async () => {
    tokenSeedKeypair = Keypair.generate();

    // XyberToken PDA
    [xyberTokenPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("xyber_token"),
        creatorKeypair.publicKey.toBuffer(),
        tokenSeedKeypair.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Mint PDA (token_factory seeds)
    [mintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("MINT"), tokenSeedKeypair.publicKey.toBuffer()],
      TOKEN_FACTORY_PROGRAM_ID
    );

    // Escrow ATA for base tokens
    escrowTokenAccount = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      xyberTokenPda,
      true
    );

    // Creator’s ATA for the project token
    creatorTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      creatorKeypair.publicKey
    );

    // Metadata PDA for the minted token
    [metadataPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METAPLEX_PROGRAM_ID.toBuffer(),
        mintPda.toBuffer(),
      ],
      METAPLEX_PROGRAM_ID
    );

    // Vault ATA (owned by xyberTokenPda)
    vaultTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      xyberTokenPda,
      true
    );
  });

  // ------------------------------------------------------------------
  // 3) Tests
  // ------------------------------------------------------------------

  // 3.1) init_core_instruction
  it("1.1) init_core_instruction", async () => {
    console.log("----- Step 1: init_core_instruction -----");
    const createTokenParams = {
      tokenSupply: TOTAL_TOKENS,
      tokenGradThrUsd: 999, // example threshold
      bondingCurve: {
        aTotalTokens: TOTAL_TOKENS,
        kVirtualPoolOffset: BONDING_K_VIRTUAL,
        cBondingScaleFactor: VIRTUAL_POOL_OFFSET,
        xTotalBaseDeposit: new BN(0),
      },
      acceptedBaseMint: PAYMENT_MINT_PUBKEY,
      admin: creatorKeypair.publicKey,
      graduateDollarsAmount: GRADUATE_DOLLARS_AMOUNT,
    };

    const ixCore = await program.methods
      .initCoreInstruction(createTokenParams)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creatorKeypair.publicKey,
        xyberToken: xyberTokenPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([creatorKeypair])
      .instruction();

    const txCore = new Transaction().add(ixCore);

    console.log("Sending init_core_instruction transaction...");
    const sigCore = await provider.sendAndConfirm(txCore, [creatorKeypair]);
    console.log("init_core_instruction SUCCESS, signature =", sigCore);

    // Optionally fetch the state
    const xyberState = await program.account.xyberToken.fetch(xyberTokenPda);
    console.log("After init_core, XYBER state:", xyberState);
  });

  // 3.2) init_and_mint_full_supply_instruction
  it("1.2) init_and_mint_full_supply_instruction", async () => {
    console.log("----- Step 2: init_and_mint_full_supply_instruction -----");

    const ixMintFullSupply = await program.methods
      .initAndMintFullSupplyInstruction(tokenName, tokenSymbol, tokenUri)
      .accounts({
        xyberToken: xyberTokenPda,
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creatorKeypair.publicKey,
        mint: mintPda,
        vaultTokenAccount: vaultTokenAccount,
        metadataAccount: metadataPda,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: METAPLEX_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenFactoryProgram: TOKEN_FACTORY_PROGRAM_ID,
      })
      .instruction();

    const txMintFullSupply = new Transaction().add(ixMintFullSupply);

    console.log("Sending init_and_mint_full_supply_instruction transaction...");
    const sigMintFullSupply = await provider.sendAndConfirm(txMintFullSupply, [
      creatorKeypair,
    ]);
    console.log(
      "init_and_mint_full_supply_instruction SUCCESS, signature =",
      sigMintFullSupply
    );

    const xyberState = await program.account.xyberToken.fetch(xyberTokenPda);
    console.log("After init_and_mint_full_supply, XYBER state:", xyberState);
  });

  // 3.3) mint_initial_tokens_instruction
  it("1.3) mint_initial_tokens_instruction", async () => {
    console.log("----- Step 3: mint_initial_tokens_instruction -----");

    // Example deposit: 1 lamport in the base token
    // (Change to 0.001 * LAMPORTS_PER_TOKEN if needed)
    const depositLamports = new BN(1);

    // Creator's payment account for base token
    const creatorPaymentAccount = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      creatorKeypair.publicKey
    );

    const ixMintInitial = await program.methods
      .mintInitialTokensInstruction(depositLamports)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        creator: creatorKeypair.publicKey,
        xyberToken: xyberTokenPda,
        escrowTokenAccount: escrowTokenAccount,
        paymentMint: PAYMENT_MINT_PUBKEY,
        creatorPaymentAccount: creatorPaymentAccount,
        mint: mintPda,
        vaultTokenAccount: vaultTokenAccount,
        creatorTokenAccount: creatorTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([creatorKeypair])
      .instruction();

    const txMintInitial = new Transaction().add(ixMintInitial);

    console.log("Sending mint_initial_tokens_instruction transaction...");
    const sigMintInitial = await provider.sendAndConfirm(txMintInitial, [
      creatorKeypair,
    ]);
    console.log(
      "mint_initial_tokens_instruction SUCCESS, signature =",
      sigMintInitial
    );

    const xyberState = await program.account.xyberToken.fetch(xyberTokenPda);
    console.log("After mint_initial_tokens, XYBER state:", xyberState);
  });

  // 3.4) Buyer buys token with exact base input
  it("1.4) Buyer buys token with exact base input (buy_exact_input_instruction)", async () => {
    const vaultInfo = await getAccount(connection, vaultTokenAccount);
    console.log("Vault token balance (raw) =", vaultInfo.amount.toString());

    // 1) Derive buyer's ATA for the project token
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      buyerKeypair.publicKey
    );

    // 2) Derive buyer's ATA for the payment token
    const buyerPaymentAccount = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      buyerKeypair.publicKey
    );

    // 3) Buyer pays
    const baseIn = new BN(0.001 * LAMPORTS_PER_TOKEN);

    // 4) Call the instruction
    await program.methods
      .buyExactInputInstruction(baseIn)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
        creator: creatorKeypair.publicKey,
        xyberToken: xyberTokenPda,
        escrowTokenAccount: escrowTokenAccount,
        paymentMint: PAYMENT_MINT_PUBKEY,
        mint: mintPda,
        vaultTokenAccount: vaultTokenAccount,
        buyerTokenAccount: buyerTokenAccount,
        buyerPaymentAccount: buyerPaymentAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .rpc();

    // 5) Check buyer's project-token balance
    const buyerAtaInfo = await getAccount(connection, buyerTokenAccount);
    console.log(
      "Buyer project-token balance =>",
      buyerAtaInfo.amount.toString()
    );

    // 6) Confirm buyer has some nonzero amount
    assert(
      buyerAtaInfo.amount > BigInt(0),
      "Buyer should have received some project tokens."
    );
  });

  // 3.5) Buyer sells token with exact input
  it("1.5) Buyer sells token with exact input (sell_exact_input_instruction)", async () => {
    // 1) Derive buyer's ATA for project token & payment token
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      buyerKeypair.publicKey
    );
    const buyerPaymentAccount = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      buyerKeypair.publicKey
    );

    // 2) Let's sell half of the tokens buyer has (in unscaled "curve units")
    const buyerAtaInfo = await getAccount(connection, buyerTokenAccount);
    const tokensBuyerHasRaw = buyerAtaInfo.amount;
    console.log("Buyer raw token balance =>", tokensBuyerHasRaw.toString());

    // Convert raw tokens to unscaled
    // Because on-chain "sell_exact_input" expects unscaled units
    const tokensBuyerHasUnscaled = new BN(tokensBuyerHasRaw.toString()).div(new BN(10 ** DECIMALS));

    // We'll sell half of those unscaled tokens
    const halfTokensInCurveUnits = tokensBuyerHasUnscaled.divn(2);
    console.log("Selling (unscaled) =>", halfTokensInCurveUnits.toString());

    await program.methods
      .sellExactInputInstruction(halfTokensInCurveUnits)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey, // "user" per SellToken struct
        creator: creatorKeypair.publicKey,
        xyberToken: xyberTokenPda,
        escrowTokenAccount: escrowTokenAccount,
        paymentMint: PAYMENT_MINT_PUBKEY,
        mint: mintPda,
        vaultTokenAccount: vaultTokenAccount,
        userTokenAccount: buyerTokenAccount,
        userPaymentAccount: buyerPaymentAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .rpc();

    // 3) Check final balance
    const buyerAtaInfoAfter = await getAccount(connection, buyerTokenAccount);
    assert(
      buyerAtaInfoAfter.amount < buyerAtaInfo.amount,
      "Buyer’s token balance should decrease after selling"
    );
    console.log(
      "SellExactInput done. Before=",
      buyerAtaInfo.amount.toString(),
      " after=",
      buyerAtaInfoAfter.amount.toString()
    );
  });

  // 3.6) Buyer buys EXACT output
  it("1.6) Buyer buys EXACT output: e.g. 10 project tokens (buy_exact_output_instruction)", async () => {
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      buyerKeypair.publicKey
    );
    const buyerPaymentAccount = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      buyerKeypair.publicKey
    );

    // 1) Suppose buyer wants exactly 10 unscaled tokens
    const tokensOutWanted = new BN(10);

    // 2) Check buyer's current raw balance
    const buyerAtaInfoBefore = await getAccount(connection, buyerTokenAccount);
    const buyerTokensBefore = buyerAtaInfoBefore.amount;

    // 3) Buy instruction
    await program.methods
      .buyExactOutputInstruction(tokensOutWanted)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
        creator: creatorKeypair.publicKey,
        xyberToken: xyberTokenPda,
        escrowTokenAccount: escrowTokenAccount,
        paymentMint: PAYMENT_MINT_PUBKEY,
        mint: mintPda,
        vaultTokenAccount: vaultTokenAccount,
        buyerTokenAccount: buyerTokenAccount,
        buyerPaymentAccount: buyerPaymentAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .rpc();

    // 4) Check the difference in buyer’s raw balance
    const buyerAtaInfoAfter = await getAccount(connection, buyerTokenAccount);
    const diff = buyerAtaInfoAfter.amount - buyerTokensBefore;

    // Because on-chain code multiplies by 10^decimals
    // we expect minted = 10 * 10^decimals
    const expectedRawMinted = tokensOutWanted.mul(new BN(10 ** DECIMALS));
    assert(
      diff === BigInt(expectedRawMinted.toString()),
      `Expected minted=${expectedRawMinted}, but got diff=${diff}`
    );
    console.log(
      "BuyExactOutput success. Minted raw =",
      diff.toString(),
      " (Wanted 10 unscaled tokens.)"
    );
  });

  // 3.7) Buyer sells EXACT output
  it("1.7) Buyer sells EXACT output: requests 5,000 lamports back (sell_exact_output_instruction)", async () => {
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      buyerKeypair.publicKey
    );
    const buyerPaymentAccount = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      buyerKeypair.publicKey
    );

    // 1) Suppose user wants 5,000 lamports of base back
    const lamportsWanted = new BN(5000);

    // 2) Check the user's current raw token balance
    const buyerAtaInfoBefore = await getAccount(connection, buyerTokenAccount);
    const buyerTokensBefore = buyerAtaInfoBefore.amount;

    // 3) Sell instruction
    await program.methods
      .sellExactOutputInstruction(lamportsWanted)
      .accounts({
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey,
        creator: creatorKeypair.publicKey,
        xyberToken: xyberTokenPda,
        escrowTokenAccount: escrowTokenAccount,
        paymentMint: PAYMENT_MINT_PUBKEY,
        mint: mintPda,
        vaultTokenAccount: vaultTokenAccount,
        userTokenAccount: buyerTokenAccount,
        userPaymentAccount: buyerPaymentAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .rpc();

    // 4) Check final balance
    const buyerAtaInfoAfter = await getAccount(connection, buyerTokenAccount);
    const tokensBurned = buyerTokensBefore - buyerAtaInfoAfter.amount;

    console.log(
      "SellExactOutput => user burned:",
      tokensBurned.toString(),
      " raw tokens"
    );
    assert(
      tokensBurned > BigInt(0),
      "Expected to burn some positive number of tokens"
    );
  });

  it("1.8) Final state check with base token escrow", async () => {
    console.log("----- Final State Check (Base Token Escrow) -----");

    // 1) Fetch the on-chain XyberToken struct
    const xyberState = await program.account.xyberToken.fetch(xyberTokenPda);
    console.log("=== XyberToken State ===");
    console.log(JSON.stringify(xyberState, null, 2));

    // 2) Fetch how many *base tokens* (the payment token) are in the escrow ATA
    const escrowInfo = await getAccount(connection, escrowTokenAccount);
    const escrowBalanceRaw = escrowInfo.amount; // raw, scaled by base token decimals
    console.log("Escrow base token balance (raw) =", escrowBalanceRaw.toString());

    // If your base token is also 9 decimals (like your project token),
    // you can convert to a human-readable float:
    const BASE_TOKEN_DECIMALS = 9;
    const escrowBalanceHuman =
      Number(escrowBalanceRaw) / 10 ** BASE_TOKEN_DECIMALS;
    console.log("Escrow base token balance (human-readable) =", escrowBalanceHuman);

    // 3) Compare escrow balance to the graduation threshold
    //    (compares as if escrow >= graduateDollarsAmount).
    const thresholdNeeded = Number(xyberState.graduateDollarsAmount);
    const leftToGraduate = thresholdNeeded - (escrowBalanceHuman * XBT_PRICE_DOLLARS);

    if (leftToGraduate <= 0) {
      console.log("** Already at or above graduation threshold! **");
    } else {
      console.log(
        "Left to reach graduation threshold (raw):",
        leftToGraduate
      );
      console.log(
        "Left to reach graduation (human-readable):",
        leftToGraduate / 10 ** BASE_TOKEN_DECIMALS
      );
    }

    // 4) Vault token balance (how many project tokens remain unsold)
    const vaultInfo = await getAccount(connection, vaultTokenAccount);
    const vaultBalanceRaw = vaultInfo.amount;
    console.log("Vault token balance (raw) =", vaultBalanceRaw.toString());

    const vaultBalanceHuman = Number(vaultBalanceRaw) / 10 ** DECIMALS;
    console.log(
      "Vault token balance (human-readable) =",
      vaultBalanceHuman
    );

    // 5) (Optional) Buyer’s project-token balance
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      buyerKeypair.publicKey
    );
    const buyerAtaInfo = await getAccount(connection, buyerTokenAccount);
    console.log(
      "Buyer project-token balance (raw) =",
      buyerAtaInfo.amount.toString()
    );

    const buyerBalanceHuman = Number(buyerAtaInfo.amount) / 10 ** DECIMALS;
    console.log("Buyer project-token balance (human-readable) =", buyerBalanceHuman);
  });

});