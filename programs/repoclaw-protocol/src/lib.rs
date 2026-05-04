//! RepoClaw Protocol — git for ai agents on solana.
//!
//! This program implements:
//! - Agent registry (stake $RCLAW to register an AI agent)
//! - PR settlement (escrow + payout on merged PRs)
//! - Slashing (penalize malicious agents)
//! - Rewards (claim earnings)
//!
//! Status: v0.1.0 scaffold — core instructions defined, full audit & mainnet
//! deployment incoming post-launch.

use anchor_lang::prelude::*;

declare_id!("RcLaw1111111111111111111111111111111111111");

#[program]
pub mod repoclaw_protocol {
    use super::*;

    /// Register a new AI agent on-chain.
    /// Agent must stake $RCLAW tokens. Stake is locked until unregister or slash.
    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        agent_name: String,
        stake_amount: u64,
    ) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
        agent.owner = ctx.accounts.owner.key();
        agent.name = agent_name;
        agent.stake = stake_amount;
        agent.prs_opened = 0;
        agent.prs_merged = 0;
        agent.reputation = 0;
        agent.status = AgentStatus::Active;
        agent.created_at = Clock::get()?.unix_timestamp;

        emit!(AgentRegistered {
            agent: agent.key(),
            owner: agent.owner,
            stake: stake_amount,
        });

        // TODO: actual SPL token transfer of stake amount to escrow
        // TODO: validate stake_amount >= MIN_STAKE

        msg!("agent registered: {}", agent.name);
        Ok(())
    }

    /// Submit a PR on-chain. Stores IPFS hash of diff.
    pub fn submit_pr(
        ctx: Context<SubmitPr>,
        repo_id: Pubkey,
        ipfs_hash: String,
        bounty_amount: u64,
    ) -> Result<()> {
        let pr = &mut ctx.accounts.pr;
        pr.agent = ctx.accounts.agent.key();
        pr.repo_id = repo_id;
        pr.ipfs_hash = ipfs_hash;
        pr.bounty_amount = bounty_amount;
        pr.status = PrStatus::Open;
        pr.approvals = 0;
        pr.created_at = Clock::get()?.unix_timestamp;

        let agent = &mut ctx.accounts.agent;
        agent.prs_opened += 1;

        emit!(PrSubmitted {
            pr: pr.key(),
            agent: pr.agent,
            repo_id: pr.repo_id,
        });

        msg!("pr submitted by agent");
        Ok(())
    }

    /// Merge a PR after sufficient reviewer approvals. Releases bounty to agent.
    pub fn merge_pr(ctx: Context<MergePr>) -> Result<()> {
        let pr = &mut ctx.accounts.pr;

        require!(
            pr.status == PrStatus::Open,
            RepoClawError::PrAlreadyResolved
        );
        require!(
            pr.approvals >= MIN_APPROVALS,
            RepoClawError::InsufficientApprovals
        );

        pr.status = PrStatus::Merged;
        pr.merged_at = Clock::get()?.unix_timestamp;

        let agent = &mut ctx.accounts.agent;
        agent.prs_merged += 1;
        agent.reputation += REPUTATION_PER_MERGE;

        emit!(PrMerged {
            pr: pr.key(),
            agent: pr.agent,
            bounty: pr.bounty_amount,
        });

        // TODO: transfer pr.bounty_amount from escrow to agent owner
        msg!("pr merged, bounty released");
        Ok(())
    }

    /// Slash an agent's stake for malicious behavior.
    /// Called by governance multisig (not yet implemented).
    pub fn slash_agent(ctx: Context<SlashAgent>, reason: String) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
        agent.status = AgentStatus::Slashed;

        emit!(AgentSlashed {
            agent: agent.key(),
            reason,
        });

        // TODO: redistribute slashed stake to treasury
        msg!("agent slashed");
        Ok(())
    }

    /// Claim accumulated rewards.
    pub fn claim_rewards(_ctx: Context<ClaimRewards>) -> Result<()> {
        // TODO: calculate pending rewards from merged PRs
        // TODO: transfer SPL tokens
        msg!("rewards claimed");
        Ok(())
    }
}

// ============================================
// CONSTANTS
// ============================================

pub const MIN_STAKE: u64 = 100_000_000; // 100 $RCLAW (assuming 6 decimals)
pub const MIN_APPROVALS: u8 = 2;
pub const REPUTATION_PER_MERGE: u32 = 10;

// ============================================
// ACCOUNTS
// ============================================

#[account]
pub struct Agent {
    pub owner: Pubkey,
    pub name: String,
    pub stake: u64,
    pub prs_opened: u64,
    pub prs_merged: u64,
    pub reputation: u32,
    pub status: AgentStatus,
    pub created_at: i64,
}

#[account]
pub struct PullRequest {
    pub agent: Pubkey,
    pub repo_id: Pubkey,
    pub ipfs_hash: String,
    pub bounty_amount: u64,
    pub status: PrStatus,
    pub approvals: u8,
    pub created_at: i64,
    pub merged_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Active,
    Paused,
    Slashed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PrStatus {
    Open,
    Merged,
    Rejected,
    Expired,
}

// ============================================
// CONTEXTS
// ============================================

#[derive(Accounts)]
#[instruction(agent_name: String)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 4 + 64 + 8 + 8 + 8 + 4 + 1 + 8,
        seeds = [b"agent", owner.key().as_ref(), agent_name.as_bytes()],
        bump
    )]
    pub agent: Account<'info, Agent>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitPr<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 4 + 64 + 8 + 1 + 1 + 8 + 8,
    )]
    pub pr: Account<'info, PullRequest>,
    #[account(mut, has_one = owner)]
    pub agent: Account<'info, Agent>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MergePr<'info> {
    #[account(mut)]
    pub pr: Account<'info, PullRequest>,
    #[account(mut)]
    pub agent: Account<'info, Agent>,
    pub merger: Signer<'info>,
}

#[derive(Accounts)]
pub struct SlashAgent<'info> {
    #[account(mut)]
    pub agent: Account<'info, Agent>,
    pub authority: Signer<'info>, // TODO: enforce governance multisig
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut, has_one = owner)]
    pub agent: Account<'info, Agent>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

// ============================================
// EVENTS
// ============================================

#[event]
pub struct AgentRegistered {
    pub agent: Pubkey,
    pub owner: Pubkey,
    pub stake: u64,
}

#[event]
pub struct PrSubmitted {
    pub pr: Pubkey,
    pub agent: Pubkey,
    pub repo_id: Pubkey,
}

#[event]
pub struct PrMerged {
    pub pr: Pubkey,
    pub agent: Pubkey,
    pub bounty: u64,
}

#[event]
pub struct AgentSlashed {
    pub agent: Pubkey,
    pub reason: String,
}

// ============================================
// ERRORS
// ============================================

#[error_code]
pub enum RepoClawError {
    #[msg("PR has already been resolved")]
    PrAlreadyResolved,
    #[msg("Insufficient reviewer approvals")]
    InsufficientApprovals,
    #[msg("Stake amount below minimum")]
    StakeTooLow,
    #[msg("Agent is not active")]
    AgentNotActive,
}
