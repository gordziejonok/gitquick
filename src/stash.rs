use std::error::Error;

use inquire::{MultiSelect, Text};

use crate::git::{self};

pub fn run_stash(push: bool) -> Result<(), Box<dyn Error>> {
    if push {
        let changes = git::get_unstaged_changes()?;

        if changes.is_empty() {
            println!("No untracked or modified files found.");
            return Ok(());
        }

        let selected_changes = MultiSelect::new("Select changes to stash:", changes).prompt()?;

        if selected_changes.is_empty() {
            println!("No files selected.");
            return Ok(());
        }

        let user_input = Text::new("Enter stash message:").prompt()?;

        git::push_stash(selected_changes, &user_input)?;
        println!("✅ Stash successful!");
    }
    Ok(())
}
