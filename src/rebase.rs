use crate::git_operations::rebase;

use inquire::Select;

use crate::git_operations::get_log;

pub fn run_rebase(interactive: bool) -> Result<(), String> {
    let commit_log = get_log()?;
    let selected_commit = Select::new("Select commit to rebase:", commit_log)
        .prompt()
        .map_err(|e| format!("Failed to rebase commit: {}", e))?;

    rebase(&selected_commit.hash, interactive)
        .map_err(|e| format!("Failed to rebase commit: {}", e))?;

    Ok(())
}
