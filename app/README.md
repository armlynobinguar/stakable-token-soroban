# Staking dApp (Svelte)

Front end for the **stakable token** contract. Connect Freighter (testnet), view balance/staked/pending reward, and stake or unstake.

## Setup

1. Deploy the contract and set the contract ID (see [../DEPLOY.md](../DEPLOY.md)).
2. Copy env example and set your contract ID:
   ```bash
   cp .env.example .env
   # Edit .env: VITE_CONTRACT_ID=C...
   ```
3. Install and run:
   ```bash
   npm install
   npm run dev
   ```
4. Open the URL (e.g. http://localhost:5173), install [Freighter](https://www.freighter.app/) if needed, switch to **testnet**, then connect wallet.

## Scripts

- `npm run dev` — start dev server
- `npm run build` — production build
- `npm run preview` — preview production build
