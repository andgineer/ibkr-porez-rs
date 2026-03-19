use anyhow::{Result, bail};
use rust_decimal::Decimal;

use super::{output, resolve_ids, run_bulk, run_bulk_resolved, validate_non_negative_decimal};

pub fn run(declaration_id: Vec<String>, tax: Option<Decimal>) -> Result<()> {
    if let Some(raw_amount) = tax {
        let amount = validate_non_negative_decimal(raw_amount)?;
        let ids = resolve_ids(declaration_id);
        if ids.len() > 1 {
            bail!("--tax can only be used with a single declaration ID");
        }
        return run_bulk_resolved(&ids, |m, id| {
            m.set_assessed_tax(id, amount, true)?;
            output::success(&format!("Paid: {id} ({amount:.2} RSD recorded)"));
            Ok(())
        });
    }

    run_bulk(declaration_id, |m, id| {
        m.pay(&[id])?;
        output::success(&format!("Paid: {id}"));
        Ok(())
    })
}
