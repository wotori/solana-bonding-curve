import React, { useState } from "react";
import {
  fetchCoreState,
  updateCoreParams,
  exactBuy,
  exactSell,
  // 1) Import new combined function
  mintFullSupplyAndInitialBuyInOneTx
} from "./actions";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { WalletModalButton } from "@solana/wallet-adapter-react-ui";
import { PublicKey } from "@solana/web3.js";
import { BN } from "@project-serum/anchor";

// For logging only — blindly treats alphanumeric as hex
function bnReplacer(key: any, value: any) {
  if (typeof value === "string" && /^[0-9a-fA-F]+$/.test(value)) {
    try {
      return BigInt("0x" + value).toString();
    } catch { }
  }
  return value;
}

export default function App() {
  const [log, setLog] = useState("");


  const wallet = useWallet();
  const { publicKey, sendTransaction } = wallet;
  const { connection } = useConnection();

  /**************************
   * 1) Mint + Initial Buy *
   **************************/
  const [tokenName, setTokenName] = useState("MyToken");
  const [tokenSymbol, setTokenSymbol] = useState("MYTK");
  const [tokenUri, setTokenUri] = useState("https://example.com/meta.json");
  const [depositLamports, setDepositLamports] = useState("1");

  async function onMintFullSupplyAndInitialBuy() {
    if (!publicKey) {
      alert("Connect wallet first!");
      return;
    }
    try {
      const lamports = parseInt(depositLamports, 10);
      if (isNaN(lamports) || lamports <= 0) {
        alert("Invalid deposit lamports amount!");
        return;
      }

      // same pattern as exactBuy
      const txSig = await mintFullSupplyAndInitialBuyInOneTx(
        sendTransaction,
        connection,
        wallet,
        {
          tokenName,
          tokenSymbol,
          tokenUri,
          depositLamports: lamports,
        }
      );

      setLog(`Mint+InitialBuy TX = ${txSig}`);
      alert(`Mint + Buy Succeeded!\nTx Signature: ${txSig}`);
    } catch (error) {
      console.error(error);
      setLog("Error in mintFullSupplyAndInitialBuyInOneTx");
    }
  }

  /******************************************
   * Existing Core Param + Buy/Sell Fields  *
   ******************************************/
  const [gradThreshold, setGradThreshold] = useState("1234");
  const [graduateDollarsAmount, setGraduateDollarsAmount] = useState("9999");
  const [aTotalTokens, setATotalTokens] = useState("1073000191");
  const [kVirtualPoolOffset, setKVirtualPoolOffset] = useState("32190005730");
  const [cBondingScaleFactor, setCBondingScaleFactor] = useState("30");
  const [buyAmount, setBuyAmount] = useState("5");
  const [sellAmount, setSellAmount] = useState("2");

  // Fetch and log the core state
  async function onFetchCore() {
    try {
      const raw = await fetchCoreState(wallet);
      const processed = JSON.stringify(raw, bnReplacer, 2);
      setLog(processed);
    } catch (error) {
      console.error(error);
      setLog("Error fetching core state");
    }
  }

  // Prefill form fields from on-chain data
  async function onPrefillParams() {
    try {
      const raw = await fetchCoreState(wallet);
      // Log for debugging
      setLog(JSON.stringify(raw, bnReplacer, 2));

      setGradThreshold(String(raw.gradThreshold || 0));
      setGraduateDollarsAmount(String(raw.graduateDollarsAmount || 0));

      const bc = raw.bondingCurve || {};
      setATotalTokens(String(bc.aTotalTokens || "0"));
      setKVirtualPoolOffset(String(bc.kVirtualPoolOffset || "0"));
      setCBondingScaleFactor(String(bc.cBondingScaleFactor || "0"));
    } catch (error) {
      console.error(error);
      setLog("Error pre-filling core params");
    }
  }

  // Update core parameters
  async function onUpdateCore() {
    if (!publicKey) {
      alert("Connect wallet first!");
      return;
    }
    try {
      const txSig = await updateCoreParams(
        publicKey,
        sendTransaction,
        connection,
        wallet,
        {
          gradThreshold: Number(gradThreshold),
          graduateDollarsAmount: Number(graduateDollarsAmount),
          aTotalTokens,
          kVirtualPoolOffset,
          cBondingScaleFactor,
        }
      );
      setLog(`Update successful. TxSig = ${txSig}`);
      alert(`Core update succeeded!\nTx Signature: ${txSig}`);
    } catch (error) {
      console.error(error);
      setLog("Error updating core");
    }
  }

  // Exact Buy
  async function onBuy() {
    if (!publicKey) {
      alert("Connect wallet first!");
      return;
    }
    try {
      const amount = Number(buyAmount);
      if (isNaN(amount) || amount <= 0) {
        alert("Invalid buy amount!");
        return;
      }

      const txSig = await exactBuy(publicKey, sendTransaction, connection, amount, 0, wallet);
      setLog(`exactBuy txSig = ${txSig}`);

      const latestBlockhash = await connection.getLatestBlockhash();
      await connection.confirmTransaction(
        {
          blockhash: latestBlockhash.blockhash,
          lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
          signature: txSig,
        },
        "confirmed"
      );

      alert(`Buy transaction succeeded!\nTx Signature: ${txSig}`);
    } catch (error) {
      console.error(error);
      setLog("Error exactBuy");
    }
  }

  // Exact Sell
  async function onSell() {
    if (!publicKey) {
      alert("Connect wallet first!");
      return;
    }
    try {
      const amount = Number(sellAmount);
      if (isNaN(amount) || amount <= 0) {
        alert("Invalid sell amount!");
        return;
      }

      const txSig = await exactSell(publicKey, sendTransaction, connection, amount, wallet);
      setLog(`exactSell txSig = ${txSig}`);

      const latestBlockhash = await connection.getLatestBlockhash();
      await connection.confirmTransaction(
        {
          blockhash: latestBlockhash.blockhash,
          lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
          signature: txSig,
        },
        "confirmed"
      );

      alert(`Sell transaction succeeded!\nTx Signature: ${txSig}`);
    } catch (error) {
      console.error(error);
      setLog("Error exactSell");
    }
  }

  return (
    <div style={{ padding: "20px", maxWidth: "700px", margin: "0 auto" }}>
      <h1>Bonding Curve UI</h1>

      {/* Connect Wallet */}
      <div style={{ marginBottom: "1rem" }}>
        <WalletModalButton />
      </div>

      {/* 1) Mint Full Supply + Initial Buy */}
      <section style={{ border: "1px solid #ccc", padding: "10px", marginBottom: "20px" }}>
        <h2>Mint Token & Initial Buy</h2>
        <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", maxWidth: "300px" }}>
          <label>
            Token Name:
            <input
              type="text"
              value={tokenName}
              onChange={(e) => setTokenName(e.target.value)}
            />
          </label>
          <label>
            Token Symbol:
            <input
              type="text"
              value={tokenSymbol}
              onChange={(e) => setTokenSymbol(e.target.value)}
            />
          </label>
          <label>
            Token URI:
            <input
              type="text"
              value={tokenUri}
              onChange={(e) => setTokenUri(e.target.value)}
            />
          </label>
          <label>
            Deposit Lamports:
            <input
              type="number"
              value={depositLamports}
              onChange={(e) => setDepositLamports(e.target.value)}
            />
          </label>
          <button onClick={onMintFullSupplyAndInitialBuy}>
            Mint + Initial Buy
          </button>
        </div>
      </section>

      {/* 2) Fetch Core State */}
      <section style={{ border: "1px solid #ccc", padding: "10px", marginBottom: "20px" }}>
        <h2>Fetch Core State</h2>
        <button onClick={onFetchCore}>Fetch XyberCore</button>
      </section>

      {/* 3) Update Core Params */}
      <section style={{ border: "1px solid #ccc", padding: "10px", marginBottom: "20px" }}>
        <h2>Update Core Params</h2>
        <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
          <label>
            gradThreshold:
            <input
              type="number"
              value={gradThreshold}
              onChange={(e) => setGradThreshold(e.target.value)}
              style={{ marginLeft: "1rem" }}
            />
          </label>
          <label>
            graduateDollarsAmount:
            <input
              type="number"
              value={graduateDollarsAmount}
              onChange={(e) => setGraduateDollarsAmount(e.target.value)}
              style={{ marginLeft: "1rem" }}
            />
          </label>
          <label>
            aTotalTokens (BN string):
            <input
              type="text"
              value={aTotalTokens}
              onChange={(e) => setATotalTokens(e.target.value)}
              style={{ marginLeft: "1rem" }}
            />
          </label>
          <label>
            kVirtualPoolOffset (BN string):
            <input
              type="text"
              value={kVirtualPoolOffset}
              onChange={(e) => setKVirtualPoolOffset(e.target.value)}
              style={{ marginLeft: "1rem" }}
            />
          </label>
          <label>
            cBondingScaleFactor (BN string):
            <input
              type="text"
              value={cBondingScaleFactor}
              onChange={(e) => setCBondingScaleFactor(e.target.value)}
              style={{ marginLeft: "1rem" }}
            />
          </label>
        </div>
        <div style={{ display: "flex", gap: "1rem", marginTop: "1rem" }}>
          <button onClick={onUpdateCore}>Update Core</button>
          <button onClick={onPrefillParams}>Prefill Params</button>
        </div>
      </section>

      {/* 4) Exact Buy / Sell */}
      <section style={{ border: "1px solid #ccc", padding: "10px", marginBottom: "20px" }}>
        <h2>Exact Buy / Sell</h2>
        <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", maxWidth: "300px" }}>
          <label>
            Buy Amount:
            <input
              type="number"
              value={buyAmount}
              onChange={(e) => setBuyAmount(e.target.value)}
            />
          </label>
          <button onClick={onBuy}>Exact Buy</button>

          <label>
            Sell Amount:
            <input
              type="number"
              value={sellAmount}
              onChange={(e) => setSellAmount(e.target.value)}
            />
          </label>
          <button onClick={onSell}>Exact Sell</button>
        </div>
      </section>

      {/* 5) Logs / Output */}
      <section style={{ border: "1px solid #ccc", padding: "10px" }}>
        <h2>Logs / Output</h2>
        <pre style={{ whiteSpace: "pre-wrap" }}>{log}</pre>
      </section>
    </div>
  );
}