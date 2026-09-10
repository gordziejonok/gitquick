use crate::{
    config::Commit,
    git_operations::{commit, commit_fixup, get_current_branch, get_log, get_staged_changes},
};
use crossterm::terminal;
use inquire::{Confirm, Select, Text};
use regex::Regex;
use std::error::Error;

pub fn run_commit(commit_config: Commit, fixup: bool, amend: bool) -> Result<(), Box<dyn Error>> {
    let staged = get_staged_changes()?;

    if staged.is_empty() {
        println!("No staged files found.");
        return Ok(());
    }

    if fixup {
        run_fixup()?;
        return Ok(());
    }

    if amend {
        let log = get_log()?;
        if let Some(log) = log.first() {
            print_in_box(&log.message)?;

            let should_commit = Confirm::new("Commit with previous message?")
                .with_default(true)
                .prompt()?;

            if should_commit {
                commit(&log.message, amend)?;
                println!("✅ Commit successful!");
                return Ok(());
            }
        }
    }

    let mut commit_header = if commit_config.conventional {
        get_type_and_scope(commit_config.types)?
    } else {
        String::new()
    };

    let ticket = if commit_config.ticket {
        let re = Regex::new(r"[A-Z]+-[0-9]+").unwrap();
        let branch = get_current_branch().unwrap();
        re.find(&branch)
            .map(|regex_match| format!(" ({})", regex_match.as_str()))
            .unwrap_or_default()
    } else {
        "".to_string()
    };

    let user_input = Text::new("Enter commit message:").prompt()?;

    let body = if commit_config.conventional {
        let mut body_text = Text::new("Body:").prompt()?;
        if !body_text.is_empty() {
            body_text = format!("\n\n{}", body_text);
        };
        body_text
    } else {
        String::new()
    };

    let footer = if commit_config.conventional {
        let is_breaking_change = Confirm::new("BREAKING CHANGE?")
            .with_default(false)
            .prompt()?;

        let breaking_change = if is_breaking_change {
            let breaking_change_desc = Text::new("Breaking change description:").prompt()?;
            commit_header.push('!');
            format!("\n\nBREAKING CHANGE: {}", breaking_change_desc)
        } else {
            String::new()
        };
        commit_header.push_str(": ");
        breaking_change
    } else {
        String::new()
    };

    let message = format!(
        "{}{}{}{}{}",
        commit_header, user_input, ticket, body, footer
    );

    print_in_box(&message)?;

    let should_commit = Confirm::new("Commit?").with_default(true).prompt()?;

    if should_commit {
        commit(&message, amend)?;
        println!("✅ Commit successful!");
    } else {
        println!("❌ Commit canceled or failed to get user confirmation.");
    }

    Ok(())
}

fn run_fixup() -> Result<(), Box<dyn Error>> {
    let commits = get_log()?;

    let selected_commit = Select::new("Select commit to fixup:", commits).prompt()?;
    commit_fixup(&selected_commit.hash)?;
    println!("✅ Fixup successful!");
    Ok(())
}

pub fn print_in_box(message: &str) -> Result<(), Box<dyn Error>> {
    let lines: Vec<&str> = message.lines().collect();
    let mut max_len = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let (width, _) = terminal::size()?;
    let width = usize::from(width);

    if max_len >= width {
        max_len = width - 4;
    }

    let mut new_lines: Vec<&str> = vec![];
    for line in lines {
        if line.len() < width - 4 {
            new_lines.push(line);
        } else {
            let mut result = Vec::new();
            let mut start = 0;
            let mut count = 0;

            for (i, _) in line.char_indices() {
                if count == max_len {
                    result.push(&line[start..i]);
                    start = i;
                    count = 0;
                }
                count += 1;
            }

            if start < line.len() {
                result.push(&line[start..]);
            }
            new_lines.append(&mut result);
        }
    }

    println!("┌{}┐", "─".repeat(max_len + 2));
    for line in new_lines {
        println!("│ {:width$} │", line, width = max_len);
    }
    println!("└{}┘", "─".repeat(max_len + 2));
    Ok(())
}

fn get_type_and_scope(commit_types: Vec<String>) -> Result<String, Box<dyn Error>> {
    let selected_type = Select::new("Select commit type", commit_types).prompt()?;

    let mut scope = Text::new("Scope:").prompt()?;

    if !scope.is_empty() {
        scope = format!("({})", scope);
    }

    Ok(format!("{}{}", selected_type, scope))
}
