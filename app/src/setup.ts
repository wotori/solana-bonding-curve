// setup.ts
import { Program, Idl } from "@coral-xyz/anchor";
import { clusterApiUrl, Connection } from "@solana/web3.js";
import idl from "../../target/idl/bonding_curve.json";
console.log("IDL metadata.address =", (idl as any).metadata?.address);
import { BondingCurve } from "../../target/types/bonding_curve";

const connection = new Connection(clusterApiUrl("devnet"), "confirmed");

// export const program = new Program(
//     idl as Idl,
//     undefined as unknown as string,  // или as any
//     { connection } as any
// ) as unknown as Program<BondingCurve>;

export const program = new Program(
    idl as Idl,
    "7TtWm2z8uixrGbxhkT1SYZfWfbiAJEg7zRaozUh46v2C",
    { connection } as any
);