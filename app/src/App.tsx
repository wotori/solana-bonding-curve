import React, { useState } from "react";
import {
  fetchCoreState,
  updateCoreParams,
  exactBuy,
  exactSell,
} from "./actions";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { WalletModalButton } from "@solana/wallet-adapter-react-ui";

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

  // Fields for "Update Core Params"
  const [gradThreshold, setGradThreshold] = useState("1234");
  const [graduateDollarsAmount, setGraduateDollarsAmount] = useState("9999");
  const [aTotalTokens, setATotalTokens] = useState("1073000191");
  const [kVirtualPoolOffset, setKVirtualPoolOffset] = useState("32190005730");
  const [cBondingScaleFactor, setCBondingScaleFactor] = useState("30");

  // Fields for "Exact Buy" and "Exact Sell"
  const [buyAmount, setBuyAmount] = useState("5");
  const [sellAmount, setSellAmount] = useState("2");

  const { publicKey, sendTransaction } = useWallet();
  const { connection } = useConnection();

  // Fetch and log the core state
  async function onFetchCore() {
    try {
      const raw = await fetchCoreState();
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
      const raw = await fetchCoreState();
      // Log for debugging (bnReplacer is fine for logs)
      setLog(JSON.stringify(raw, bnReplacer, 2));

      setGradThreshold(String(raw.gradThreshold || 0));
      setGraduateDollarsAmount(String(raw.graduateDollarsAmount || 0));

      // If these fields are decimal, just store them directly
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

      // 1) Send the transaction
      const txSig = await exactBuy(publicKey, sendTransaction, connection, amount, 0);
      setLog(`exactBuy txSig = ${txSig}`);

      // 2) Explicitly confirm it before alerting
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

      // 1) Send the transaction
      const txSig = await exactSell(publicKey, sendTransaction, connection, amount);
      setLog(`exactSell txSig = ${txSig}`);

      // 2) Wait for on-chain confirmation
      const latestBlockhash = await connection.getLatestBlockhash();
      await connection.confirmTransaction(
        {
          blockhash: latestBlockhash.blockhash,
          lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
          signature: txSig,
        },
        "confirmed"
      );

      // 3) Alert the user
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

      {/* Fetch Core State */}
      <section style={{ border: "1px solid #ccc", padding: "10px", marginBottom: "20px" }}>
        <h2>Fetch Core State</h2>
        <button onClick={onFetchCore}>Fetch XyberCore</button>
      </section>

      {/* Update Core Params */}
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

      {/* Exact Buy / Sell */}
      <section style={{ border: "1px solid #ccc", padding: "10px", marginBottom: "20px" }}>
        <h2>Exact Buy / Sell</h2>
        <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", maxWidth: "300px" }}>
          <label>
            Buy Amount:
            <input
              type="number"
              value={buyAmount}
              onChange={(e) => setBuyAmount(e.target.value)}
              style={{ marginLeft: "1rem" }}
            />
          </label>
          <button onClick={onBuy}>Exact Buy</button>

          <label>
            Sell Amount:
            <input
              type="number"
              value={sellAmount}
              onChange={(e) => setSellAmount(e.target.value)}
              style={{ marginLeft: "1rem" }}
            />
          </label>
          <button onClick={onSell}>Exact Sell</button>
        </div>
      </section>

      {/* Logs / Output */}
      <section style={{ border: "1px solid #ccc", padding: "10px" }}>
        <h2>Logs / Output</h2>
        <pre style={{ whiteSpace: "pre-wrap" }}>{log}</pre>
      </section>
    </div>
  );
}