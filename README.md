## Intro to Soroban Smart Contracts (Rust)

This repository is a **beginner‑friendly playground** for learning Soroban (Stellar) smart contracts in Rust.

It is organized as a small Rust workspace with **separate folders for each sample contract**:

- `contracts/hello_world`: minimal "Hello, \<name\>" contract and test
- `contracts/increment`: simple counter that **stores data on-chain**
- `contracts/stakable_token`: DeFi staking (mint, transfer, stake, unstake, 10% APY)

For a quick tour of the samples and what each one teaches, see `contracts/README.md`.

---

## Staking DeFi dApp (Svelte + Soroban)

This repo includes a **stakable token** contract and a **Svelte** front end for staking on Stellar testnet.

- **Contract:** `contracts/stakable_token` — mint, transfer, stake, unstake, 10% APY rewards.
- **UI:** `app/` — Svelte app that connects Freighter, shows balance/staked/rewards, and lets you stake/unstake.

### Prerequisites

- **Rust** (1.84+): `rustup default stable`, then `rustup target add wasm32v1-none`
- **Stellar CLI:** `brew install stellar-cli` (macOS) or [install script](https://github.com/stellar/stellar-cli) — verify with `stellar --version`
- **Freighter** browser extension: [freighter.app](https://www.freighter.app/)

### 1. Test the contract (no deployment)

From the **repo root**:

```bash
cargo test -p stakable-token
```

You should see: `test test::test_stake_and_unstake ... ok`

### 2. Build the contract

From the **repo root**:

```bash
stellar contract build
```

Produces `target/wasm32v1-none/release/stakable_token.wasm`.

### 3. Create a testnet identity and deploy

```bash
# Create and fund admin on testnet
stellar keys generate admin --network testnet --fund

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/stakable_token.wasm \
  --source-account admin \
  --network testnet
```

Copy the **contract ID** from the output (starts with `C...`). Get your admin public key:

```bash
stellar keys public-key admin
```

### 4. Initialize the contract

Replace `CONTRACT_ID` with the ID from step 3 and `ADMIN_PUBLIC_KEY` with the output of `stellar keys public-key admin`:

```bash
stellar contract invoke \
  --id CONTRACT_ID \
  --source-account admin \
  --network testnet \
  -- \
  init \
  --admin ADMIN_PUBLIC_KEY
```

### 5. Fund the reward pool and mint to users

The contract pays rewards from its own balance. Mint to the **contract ID** (reward pool) and to your Freighter address so you can stake in the UI.

Mint to the contract (reward pool):

```bash
stellar contract invoke \
  --id CONTRACT_ID \
  --source-account admin \
  --network testnet \
  -- \
  mint \
  --admin ADMIN_PUBLIC_KEY \
  --to CONTRACT_ID \
  --amount 1000000
```

Mint to your Freighter address (use `stellar keys public-key freighter` if you added it, or your G... address):

```bash
stellar contract invoke \
  --id CONTRACT_ID \
  --source-account admin \
  --network testnet \
  -- \
  mint \
  --admin ADMIN_PUBLIC_KEY \
  --to YOUR_FREIGHTER_PUBLIC_KEY \
  --amount 10000
```

### 6. Run the Svelte app

Run these commands **from the `app` directory** (not the repo root), or you’ll get “Missing script: dev”:

```bash
cd app
cp .env.example .env
```

Edit `.env` and set your contract ID:

```
VITE_CONTRACT_ID=CONTRACT_ID
```

Then:

```bash
npm install
npm run dev
```

Open the URL (e.g. http://localhost:5173), connect Freighter (switch to **testnet**), and use Stake / Unstake.

### Keys and identities

- Use **admin** (CLI) for deploy, init, and mint. You don’t need to add your Freighter key to the CLI for that.
- To add your Freighter **public** key so you can use it as `--to` when minting:  
  `stellar keys add freighter --public-key "G..."`  
  (That identity cannot sign; it’s for reference only.)
- Use **Freighter in the browser** when running the app (connect wallet, stake/unstake).

---

## General setup (Rust + Stellar CLI)

### Install Rust (macOS)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup target add wasm32v1-none
rustc --version
```

Soroban requires **Rust 1.84.0 or newer**.

### Install Stellar CLI

Pick ONE method (macOS):

```bash
brew install stellar-cli
# OR: curl -fsSL https://github.com/stellar/stellar-cli/raw/main/install.sh | sh
# OR: cargo install --locked stellar-cli
```

Verify: `stellar --version`

### Build and test all contracts

```bash
stellar contract build
cargo test
```

### Deploy and invoke sample contracts (hello_world, increment)

Generate a funded testnet key:

```bash
stellar keys generate alice --network testnet --fund
```

Deploy and invoke as needed (see `contracts/README.md` for examples). Example for hello_world:

```bash
stellar contract deploy --wasm target/wasm32v1-none/release/hello_world.wasm --source-account alice --network testnet --alias hello_world
stellar contract invoke --id hello_world --source-account alice --network testnet -- hello --to RPC
```

You can run `stellar contract invoke ... -- --help` for more options.
