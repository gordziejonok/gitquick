use std::error::Error;

use inquire::{Confirm, Select};

use crate::{
    commit::print_in_box,
    git::{commit, get_log, revert},
};

pub fn run_revert() -> Result<(), Box<dyn Error>> {
    let commits = get_log()?;

    let selected_commit = Select::new("Select commit to revert:", commits).prompt()?;
    let message = format!(
        "revert: \"{}\"\nThis reverts commit: {}",
        selected_commit.message, selected_commit.hash
    );
    print_in_box(&message)?;

    let should_commit = Confirm::new("Revert?").with_default(true).prompt()?;

    if should_commit {
        revert(&selected_commit.hash)?;
        commit(&message, false)?;
        println!("✅ Revert successful!");
    } else {
        println!("❌ Revert canceled or failed to get user confirmation.");
    }

    Ok(())
}
