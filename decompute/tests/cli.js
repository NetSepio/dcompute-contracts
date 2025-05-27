import pkg from '@coral-xyz/anchor';
const { AnchorProvider, Program, Wallet, Idl, BN } = pkg;
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import idl from './escrow_job.json' with { type: 'json' };
import fs from 'fs';
import * as bip39 from 'bip39';
import { derivePath } from 'ed25519-hd-key';
import os from 'os';
import dotenv from 'dotenv';
dotenv.config(); // Load .env if present

// === CONFIG ===
const keypairPath = `${os.homedir()}/.config/solana/id.json`;
const RPC_URL = 'https://api.devnet.solana.com';
const WORKER_MNEMONIC = process.env.WORKER_MNEMONIC;

if (!WORKER_MNEMONIC) {
  console.error("❌ Please set WORKER_MNEMONIC in your environment variables.");
  process.exit(1);
}

// === UTILS ===
const secret = Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, 'utf-8')));
const ownerKeypair = Keypair.fromSecretKey(secret);

const getKeypairFromMnemonic = async (mnemonic) => {
  const seed = await bip39.mnemonicToSeed(mnemonic);
  const path = "m/44'/501'/0'/0'";
  const derivedSeed = derivePath(path, seed.toString('hex')).key;
  return Keypair.fromSeed(derivedSeed);
};

const getProgram = (payer) => {
  const connection = new Connection(RPC_URL, 'processed');
  const wallet = new (class {
    constructor(payer) {
      this.payer = payer;
    }
    async signTransaction(tx) {
      tx.partialSign(this.payer);
      return tx;
    }
    async signAllTransactions(txs) {
      return txs.map((tx) => {
        tx.partialSign(this.payer);
        return tx;
      });
    }
    get publicKey() {
      return this.payer.publicKey;
    }
  })(payer);

  const provider = new AnchorProvider(connection, wallet, {});
  return new Program(idl, provider);
};

const getJobPDA = (jobId, program) => {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("job"), jobId.toArrayLike(Buffer, "le", 8)],
    program.programId
  );
};

// === FUNCTIONS ===
const functions = {
  async initialize_job(jobIdRaw, metadata, amountRaw) {
    const jobId = new BN(parseInt(jobIdRaw));
    const amount = new BN(parseInt(amountRaw));
    const program = getProgram(ownerKeypair);
    const [jobPDA] = getJobPDA(jobId, program);

    const tx = await program.methods
      .initializeJob(jobId, metadata, amount)
      .accounts({
        job: jobPDA,
        owner: ownerKeypair.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([ownerKeypair])
      .rpc();
    console.log("✅ initialize_job tx:", tx);
  },

  async start_job(jobIdRaw) {
    const jobId = new BN(parseInt(jobIdRaw));
    const program = getProgram(ownerKeypair);
    const [jobPDA] = getJobPDA(jobId, program);
    const workerKeypair = await getKeypairFromMnemonic(WORKER_MNEMONIC);

    const tx = await program.methods
      .startJob()
      .accounts({
        job: jobPDA,
        worker: workerKeypair.publicKey,
        owner: ownerKeypair.publicKey,
      })
      .signers([workerKeypair])
      .rpc();
    console.log("✅ start_job tx:", tx);
  },

  async mark_processing(jobIdRaw) {
    const jobId = new BN(parseInt(jobIdRaw));
    const program = getProgram(ownerKeypair);
    const [jobPDA] = getJobPDA(jobId, program);
    const workerKeypair = await getKeypairFromMnemonic(WORKER_MNEMONIC);

    const tx = await program.methods
      .markProcessing()
      .accounts({
        job: jobPDA,
        worker: workerKeypair.publicKey,
      })
      .signers([workerKeypair])
      .rpc();
    console.log("✅ mark_processing tx:", tx);
  },

  async complete_job(jobIdRaw) {
    const jobId = new BN(parseInt(jobIdRaw));
    const program = getProgram(ownerKeypair);
    const [jobPDA] = getJobPDA(jobId, program);
    const workerKeypair = await getKeypairFromMnemonic(WORKER_MNEMONIC);

    const tx = await program.methods
      .completeJob()
      .accounts({
        job: jobPDA,
        owner: ownerKeypair.publicKey,
        worker: workerKeypair.publicKey,
      })
      .signers([ownerKeypair])
      .rpc();
    console.log("✅ complete_job tx:", tx);
  }
};

// === CLI ENTRY ===
(async () => {
  const [, , func, ...args] = process.argv;
  if (!functions[func]) {
    console.error(`Unknown function: ${func}`);
    process.exit(1);
  }
  await functions[func](...args);
})();
