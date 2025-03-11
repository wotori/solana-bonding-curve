
import {
    PublicKey,
} from "@solana/web3.js";
import path from "path";

// Metaplex Metadata program ID
export const METAPLEX_PROGRAM_ID = new PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Token factory program ID
export const TOKEN_FACTORY_PROGRAM_ID = new PublicKey(
    "TF5AoQEG87r1gpWsNzADMxYean6tfdGVUouQQ5LbYPP"
);

// Adjust these paths/keys to suit environment:
export const DEVNET_URL = "https://api.devnet.solana.com";
export const CREATOR_KEYPAIR_PATH = path.join(
    process.env.HOME!,
    ".config",
    "solana",
    "devnet-owner.json"
);
export const BUYER_KEYPAIR_PATH = path.join(
    process.env.HOME!,
    ".config",
    "solana",
    "devnet-buyer.json"
);

// Base (payment) token mint
export const PAYMENT_MINT_PUBKEY = new PublicKey(
    // "2fV8xnkYe5pjuQh6AsexFHJDUUdycUVU3ioRsJU4wsh2"
    "3EsoJyspCZ4tjcPX8v4UdvayBub9h9cxcJQvwAXqs5KZ"
);
