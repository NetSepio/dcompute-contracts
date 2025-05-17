import { AnchorProvider, Program, Wallet, Idl, BN } from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import idl from '../target/idl/escrow_job.json';
import fs from 'fs';
import * as bip39 from 'bip39';
import { derivePath } from 'ed25519-hd-key';

// === LOAD OWNER KEYPAIR FROM LOCAL FILE ===
const keypairPath = '/home/evadshell/.config/solana/id.json';
const secret = Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, 'utf-8')));
const ownerKeypair = Keypair.fromSecretKey(secret);

// === DERIVE WORKER KEYPAIR FROM MNEMONIC ===
const workerMnemonic = "diagram engage adjust youth toy draft vehicle turtle own unaware demand hour";
const getKeypairFromMnemonic = async (mnemonic: string): Promise<Keypair> => {
  const seed = await bip39.mnemonicToSeed(mnemonic);
  const path = "m/44'/501'/0'/0'";
  const derivedSeed = derivePath(path, seed.toString('hex')).key;
  return Keypair.fromSeed(derivedSeed);
};

(async () => {
  const workerKeypair = await getKeypairFromMnemonic(workerMnemonic);

  // === ANCHOR SETUP ===
  const connection = new Connection('https://api.devnet.solana.com', 'processed');
  const wallet = new (class implements Wallet {
    constructor(readonly payer: Keypair) {}
    signTransaction = async (tx) => { tx.partialSign(this.payer); return tx; };
    signAllTransactions = async (txs) => txs.map((tx) => { tx.partialSign(this.payer); return tx; });
    get publicKey() { return this.payer.publicKey; }
  })(ownerKeypair);
  const provider = new AnchorProvider(connection, wallet, {});
  const program = new Program(idl as Idl, provider);

  // === UTILITY: DERIVE PDA ===
  function getJobPDA(jobId: BN): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("job"), jobId.toArrayLike(Buffer, "le", 8)], // ✅ Fixed seed encoding
      program.programId
    );
  }

  // === JOB FLOW ===
  const jobIdNum = Math.floor(Math.random() * 1_000_000);
  const jobId = new BN(jobIdNum);
  const metadata = "Test job using mnemonic-based worker key";
  const amount = new BN(1_000_000); // 0.001 SOL

  const [jobPDA] = getJobPDA(jobId);

  console.log("Creating Job PDA:", jobPDA.toBase58());
  console.log("Worker pubkey:", workerKeypair.publicKey.toBase58());

  // 1. Initialize Job (owner signs)
  const tx1 = await program.methods
    .initializeJob(jobId, metadata, amount)
    .accounts({
      job: jobPDA,
      owner: ownerKeypair.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([ownerKeypair])
    .rpc();
  console.log("✅ initialize_job tx:", tx1);

  // 2. Start Job (worker signs)
  const tx2 = await program.methods
    .startJob()
    .accounts({
      job: jobPDA,
      worker: workerKeypair.publicKey,
      owner: ownerKeypair.publicKey,
    })
    .signers([workerKeypair])
    .rpc();
  console.log("✅ start_job tx:", tx2);

  // 3. Mark Processing (worker signs)
  const tx3 = await program.methods
    .markProcessing()
    .accounts({
      job: jobPDA,
      worker: workerKeypair.publicKey,
    })
    .signers([workerKeypair])
    .rpc();
  console.log("✅ mark_processing tx:", tx3);

  // 4. Complete Job (owner signs)
  try {
    const tx4 = await program.methods
      .completeJob()
      .accounts({
        job: jobPDA,
        owner: ownerKeypair.publicKey,
        worker: workerKeypair.publicKey,
      })
      .signers([ownerKeypair])
      .rpc();
    console.log("✅ complete_job tx:", tx4);
    console.log("✅ complete_job tx:", tx4);
  } catch (e: any) {
    console.error("❌ complete_job failed:", e.message);
    if (e.logs) console.error("Logs:", e.logs);
  }
  
})();
