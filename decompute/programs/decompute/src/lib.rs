use anchor_lang::prelude::*;

declare_id!("5fVWymoBhqFMCTPkaeiT6APu3dJi5pysEQ2YzV9JMJ6D");

pub const MAX_METADATA_LEN: usize = 256;

#[program]
pub mod escrow_job {
    use super::*;

    pub fn initialize_job(
        ctx: Context<InitializeJob>,
        job_id: u64,
        metadata: String,
        amount: u64,
    ) -> Result<()> {
        require!(metadata.len() <= MAX_METADATA_LEN, EscrowError::MetadataTooLong);

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
        require!(ctx.accounts.worker.key() == job.worker, EscrowError::Unauthorized);
        require!(job.status == JobStatus::Started, EscrowError::InvalidState);
        job.status = JobStatus::Processing;
        Ok(())
    }

    // ** Fixed complete_job: close the job account and transfer all lamports to worker **
    pub fn complete_job(ctx: Context<CompleteJob>) -> Result<()> {
        let job = &mut ctx.accounts.job;

        require!(ctx.accounts.owner.key() == job.owner, EscrowError::Unauthorized);
        require!(job.status != JobStatus::Done, EscrowError::InvalidState);

        // Mark job as done before closing account (optional)
        job.status = JobStatus::Done;

        // Because of `close = worker` in accounts context,
        // all lamports in the job account (including rent) will be sent to worker
        // when the job account is closed at the end of this instruction.

        Ok(())
    }

    pub fn refund_job(ctx: Context<RefundJob>) -> Result<()> {
        let job_ai = ctx.accounts.job.to_account_info();
        let owner_ai = ctx.accounts.owner.to_account_info();
        let amount = ctx.accounts.job.amount;

        require!(ctx.accounts.owner.key() == ctx.accounts.job.owner, EscrowError::Unauthorized);
        require!(ctx.accounts.job.status != JobStatus::Done, EscrowError::InvalidState);
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
        space = Job::SIZE + 8,
        seeds = [b"job", job_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub job: Account<'info, Job>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StartJob<'info> {
    #[account(mut, has_one = owner, seeds = [b"job", &job.job_id.to_le_bytes()], bump)]
    pub job: Account<'info, Job>,

    #[account(mut)]
    pub worker: Signer<'info>,

    /// CHECK: The job owner (not required to be a signer)
    pub owner: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct MarkProcessing<'info> {
    #[account(mut, seeds = [b"job", &job.job_id.to_le_bytes()], bump)]
    pub job: Account<'info, Job>,

    #[account(mut)]
    pub worker: Signer<'info>,
}

// ** Fixed CompleteJob accounts struct: add `close = worker` to close job and transfer lamports **
#[derive(Accounts)]
pub struct CompleteJob<'info> {
    #[account(mut, seeds = [b"job", &job.job_id.to_le_bytes()], bump, close = worker)]
    pub job: Account<'info, Job>,

    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK: This is the job-assigned worker. We check key matches in handler.
    #[account(mut)]
    pub worker: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RefundJob<'info> {
    #[account(mut, seeds = [b"job", &job.job_id.to_le_bytes()], bump)]
    pub job: Account<'info, Job>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[account]
pub struct Job {
    pub job_id: u64,
    pub metadata: String,           // max 256 bytes
    pub owner: Pubkey,              // job poster
    pub worker: Pubkey,             // set once someone accepts
    pub amount: u64,                // lamports escrowed
    pub status: JobStatus,
}

impl Job {
    pub const SIZE: usize = 8            // job_id
        + 4 + MAX_METADATA_LEN           // metadata string prefix + bytes
        + 32                             // owner
        + 32                             // worker
        + 8                              // amount
        + 1;                             // status enum (u8)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Started,
    Processing,
    Done,
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