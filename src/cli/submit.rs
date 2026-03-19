use anyhow::Result;
use ibkr_porez::models::DeclarationStatus;

use super::{output, run_bulk};

pub fn run(declaration_id: Vec<String>) -> Result<()> {
    run_bulk(declaration_id, |m, id| {
        m.submit(&[id])?;
        let msg = match m.get_status(id) {
            Some(DeclarationStatus::Finalized) => {
                format!("Finalized: {id} (no tax to pay)")
            }
            Some(DeclarationStatus::Pending) => {
                format!("Submitted: {id} (pending tax authority assessment)")
            }
            _ => {
                format!("Submitted: {id}")
            }
        };
        output::success(&msg);
        Ok(())
    })
}
