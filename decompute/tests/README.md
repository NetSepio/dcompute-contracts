
# 🧰 Solana Job Escrow CLI

This is a command-line interface (CLI) for interacting with an Anchor-based Solana smart contract that handles job escrows. The CLI allows you to:

- Initialize a job
- Start a job
- Mark the job as processing
- Complete the job

Supports interaction with the Solana Devnet via a configurable RPC and wallet setup.

---

## 📦 Prerequisites

- [Node.js](https://nodejs.org/) v18+
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli)
- A funded Solana devnet wallet at `~/.config/solana/id.json`
- An `.env` file with worker mnemonic

---

## ⚙️ Configuration

Create a `.env` file in the root of the project with the following variables:

```env
# Worker account mnemonic (used to start/complete jobs)
WORKER_MNEMONIC="your twelve word mnemonic here"

# Optional: RPC endpoint
RPC_URL="https://api.devnet.solana.com"
```

> ✅ The CLI uses your local Solana CLI keypair (`~/.config/solana/id.json`) as the **owner** wallet.

---

## 🚀 Usage

Each command follows this format:

```bash
node cli.js <function_name> [arguments...]
```

### ✅ Available Functions

| Function Name     | Arguments                     | Description                                |
| ----------------- | ----------------------------- | ------------------------------------------ |
| `initialize_job`  | `<jobId> <metadata> <amount>` | Initializes a job with metadata and amount |
| `start_job`       | `<jobId>`                     | Worker starts the job                      |
| `mark_processing` | `<jobId>`                     | Worker marks the job as "processing"       |
| `complete_job`    | `<jobId>`                     | Completes the job and triggers payout      |

---

## 🧪 Examples

### Initialize a job (owner)

```bash
node cli.js initialize_job 1 "Compress PDF files" 1000000
```

### Start the job (worker)

```bash
node cli.js start_job 1
```

### Mark the job as processing (worker)

```bash
node cli.js mark_processing 1
```

### Complete the job (owner & worker)

```bash
node cli.js complete_job 1
```

---

## 🔐 Security Notes

* **DO NOT hardcode** sensitive values like mnemonics in the code.
* Always load keys and RPC endpoints from environment variables or secure secret management systems.

---

## 📄 IDL

This CLI expects an Anchor-generated IDL file located as:

```bash
./escrow_job.json
```

Make sure the IDL matches your deployed smart contract.

---

## 🧠 How It Works

* The CLI uses the Anchor IDL to generate transaction instructions.
* The **owner wallet** is read from `~/.config/solana/id.json`.
* The **worker wallet** is derived from the provided mnemonic in `.env`.
* PDAs are derived for each job using the Anchor convention (`["job", job_id]`).
* All interactions use the `@coral-xyz/anchor` and `@solana/web3.js` packages.

---
