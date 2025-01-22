# Bonding Curve – Solana
A bonding curve program with enhanced features for the Xyber project. Includes token locking logic, liquidity pool creation, and more. 

<img src="logo.png" alt="solana-bonding-curve" width="300" height="300">

## Formula

We used the following formula to define the bonding curve:

**y(x) = A - K / (C + x)**

This formula represents the cumulative minted tokens (y) as a function of the base asset contributed (x). The parameters are:

- **A**: Maximum number of tokens (asymptotic limit)
- **K**: Determines the "speed" of approaching the maximum
- **C**: Virtual pool or offset value

For visualization, a script is available in the `qa` folder that generates the curve. 

### Example Visualization

Below is an example of the bonding curve:

<img src="bonding_curve.png" alt="Solana Bonding Curve" width="400" height="300">

## Testing Instructions

Below are common commands and steps for testing Solana programs in both local and devnet environments.

Note: Tests may not fully run on localnet at this time because the Metaplex program is required on localnet. For that reason, tests can currently only be run in full on devnet.

It may also be useful to note that after generating your devnet wallet, you can add its key array directly to Phantom so you can see tokens appear in your wallet.

---

### 1. Generate dev-net keypair
```bash
solana-keygen new --outfile ~/.config/solana/devnet-owner.json

solana-keygen new --outfile ~/.config/solana/devnet-buyer.json
```

### 2. Airdrop More Than the Daily Limit on Devnet
If you need more than the default devnet faucet limit, register at [Solana Faucet](https://faucet.solana.com/) with your GitHub account to request larger amounts of SOL.

### 3. Airdrop on Localnet
```bash
solana airdrop 10 <YOUR_WALLET_ADDRESS> --url http://127.0.0.1:8899
```

### 4. Switch Between Devnet and Localnet
```bash
# Set CLI to Devnet
solana config set --url devnet

# Or set CLI to Localnet
solana config set --url http://127.0.0.1:8899
```

### 5. Get Your Solana Address and Check Balance
```bash
solana address
solana balance
```

### 6. Build and Deploy Solana Program
```bash
anchor build
anchor deploy

# or
make bd
```

### 7. Run a Local Validator and Tests
```bash
solana-test-validator
anchor test --skip-local-validator
```

### 8. Run Tests on Devnet
```bash
anchor test --provider.cluster devnet
#or
anchor test --skip-build --skip-deploy --provider.cluster devnet
#or
make test-dev
```

### 9. Standard Testing (Localnet Auto-Spawned)
```bash
anchor test
```