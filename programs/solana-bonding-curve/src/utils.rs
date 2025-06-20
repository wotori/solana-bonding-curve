use anchor_lang::prelude::*;
use anchor_lang::ToAccountInfo;
use anchor_spl::token::{transfer, Token, Transfer};

pub fn transfer_commission<'info>(
    token_program: &Program<'info, Token>,
    escrow: &AccountInfo<'info>,
    agent_account: &AccountInfo<'info>,
    treasury_account: &AccountInfo<'info>,
    signer: &AccountInfo<'info>,
    signer_seeds: &[&[u8]],
    total_amount: u64,
    commission_bps: u64,
) -> Result<()> {
    let commission_amount = total_amount
        .checked_mul(commission_bps)
        .ok_or_else(|| error!(crate::errors::CustomError::MathOverflow))?
        .checked_div(10_000)
        .ok_or_else(|| error!(crate::errors::CustomError::MathOverflow))?;

    let agent_share = commission_amount / 2;
    let treasury_share = commission_amount
        .checked_sub(agent_share)
        .ok_or_else(|| error!(crate::errors::CustomError::MathOverflow))?;

    let signer_seeds_arr = &[signer_seeds];
    let cpi_ctx_agent = CpiContext::new_with_signer(
        token_program.to_account_info(),
        Transfer {
            from: escrow.clone(),
            to: agent_account.clone(),
            authority: signer.clone(),
        },
        signer_seeds_arr,
    );
    transfer(cpi_ctx_agent, agent_share)?;

    let cpi_ctx_treasury = CpiContext::new_with_signer(
        token_program.to_account_info(),
        Transfer {
            from: escrow.clone(),
            to: treasury_account.clone(),
            authority: signer.clone(),
        },
        signer_seeds_arr,
    );
    transfer(cpi_ctx_treasury, treasury_share)?;

    Ok(())
}
