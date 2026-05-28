use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;


pub fn process_refund_instruction(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint_a,
        escrow_account,
        maker_a_ata,
        escrow_ata,
        _associated_token_program
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    {
        if !escrow_account.owned_by(&crate::ID) {
            return Err(ProgramError::UninitializedAccount);
        }

        let bump = data[0];
        let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];

        let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
        assert_eq!(escrow_account_pda, *escrow_account.address().as_array());

        let escrow_data = Escrow::from_account_info(escrow_account)?;

        /*

        if escrow_data.maker != *maker.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        if escrow_data.mint_a != *mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        if escrow_data.mint_b != *mint_b.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        */

        // check A
        {
            let maker_a_ata_state = pinocchio_token::state::Account::from_account_view(maker_a_ata)?;
            if maker_a_ata_state.owner() != maker.address() {
                return Err(ProgramError::IllegalOwner);
            }

        }

        let amount_to_give = u64::from_le_bytes(escrow_data.amount_to_give);

        let bump_bytes = [bump];
        let signer_seeds = [
            Seed::from(b"escrow"),
            Seed::from(maker.address().as_array()),
            Seed::from(bump_bytes.as_ref()),
        ];
        let signer = Signer::from(&signer_seeds);


        pinocchio_token::instructions::Transfer::new(escrow_ata, maker_a_ata, escrow_account, amount_to_give)
            .invoke_signed(&[signer])?;

        let escrow_lamports = escrow_account.lamports();
        let new_maker_lamports = maker.lamports() + escrow_lamports;

        maker.set_lamports(new_maker_lamports);
        escrow_account.set_lamports(0);

        //let _ = drop(escrow_data);
        let _ = escrow_account.close();
    }
    Ok(())
}