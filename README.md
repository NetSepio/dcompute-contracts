# dcompute-contracts

## Overview

Cyrene AI is a decentralized platform for managing and executing tasks performed by AI agents on the Solana blockchain. This innovative smart contract provides a secure, transparent system for requesting AI computation, monitoring task progress, and automating payments for completed AI services.

## Features

- **Secure Payment Escrow**: Funds are locked until AI task completion
- **Automated Task States**: Clear tracking of AI task progression
- **On-chain Task Metadata**: Transparent storage of task specifications
- **Trustless Execution**: No intermediaries between users and AI agents
- **Programmatic Integration**: Easy to integrate with existing AI systems
- **Payment Automation**: Instant settlement upon task completion

## Smart Contract Architecture

The Cyrene AI smart contract implements a complete workflow for task creation, AI agent assignment, execution, and payment using native SOL.

### Task Statuses

| Status | Description |
| --- | --- |
| **Pending** | Task created and awaiting AI agent allocation |
| **Started** | Task assigned to specific AI agent |
| **Processing** | AI agent actively working on computation |
| **Done** | Task completed and payment released |

### Contract Functions

### For Task Creators

- **initialize_job**: Create a new AI task with specifications and payment amount
- **complete_job**: Confirm task completion and release payment
- **refund_job**: Cancel a task and retrieve escrowed funds

### For AI Agents/Node Operators

- **start_job**: Allocate AI resources to a specific task
- **mark_processing**: Update task status to "in computation"

## Usage Guide

### For End Users

1. **Creating an AI Task**
    
    ```bash
    solana call initialize_job <job_id> <task_specifications> <payment_amount>
    
    ```
    
    This creates a new AI task with a unique ID, detailed specifications, and payment amount in SOL.
    
2. **Completing a Task**
    
    ```bash
    solana call complete_job <task_pubkey>
    
    ```
    
    When satisfied with the AI results, mark it complete to release payment to the AI agent.
    
3. **Requesting a Refund**
    
    ```bash
    solana call refund_job <task_pubkey>
    
    ```
    
    If needed, cancel a task before it's complete to recover your funds.
    

### For AI Service Providers

1. **Accepting a Task**
    
    ```bash
    solana call start_job <task_pubkey>
    
    ```
    
    Assign computational resources to an available task.
    
2. **Updating Task Status**
    
    ```bash
    solana call mark_processing <task_pubkey>
    
    ```
    
    Signal that AI computation has begun on the task.
    

## Technical Details

- **Framework**: Anchor Framework
- **Language**: Rust
- **Network**: Solana
- **Storage**: On-chain account data
- **Payment**: Native SOL

## Example Workflow

1. User creates a task requesting an AI-generated image for 0.5 SOL
2. AI service node accepts the task, changing status to "Started"
3. Node updates status to "Processing" when computation begins
4. Upon completion, results are delivered to the user
5. User verifies results and marks the task "Done", releasing 0.5 SOL to the AI service provider

## Use Cases

- **Decentralized AI Image Generation**: Pay-per-use image creation services
- **Natural Language Processing**: Translation, summarization, content creation
- **Data Analysis**: On-demand processing of large datasets
- **AI Model Training**: Distributed machine learning tasks
- **Autonomous Agent Tasks**: Execution of complex multi-step AI workflows

## Security Considerations

- All payment operations use secure escrow mechanics
- Permission controls ensure only authorized nodes can claim tasks
- State verification prevents invalid task transitions
- Balance verification ensures payment integrity

## License

MIT

---

For API documentation, integration guides, or to become an AI service provider, please visit [https://cyrene.ai](https://cyrene.ai/) or join our Discord community.

© 2025 Cyrene AI - Decentralized Artificial Intelligence Solutions