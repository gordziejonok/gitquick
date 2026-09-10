use std::error::Error;

use crate::git::{self, Change};
use inquire::MultiSelect;

pub fn stage_files() -> Result<(), Box<dyn Error>> {
    let changes = git::get_unstaged_changes()?;

    if changes.is_empty() {
        println!("No untracked or modified files found.");
        return Ok(());
    }

    let mut selected_files = Vec::<Change>::new();

    let selected_unstaged = MultiSelect::new("Select changes to add:", changes).prompt()?;

    if selected_unstaged.is_empty() && selected_files.is_empty() {
        println!("No files selected.");
        return Ok(());
    }

    selected_files.extend(selected_unstaged);

    git::add_files(selected_files)?;

    println!("✅ Added files successfuly!");
    Ok(())
}
