use anyhow::Result;

use super::{output, run_bulk};
use crate::RevertTarget;

#[allow(clippy::needless_pass_by_value)]
pub fn run(declaration_id: Vec<String>, to: RevertTarget) -> Result<()> {
    run_bulk(declaration_id, |m, id| match to {
        RevertTarget::Draft => {
            m.revert(&[id])?;
            output::success(&format!("Reverted {id} to draft"));
            Ok(())
        }
        RevertTarget::Submitted => {
            m.submit(&[id])?;
            output::success(&format!("Submitted: {id}"));
            Ok(())
        }
    })
}
