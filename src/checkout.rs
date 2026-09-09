use std::error::Error;

use crate::git_operations;
use inquire::{Confirm, Select};
use regex::Regex;

pub fn run_checkout(create_new: bool) -> Result<(), Box<dyn Error>> {
    if create_new {
        let branch_input = inquire::Text::new("Enter branch name").prompt()?;

        let re = Regex::new(r" +").unwrap();
        let branch_name = re.replace_all(branch_input.trim(), "-");

        let should_checkout =
            Confirm::new(&format!("Create and checkout to: \"{}\"?", branch_name))
                .with_default(true)
                .prompt()?;

        if should_checkout {
            git_operations::create_and_checkout_branch(&branch_name)?;
            println!("✅ Created and switched to new branch '{}'", branch_name);
        } else {
            println!("❌ Commit canceled or failed to get user confirmation.");
        }
        Ok(())
    } else {
        let branches = git_operations::get_branches()?;
        let available_branches = branches.iter().filter(|b| !b.head).collect();

        let selected_branch =
            Select::new("Select branch to checkout", available_branches).prompt()?;

        git_operations::checkout_branch(&selected_branch.name)?;

        Ok(())
    }
}
