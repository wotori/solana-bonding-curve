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
  getAccount,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { BondingCurve } from "../target/types/bonding_curve";
import { BUYER_KEYPAIR_PATH, CREATOR_KEYPAIR_PATH, DEVNET_URL, METAPLEX_PROGRAM_ID, PAYMENT_MINT_PUBKEY, TOKEN_FACTORY_PROGRAM_ID } from "./constants";


// Test constants
const TOTAL_TOKENS = new BN("1073000191");
const DECIMALS = 9;
const LAMPORTS_PER_TOKEN = 10 ** DECIMALS;

// Adoptation (Xyber)
const scaleFactor = new BN("300");
const BONDING_K_VIRTUAL = new BN("32190005730")
  .mul(scaleFactor)
  .mul(new BN(LAMPORTS_PER_TOKEN));

const VIRTUAL_POOL_OFFSET = new BN(30)
  .mul(scaleFactor)
  .mul(new BN(LAMPORTS_PER_TOKEN));

const GRADUATE_THRESHOLD = new BN("2000000");

// Original (SOL)
// const BONDING_K_VIRTUAL = new BN("32190005730").mul(new BN(LAMPORTS_PER_TOKEN));
// const VIRTUAL_POOL_OFFSET = new BN(30 * LAMPORTS_PER_TOKEN);
// const GRADUATE_THRESHOLD = new BN("428");

// Metadata parameters for the project toke
const now = new Date();
const year = now.getFullYear().toString().slice(2);
const month = String(now.getMonth() + 1).padStart(2, "0");
const day = String(now.getDate()).padStart(2, "0");
const hours = String(now.getHours()).padStart(2, "0");
const minutes = String(now.getMinutes()).padStart(2, "0");

