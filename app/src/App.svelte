<script>
  import { onMount } from 'svelte';

  // Contract ID – set after deploying to testnet (see README)
  const CONTRACT_ID = import.meta.env.VITE_CONTRACT_ID || '';

  let hasFreighter = false;
  let connected = false;
  let address = '';
  let balance = 0n;
  let stakedBalance = 0n;
  let pendingReward = 0n;
  let stakeAmount = '';
  let unstakeAmount = '';
  let loading = false;
  let error = '';
  let txHash = '';

  async function connectWallet() {
    error = '';
    try {
      const freighter = window.freighter;
      if (!freighter) {
        throw new Error('Freighter extension not detected. Install it from freighter.app and refresh.');
      }

      // Prompt the user to connect / grant access.
      if (freighter?.isConnected && !(await freighter.isConnected())) {
        await freighter.connect();
      }

      const pub = await freighter.getPublicKey();
      if (!pub) throw new Error('Could not get public key');
      address = pub;
      connected = true;
      await refreshBalances();
    } catch (e) {
      error = e?.message || String(e);
    }
  }

  async function refreshBalances() {
    if (!CONTRACT_ID || !address) return;
    loading = true;
    error = '';
    try {
      const { getBalance, getStakedBalance, getPendingReward } = await import('./lib/soroban.js');
      [balance, stakedBalance, pendingReward] = await Promise.all([
        getBalance(CONTRACT_ID, address),
        getStakedBalance(CONTRACT_ID, address),
        getPendingReward(CONTRACT_ID, address),
      ]);
    } catch (e) {
      error = e?.message || String(e);
    } finally {
      loading = false;
    }
  }

  async function stake() {
    const amount = stakeAmount.trim();
    if (!amount || !CONTRACT_ID || !address) return;
    loading = true;
    error = '';
    txHash = '';
    try {
      const { invokeContract } = await import('./lib/soroban.js');
      const hash = await invokeContract(
        CONTRACT_ID,
        'stake',
        address,
        amount,
        window.freighter
      );
      txHash = hash;
      stakeAmount = '';
      await refreshBalances();
    } catch (e) {
      error = e?.message || String(e);
    } finally {
      loading = false;
    }
  }

  async function unstake() {
    const amount = unstakeAmount.trim();
    if (!amount || !CONTRACT_ID || !address) return;
    loading = true;
    error = '';
    txHash = '';
    try {
      const { invokeContract } = await import('./lib/soroban.js');
      const hash = await invokeContract(
        CONTRACT_ID,
        'unstake',
        address,
        amount,
        window.freighter
      );
      txHash = hash;
      unstakeAmount = '';
      await refreshBalances();
    } catch (e) {
      error = e?.message || String(e);
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    if (typeof window === 'undefined') return;
    const f = window.freighter;
    hasFreighter = !!f;

    // If Freighter is already connected, auto-connect and load balances.
    try {
      if (f?.isConnected && (await f.isConnected())) {
        await connectWallet();
      }
    } catch {
      // ignore auto-connect errors; user can click the button
    }
  });
</script>

