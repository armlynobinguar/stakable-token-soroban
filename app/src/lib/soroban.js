/**
 * Soroban contract helpers for the stakable token.
 * Uses @stellar/stellar-sdk and connects to Stellar testnet.
 */

import * as StellarSdk from '@stellar/stellar-sdk';

const TESTNET_RPC = 'https://soroban-testnet.stellar.org';
const TESTNET_NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET;

/**
 * @param {string} [rpcUrl]
 * @returns {StellarSdk.SorobanRpc.Server}
 */
export function getServer(rpcUrl = TESTNET_RPC) {
  return new StellarSdk.SorobanRpc.Server(rpcUrl);
}

/**
 * @param {string} contractId - Contract ID (C...)
 * @param {string} [rpcUrl]
 * @returns {StellarSdk.Contract}
 */
export function getContract(contractId, rpcUrl = TESTNET_RPC) {
  return new StellarSdk.Contract(contractId);
}

/**
 * @param {string} address - G... or C... strkey
 * @returns {StellarSdk.xdr.ScVal}
 */
export function addressToScVal(address) {
  return new StellarSdk.Address(address).toScVal();
}

/**
 * @param {number|string} n - amount (integer)
 * @returns {StellarSdk.xdr.ScVal}
 */
export function i128ToScVal(n) {
  const big = BigInt(n);
  const hi = big / (2n ** 64n);
  const lo = big % (2n ** 64n);
  return StellarSdk.xdr.ScVal.scvI128(
    new StellarSdk.xdr.Int128Parts({ hi: StellarSdk.xdr.Int64.fromString(hi.toString()), lo: StellarSdk.xdr.Uint64.fromString(lo.toString()) })
  );
}

/**
 * Read-only: get balance for an address.
 * @param {string} contractId
 * @param {string} address
 * @param {string} [rpcUrl]
 * @returns {Promise<bigint>}
 */
export async function getBalance(contractId, address, rpcUrl = TESTNET_RPC) {
  const server = getServer(rpcUrl);
  const contract = getContract(contractId, rpcUrl);
  const tx = new StellarSdk.TransactionBuilder(await getSourceAccount(server), {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase: TESTNET_NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call('balance', addressToScVal(address)))
    .setTimeout(180)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (StellarSdk.SorobanRpc.isSimulationError(sim)) {
    throw new Error(sim.error?.message ?? JSON.stringify(sim.error));
  }
  const result = StellarSdk.SorobanRpc.getReturnValue(sim);
  return result != null ? scValToI128(result) : 0n;
}

/**
 * Read-only: get staked balance.
 */
export async function getStakedBalance(contractId, address, rpcUrl = TESTNET_RPC) {
  const server = getServer(rpcUrl);
  const contract = getContract(contractId, rpcUrl);
  const tx = new StellarSdk.TransactionBuilder(await getSourceAccount(server), {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase: TESTNET_NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call('staked_balance', addressToScVal(address)))
    .setTimeout(180)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (StellarSdk.SorobanRpc.isSimulationError(sim)) {
    throw new Error(sim.error?.message ?? JSON.stringify(sim.error));
  }
  const result = StellarSdk.SorobanRpc.getReturnValue(sim);
  return result != null ? scValToI128(result) : 0n;
}

/**
 * Read-only: get pending reward.
 */
export async function getPendingReward(contractId, address, rpcUrl = TESTNET_RPC) {
  const server = getServer(rpcUrl);
  const contract = getContract(contractId, rpcUrl);
  const tx = new StellarSdk.TransactionBuilder(await getSourceAccount(server), {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase: TESTNET_NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call('pending_reward', addressToScVal(address)))
    .setTimeout(180)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (StellarSdk.SorobanRpc.isSimulationError(sim)) {
    throw new Error(sim.error?.message ?? JSON.stringify(sim.error));
  }
  const result = StellarSdk.SorobanRpc.getReturnValue(sim);
  return result != null ? scValToI128(result) : 0n;
}

/**
 * @param {StellarSdk.xdr.ScVal} scval
 * @returns {bigint}
 */
function scValToI128(scval) {
  const sw = scval.switch();
  if (sw.name !== 'scvI128') return 0n;
  const parts = scval.i128();
  const hi = BigInt(parts.hi().toString());
  const lo = BigInt(parts.lo().toString());
  return hi * (2n ** 64n) + lo;
}

async function getSourceAccount(server) {
  const pair = StellarSdk.Keypair.random();
  return new StellarSdk.Account(pair.publicKey(), '0');
}

/**
 * Build, simulate, assemble, sign with Freighter, and send a contract invocation.
 * @param {string} contractId
 * @param {string} method - 'stake' | 'unstake'
 * @param {string} address - signer (G...)
 * @param {string|number} amount
 * @param {{ signTransaction: (tx: string) => Promise<string> }} freighter
 * @param {string} [rpcUrl]
 * @returns {Promise<string>} transaction hash
 */
export async function invokeContract(contractId, method, address, amount, freighter, rpcUrl = TESTNET_RPC) {
  const server = getServer(rpcUrl);
  const contract = getContract(contractId, rpcUrl);

  const account = await server.getAccount(address);
  const source = new StellarSdk.Account(address, account.sequence);

  const tx = new StellarSdk.TransactionBuilder(source, {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase: TESTNET_NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(method, addressToScVal(address), i128ToScVal(amount))
    )
    .setTimeout(180)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (StellarSdk.SorobanRpc.isSimulationError(sim)) {
    throw new Error(sim.error?.message ?? JSON.stringify(sim));
  }
  const assemble =
    typeof StellarSdk.assembleTransaction === 'function'
      ? StellarSdk.assembleTransaction
      : (raw, simulation) => server.assembleTransaction(raw, simulation);
  const prepared = assemble(tx, sim);
  const signedXdr = await freighter.signTransaction(prepared.toXDR(), {
    networkPassphrase: TESTNET_NETWORK_PASSPHRASE,
    network: 'testnet',
  });
  const signed = StellarSdk.TransactionBuilder.fromXDR(signedXdr, TESTNET_NETWORK_PASSPHRASE);
  const result = await server.sendTransaction(signed);
  if (result.errorResult) {
    throw new Error(result.errorResult);
  }
  return result.hash;
}