const tokenName = `${year}_${month}_${day}_${hours}_${minutes}`;
const tokenSymbol = "HWD";
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
  let xyberCorePda: PublicKey;
  let mintPda: PublicKey;
  let vaultTokenAccount: PublicKey;
  let creatorTokenAccount: PublicKey;
  let escrowTokenAccount: PublicKey;
  let metadataPda: PublicKey;

  // Derive PDAs in before() hook
  before("Derive all PDAs", async () => {
    // XyberCore PDA
    [xyberCorePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("xyber_core")],
      program.programId
    );

    // XyberToken PDA
    tokenSeedKeypair = Keypair.generate();
    [xyberTokenPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("xyber_token"),
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
  it("1.2) update_core_instruction with random gradThreshold & small graduateDollars offset", async () => {
    console.log("----- Step 2: update_core_instruction -----");

    // await program.methods.closeXyberCoreInstruction() // uncomment to recreate the core if needed
    //   .accounts({
    //     xyberCore: xyberCorePda,
    //     admin: creatorKeypair.publicKey,
    //   })
    //   .signers([creatorKeypair])
    //   .rpc();

    // Add a small random offset (1..3) to GRADUATE_DOLLARS_AMOUNT

    // Build the update params
    const updateTokenParams = {
      admin: creatorKeypair.publicKey,
      gradThreshold: GRADUATE_THRESHOLD,
      bondingCurve: {
        aTotalTokens: TOTAL_TOKENS,
        kVirtualPoolOffset: BONDING_K_VIRTUAL,
        cBondingScaleFactor: VIRTUAL_POOL_OFFSET,
      },
      acceptedBaseMint: PAYMENT_MINT_PUBKEY,
    };

    const escrowTokenAccountPda = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      xyberCorePda,
      true // true = allowOwnerOffCurve if xyberCorePda is a PDA
    );

    // Construct the instruction
    const ixUpdate = await program.methods
      .updateXyberCoreInstruction(updateTokenParams)
      .accounts({
        admin: creatorKeypair.publicKey,
        xyberCore: xyberCorePda,

        newAcceptedBaseMint: PAYMENT_MINT_PUBKEY,
        escrowTokenAccount: escrowTokenAccountPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,

        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([creatorKeypair])
      .instruction();

    // Send the transaction
    const txUpdate = new Transaction().add(ixUpdate);
    console.log("Sending update_core_instruction transaction...");
    const sigUpdate = await provider.sendAndConfirm(txUpdate, [creatorKeypair]);
    console.log("update_core_instruction SUCCESS, signature =", sigUpdate);

    // Fetch the updated state
    const xyberState = await program.account.xyberCore.fetch(xyberCorePda);
    console.log("After update_core, XYBER state:", xyberState);
  });

  // 3.2) init_and_mint_full_supply_instruction
  it("1.3) mint_full_supply_instruction", async () => {
    console.log("----- Step 3: mint_full_supply_instruction -----");

    const ixMintFullSupply = await program.methods
      .mintFullSupplyInstruction({
        name: tokenName,
        symbol: tokenSymbol,
        uri: tokenUri,
        totalChains: 1,
      })
      .accounts({
        xyberCore: xyberCorePda,
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
        escrowTokenAccount: escrowTokenAccount,
        paymentMint: PAYMENT_MINT_PUBKEY,
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

  // 3.3) Buyer buys token with exact base input
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
    const baseIn = new BN(0.1 * LAMPORTS_PER_TOKEN);
    const expectedOut = new BN(10_000);

    // 4) Call the instruction
    await program.methods
      .buyExactInputInstruction(baseIn, expectedOut)
      .accounts({
        xyberCore: xyberCorePda,
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
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

  // 3.4) Buyer sells token with exact input
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
    const tokensBuyerHasUnscaled = new BN(tokensBuyerHasRaw.toString()).div(
      new BN(10 ** DECIMALS)
    );

    // We'll sell half of those unscaled tokens
    const halfTokensInCurveUnits = tokensBuyerHasUnscaled.divn(2);
    console.log("Selling (unscaled) =>", halfTokensInCurveUnits.toString());

    let slippage = new BN("0");
    await program.methods
      .sellExactInputInstruction(halfTokensInCurveUnits, slippage)
      .accounts({
        xyberCore: xyberCorePda,
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey, // "user" per SellToken struct
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

  // 3.5) Buyer buys EXACT output
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
    const xyberTokensOutWanted = new BN(10);
    const baseTokensPayExpected = new BN(15_000_000);

    // 2) Check buyer's current raw balance
    const buyerAtaInfoBefore = await getAccount(connection, buyerTokenAccount);
    const buyerTokensBefore = buyerAtaInfoBefore.amount;

    // 3) Buy instruction
    await program.methods
      .buyExactOutputInstruction(xyberTokensOutWanted, baseTokensPayExpected)
      .accounts({
        xyberCore: xyberCorePda,
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
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
    const expectedRawMinted = xyberTokensOutWanted.mul(new BN(10 ** DECIMALS));
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

  // 3.6) Buyer sells EXACT output
  it("1.7) Buyer sells EXACT output: requests lamports back (sell_exact_output_instruction)", async () => {
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      buyerKeypair.publicKey
    );
    const buyerPaymentAccount = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      buyerKeypair.publicKey
    );

    // 1) Suppose user wants lamports of base back
    const lamportsWanted = new BN(0.01 * LAMPORTS_PER_TOKEN);

    // 2) Check the user's current raw token balance
    const buyerAtaInfoBefore = await getAccount(connection, buyerTokenAccount);
    const buyerTokensBefore = buyerAtaInfoBefore.amount;

    // 3) Sell instruction
    await program.methods
      .sellExactOutputInstruction(lamportsWanted, lamportsWanted)
      .accounts({
        xyberCore: xyberCorePda,
        tokenSeed: tokenSeedKeypair.publicKey,
        user: buyerKeypair.publicKey,
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

    // If base token is also 9 decimals
    // you can convert to a human-readable float:
    const BASE_TOKEN_DECIMALS = 9;
    const escrowBalanceHuman =
      Number(escrowBalanceRaw) / 10 ** BASE_TOKEN_DECIMALS;
    console.log("Escrow base token balance (human-readable) =", escrowBalanceHuman);

    // 3) Compare escrow balance to the graduation threshold
    const thresholdNeeded = Number(xyberState.gradThreshold);
    const leftToGraduate =
      thresholdNeeded - escrowBalanceHuman;

    if (leftToGraduate <= 0) {
      console.log("** Already at or above graduation threshold! **");
    } else {
      console.log("Left to reach graduation threshold (raw):", leftToGraduate);
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
    console.log("Vault token balance (human-readable) =", vaultBalanceHuman);

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

  it("1.9) Dump Info for Frontend", async () => {
    console.log("----- Dumping Key Info for Frontend -----");

    // 0) The token seed public key (the most critical piece for front-end calls)
    console.log(
      "tokenSeedKeypair.publicKey =",
      tokenSeedKeypair.publicKey.toBase58()
    );

    // 1) PDAs
    console.log("xyberCorePda =", xyberCorePda.toBase58());
    console.log("xyberTokenPda =", xyberTokenPda.toBase58());
    console.log("mintPda =", mintPda.toBase58());
    console.log("vaultTokenAccount =", vaultTokenAccount.toBase58());
    console.log("escrowTokenAccount =", escrowTokenAccount.toBase58());

    // 2) PublicKeys for creator & buyer
    console.log(
      "creatorKeypair.publicKey =",
      creatorKeypair.publicKey.toBase58()
    );
    console.log("buyerKeypair.publicKey =", buyerKeypair.publicKey.toBase58());

    // 3) XyberCore & XyberToken state
    const xyberCoreState = await program.account.xyberCore.fetch(xyberCorePda);
    const xyberTokenState = await program.account.xyberToken.fetch(xyberTokenPda);

    console.log("== XyberCore State ==");
    console.log(JSON.stringify(xyberCoreState, null, 2));

    console.log("== XyberToken State ==");
    console.log(JSON.stringify(xyberTokenState, null, 2));

    // 4) Balances of escrow & vault
    const escrowInfo = await getAccount(connection, escrowTokenAccount);
    console.log("Escrow raw balance =", escrowInfo.amount.toString());

    const vaultInfo = await getAccount(connection, vaultTokenAccount);
    console.log("Vault raw balance =", vaultInfo.amount.toString());

    // 5) Creator & Buyer ATA (for the minted project token)
    const creatorAta = await getAssociatedTokenAddress(
      mintPda,
      creatorKeypair.publicKey
    );
    const buyerAta = await getAssociatedTokenAddress(
      mintPda,
      buyerKeypair.publicKey
    );

    console.log("creatorTokenAccount =", creatorAta.toBase58());
    console.log("buyerTokenAccount =", buyerAta.toBase58());

    // const creatorAtaInfo = await getAccount(connection, creatorAta);
    const buyerAtaInfo = await getAccount(connection, buyerAta);

    // console.log("creator token balance (raw) =", creatorAtaInfo.amount.toString());
    console.log("buyer token balance (raw) =", buyerAtaInfo.amount.toString());

    console.log("----- End of Dump -----");
  });

  // 1.95) Buyer buys EXACT input => 500 base_in => check isGraduated, log escrow balance
  it("1.95) Buyer buys EXACT input => 500 base_in => check isGraduated, log escrow", async () => {
    console.log("----- Step 1.95: buy_exact_input_instruction (base_in=500) -----");

    // 1) Find the ATA for the buyer
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mintPda,
      buyerKeypair.publicKey
    );
    const buyerPaymentAccount = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      buyerKeypair.publicKey
    );

    // 2) Construct baseIn = 500 * 10^9, assuming we have 9 decimals in the base token (USDC-like)
    const baseIn = new BN(500).mul(new BN(LAMPORTS_PER_TOKEN));
    const minAmountOut = new BN(0);

    console.log(
      "Buyer paying baseIn =",
      baseIn.toString(),
      " (which is 500 in human-readable form if decimals=9)"
    );

    // 3) Call buyExactInputInstruction
    await program.methods
      .buyExactInputInstruction(baseIn, minAmountOut)
      .accounts({
        xyberCore: xyberCorePda,
        tokenSeed: tokenSeedKeypair.publicKey,
        buyer: buyerKeypair.publicKey,
        xyberToken: xyberTokenPda,
        escrowTokenAccount: escrowTokenAccount,
        paymentMint: PAYMENT_MINT_PUBKEY,
        mint: mintPda,
        vaultTokenAccount: vaultTokenAccount,
        buyerTokenAccount: buyerTokenAccount,
        buyerPaymentAccount: buyerPaymentAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([buyerKeypair])
      .rpc();

    console.log("BuyExactInput(500) finished => Checking escrow & graduation...");

    // 4) Log the escrow balance
    const escrowInfo = await getAccount(connection, escrowTokenAccount);
    const escrowBalanceRaw = escrowInfo.amount;
    console.log("Escrow balance (raw) =", escrowBalanceRaw.toString());

    const escrowBalanceHuman =
      Number(escrowBalanceRaw) / Number(LAMPORTS_PER_TOKEN);
    console.log("Escrow balance (human-readable) =", escrowBalanceHuman);

    // 5) Check the isGraduated field
    const xyberTokenState = await program.account.xyberToken.fetch(xyberTokenPda);
    console.log("xyberTokenState:", xyberTokenState);

    if (xyberTokenState.isGraduated) {
      console.log("✅ Token IS graduated!");
    } else {
      console.log("❌ Token is NOT graduated yet!");
    }

    // 6) Add a strict assert, expecting graduation == true
    assert.equal(
      xyberTokenState.isGraduated,
      true,
      "Expecting that token is graduated"
    );
  });

  it("2.1) Withdraw Liquidity with correct admin", async () => {
    // Derive the admin's ATA for the base token and project token
    const adminBaseAta = await getAssociatedTokenAddress(
      PAYMENT_MINT_PUBKEY,
      creatorKeypair.publicKey
    );
    const adminVaultAta = await getAssociatedTokenAddress(
      mintPda,
      creatorKeypair.publicKey
    );

    try {
      // Withdraw all funds
      await program.methods
        .withdrawLiquidity()
        .accounts({
          admin: creatorKeypair.publicKey,
          xyberCore: xyberCorePda,
          xyberToken: xyberTokenPda,
          tokenSeed: tokenSeedKeypair.publicKey,
          creator: creatorKeypair.publicKey,
          escrowTokenAccount,
          baseTokenMint: PAYMENT_MINT_PUBKEY,
          mint: mintPda,
          vaultTokenAccount: vaultTokenAccount,
          adminTokenAccount: adminBaseAta,
          adminVaultAccount: adminVaultAta,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([creatorKeypair])
        .rpc();

      console.log("Withdrawal succeeded (no balance checks).");
    } catch (err) {
      console.error("Withdrawal attempt failed:", err);
      assert.fail(`Unexpected error: ${err.toString()}`);
    }
  });
});