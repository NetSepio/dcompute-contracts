use anchor_lang::prelude::*;
use anchor_lang::Space;

declare_id!("5fVWymoBhqFMCTPkaeiT6APu3dJi5pysEQ2YzV9JMJ6D");

pub const MAX_METADATA_LEN: usize = 256;
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

#[program]
pub mod cyrene_job {
    use super::*;

    pub fn initialize_job(
        ctx: Context<InitializeJob>,
        job_id: u64,
        metadata: String,
        amount: u64,
    ) -> Result<()> {
        // Transfer the escrow amount into the newly‑created job account
        // (the lamports came from `owner` when the account was created)
        let job = &mut ctx.accounts.job;
        job.job_id = job_id;
        job.metadata = metadata;
        job.owner = ctx.accounts.owner.key();
        job.worker = Pubkey::default();
        job.amount = amount;
        job.status = JobStatus::Pending;
        Ok(())
    }

    pub fn start_job(ctx: Context<StartJob>) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.status == JobStatus::Pending, EscrowError::InvalidState);
        job.worker = ctx.accounts.worker.key();
        job.status = JobStatus::Started;
        Ok(())
    }

    pub fn mark_processing(ctx: Context<MarkProcessing>) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(
            ctx.accounts.worker.key() == job.worker,
            EscrowError::Unauthorized
        );
        require!(job.status == JobStatus::Started, EscrowError::InvalidState);
        job.status = JobStatus::Processing;
        Ok(())
    }

    pub fn complete_job(ctx: Context<CompleteJob>) -> Result<()> {
        let job_ai = ctx.accounts.job.to_account_info();
        let worker_ai = ctx.accounts.worker.to_account_info();
        let amount = ctx.accounts.job.amount;

        require!(
            ctx.accounts.owner.key() == ctx.accounts.job.owner,
            EscrowError::Unauthorized
        );
        require!(job_ai.lamports() >= amount, EscrowError::InsufficientEscrow);
        require!(
            ctx.accounts.job.status != JobStatus::Done,
            EscrowError::InvalidState
        );

        // Pay the worker
        **job_ai.try_borrow_mut_lamports()? -= amount;
        **worker_ai.try_borrow_mut_lamports()? += amount;

        // Mark as done (job account can optionally be closed here)
        ctx.accounts.job.status = JobStatus::Done;
        Ok(())
    }

    pub fn refund_job(ctx: Context<RefundJob>) -> Result<()> {
        let job_ai = ctx.accounts.job.to_account_info();
        let owner_ai = ctx.accounts.owner.to_account_info();
        let amount = ctx.accounts.job.amount;

        require!(
            ctx.accounts.owner.key() == ctx.accounts.job.owner,
            EscrowError::Unauthorized
        );
        require!(
            ctx.accounts.job.status != JobStatus::Done,
            EscrowError::InvalidState
        );
        require!(job_ai.lamports() >= amount, EscrowError::InsufficientEscrow);

        // Refund the owner
        **job_ai.try_borrow_mut_lamports()? -= amount;
        **owner_ai.try_borrow_mut_lamports()? += amount;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(job_id: u64, metadata: String, amount: u64)]
pub struct InitializeJob<'info> {
    #[account(
        init,
        payer = owner,
        space = Job::INIT_SPACE + ANCHOR_DISCRIMINATOR_SIZE,
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StartJob<'info> {
    #[account(mut, has_one = owner)]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub worker: Signer<'info>,
    /// CHECK: The job owner (not required to be a signer for this ix)
    pub owner: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct MarkProcessing<'info> {
    #[account(mut)]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub worker: Signer<'info>,
}

#[derive(Accounts)]
pub struct CompleteJob<'info> {
    #[account(mut)]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: worker can be any account to receive payment
    #[account(mut)]
    pub worker: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RefundJob<'info> {
    #[account(mut)]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,    // waiting for a worker
    Started,    // accepted by a worker
    Processing, // worker marked in progress
    Done,       // owner marked complete & paid
}

impl anchor_lang::Space for JobStatus {
    const INIT_SPACE: usize = 1; // 1 byte for enum discriminator
}

#[account]
#[derive(InitSpace)]
pub struct Job {
    pub job_id: u64,
    #[max_len(MAX_METADATA_LEN)]
    pub metadata: String,
    pub owner: Pubkey,  // job poster
    pub worker: Pubkey, // set once someone accepts
    pub amount: u64,    // lamports escrowed
    pub status: JobStatus,
}

#[error_code]
pub enum EscrowError {
    #[msg("Metadata field is too long.")]
    MetadataTooLong,
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("Invalid job state transition.")]
    InvalidState,
    #[msg("Escrow account no longer holds the expected amount.")]
    InsufficientEscrow,
}
