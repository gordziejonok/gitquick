use std::error::Error;

use inquire::{MultiSelect, Text};

use crate::git::{self, Change};

pub fn run_stash(push: bool) -> Result<(), Box<dyn Error>> {
    if push {
        let changes = git::get_unstaged_changes()?;

        if changes.is_empty() {
            println!("No untracked or modified files found.");
            return Ok(());
        }

        let mut selected_files = Vec::<Change>::new();

        let selected_unstaged = MultiSelect::new("Select changes to stash:", changes)
            .prompt()?;

        if selected_unstaged.is_empty() && selected_files.is_empty() {
            println!("No files selected.");
            return Ok(());
        }

        let user_input = Text::new("Enter stash message:")
            .prompt()?;

        selected_files.extend(selected_unstaged);

        git::push_stash(selected_files, &user_input)?;
        println!("✅ Stash successful!");
    }
    Ok(())
}
