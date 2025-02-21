
import React, { useState } from "react";
import {
  fetchCoreState,
  updateCoreParams,
  exactBuy,
  exactSell,
} from "./actions";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";

function App() {
  const [log, setLog] = useState<string>("");


  const { publicKey, sendTransaction } = useWallet();

  const { connection } = useConnection();

  async function onFetchCore() {
    try {
      const state = await fetchCoreState();
      setLog(JSON.stringify(state, null, 2));
    } catch (error) {
      console.error(error);
      setLog("Error fetching core state");
    }
  }

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
          gradThreshold: 1234,
          graduateDollarsAmount: 9999,
          aTotalTokens: "1073000191",
          kVirtualPoolOffset: "32190005730",
          cBondingScaleFactor: "30",
        }
      );
      setLog(`Update successful. TxSig=${txSig}`);
    } catch (error) {
      console.error(error);
      setLog("Error updating core");
    }
  }

  async function onBuy() {
    if (!publicKey) {
      alert("Connect wallet first!");
      return;
    }
    try {

      const txSig = await exactBuy(publicKey, sendTransaction, connection, 5, 0);
      setLog(`exactBuy txSig=${txSig}`);
    } catch (error) {
      console.error(error);
      setLog("Error exactBuy");
    }
  }

  async function onSell() {
    if (!publicKey) {
      alert("Connect wallet first!");
      return;
    }
    try {

      const txSig = await exactSell(publicKey, sendTransaction, connection, 2);
      setLog(`exactSell txSig=${txSig}`);
    } catch (error) {
      console.error(error);
      setLog("Error exactSell");
    }
  }

  return (
    <div style={{ padding: 20 }}>
      <h1>Bonding Curve UI</h1>
      <button onClick={onFetchCore}>Fetch XyberCore</button>
      <button onClick={onUpdateCore}>Update Core</button>
      <button onClick={onBuy}>Exact Buy</button>
      <button onClick={onSell}>Exact Sell</button>
      <pre style={{ whiteSpace: "pre-wrap", marginTop: 20 }}>{log}</pre>
    </div>
  );
}

export default App;