use std::error::Error;

use crate::git::rebase;

use inquire::Select;

use crate::git::get_log;

pub fn run_rebase(interactive: bool) -> Result<(), Box<dyn Error>> {
    let commit_log = get_log()?;
    let selected_commit = Select::new("Select commit to rebase:", commit_log).prompt()?;

    rebase(&selected_commit.hash, interactive)?;

    Ok(())
}
