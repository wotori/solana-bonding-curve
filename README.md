# Bonding Curve – Solana
A bonding curve program with enhanced features for the Xyber project. Includes token locking logic, liquidity pool creation, and more. 

## Testing Instructions

Below are common commands and steps for testing your Solana programs in both local and devnet environments.

---

### 1. Airdrop More Than the Daily Limit on Devnet
If you need more than the default devnet faucet limit, register at [Solana Faucet](https://faucet.solana.com/) with your GitHub account to request larger amounts of SOL.

### 2. Switch Between Devnet and Localnet
```bash
# Set your CLI to Devnet
solana config set --url devnet

# Or set your CLI to Localnet
solana config set --url http://127.0.0.1:8899
```

### 3. Get Your Solana Address and Check Balance
```
solana address
solana balance
```

### 4. Run a Local Validator and Tests
```
solana-test-validator
anchor test --skip-local-validator
```

### 5. Airdrop on Localnet
```
solana airdrop 10 <YOUR_WALLET_ADDRESS> --url http://127.0.0.1:8899
```

### 6. Run Tests on Devnet
```
anchor test --provider.cluster devnet
```

### 7. Standard Testing (Localnet Auto-Spawned)
```
anchor test
```