<main class="app">
  <header>
    <h1>Stake Token</h1>
    <p class="subtitle">DeFi staking on Stellar testnet · 10% APY</p>
  </header>

  {#if !connected}
    <section class="card connect">
      {#if !hasFreighter}
        <p>
          Freighter wallet is not detected. Install it from
          <a href="https://www.freighter.app/" target="_blank" rel="noopener">freighter.app</a>, switch to
          <strong>Testnet</strong>, then refresh this page.
        </p>
      {:else}
        <p>Connect your Freighter wallet to view balances and stake.</p>
      {/if}
      <button on:click={connectWallet} disabled={loading}>
        Connect Freighter
      </button>
      {#if error}
        <p class="error">{error}</p>
      {/if}
    </section>
  {:else}
    <section class="card wallet">
      <p class="address">{address.slice(0, 12)}…{address.slice(-8)}</p>
      <button class="secondary" on:click={refreshBalances} disabled={loading}>Refresh</button>
    </section>

    <section class="card balances">
      <h2>Balances</h2>
      <ul>
        <li><strong>Balance</strong> <span>{balance.toString()}</span></li>
        <li><strong>Staked</strong> <span>{stakedBalance.toString()}</span></li>
        <li><strong>Pending reward</strong> <span>{pendingReward.toString()}</span></li>
      </ul>
    </section>

    <section class="card actions">
      <h2>Stake</h2>
      <p>Lock tokens to earn ~10% APY. Rewards are paid on unstake.</p>
      <div class="row">
        <input
          type="text"
          placeholder="Amount"
          bind:value={stakeAmount}
          disabled={loading}
        />
        <button on:click={stake} disabled={loading || !stakeAmount}>Stake</button>
      </div>

      <h2>Unstake</h2>
      <p>Withdraw staked tokens and claim accrued rewards.</p>
      <div class="row">
        <input
          type="text"
          placeholder="Amount"
          bind:value={unstakeAmount}
          disabled={loading}
        />
        <button on:click={unstake} disabled={loading || !unstakeAmount}>Unstake</button>
      </div>
    </section>

    {#if error}
      <p class="error">{error}</p>
    {/if}
    {#if txHash}
      <p class="success">
        Tx: <a href="https://stellar.expert/explorer/testnet/tx/{txHash}" target="_blank" rel="noopener">{txHash.slice(0, 12)}…</a>
      </p>
    {/if}
  {/if}

  {#if !CONTRACT_ID}
    <p class="hint">Set <code>VITE_CONTRACT_ID</code> in <code>.env</code> (your deployed contract ID).</p>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: system-ui, -apple-system, sans-serif;
    background: #0d1117;
    color: #e6edf3;
    min-height: 100vh;
  }
  .app {
    max-width: 420px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }
  header {
    text-align: center;
    margin-bottom: 2rem;
  }
  h1 {
    font-size: 1.75rem;
    margin: 0 0 0.25rem 0;
  }
  .subtitle {
    color: #8b949e;
    font-size: 0.9rem;
    margin: 0;
  }
  .card {
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 1.25rem;
    margin-bottom: 1rem;
  }
  .card h2 {
    font-size: 1rem;
    margin: 0 0 0.75rem 0;
    color: #58a6ff;
  }
  .card p {
    margin: 0 0 0.75rem 0;
    color: #8b949e;
    font-size: 0.9rem;
  }
  .address {
    color: #e6edf3 !important;
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
  }
  .row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }
  input {
    flex: 1;
    padding: 0.6rem 0.75rem;
    border: 1px solid #30363d;
    border-radius: 8px;
    background: #0d1117;
    color: #e6edf3;
    font-size: 1rem;
  }
  button {
    padding: 0.6rem 1rem;
    border-radius: 8px;
    border: none;
    font-weight: 600;
    cursor: pointer;
    background: #238636;
    color: #fff;
    font-size: 0.95rem;
  }
  button:hover:not(:disabled) {
    background: #2ea043;
  }
  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  button.secondary {
    background: #21262d;
    color: #e6edf3;
  }
  button.secondary:hover:not(:disabled) {
    background: #30363d;
  }
  .connect {
    text-align: center;
  }
  .connect button {
    margin-top: 0.5rem;
  }
  .balances ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .balances li {
    display: flex;
    justify-content: space-between;
    padding: 0.4rem 0;
    border-bottom: 1px solid #21262d;
  }
  .balances li:last-child {
    border-bottom: none;
  }
  .error {
    color: #f85149;
    font-size: 0.9rem;
    margin-top: 0.5rem;
  }
  .success {
    color: #3fb950;
    font-size: 0.9rem;
  }
  .success a {
    color: #58a6ff;
  }
  .hint {
    color: #8b949e;
    font-size: 0.85rem;
    margin-top: 1rem;
  }
  code {
    background: #21262d;
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
    font-size: 0.85em;
  }
</style>
