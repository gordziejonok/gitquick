use crate::{
    config,
    git::{commit, commit_fixup, get_current_branch, get_log, get_staged_changes},
};
use crossterm::terminal;
use inquire::{Confirm, Select, Text};
use regex::Regex;
use std::{error::Error, fmt::Display};

#[derive(Default)]
struct ConventionalCommit {
    r#type: String,
    scope: Option<String>,
    description: String,
    body: Option<String>,
    footers: Vec<String>,
    breaking_change: bool,
}

impl Display for ConventionalCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.r#type)?;

        if let Some(scope) = &self.scope {
            write!(f, "({})", scope)?;
        }

        if self.breaking_change {
            write!(f, "!")?;
        }

        write!(f, ": {}", self.description)?;

        if let Some(body) = &self.body {
            writeln!(f)?;
            writeln!(f, "{}", body)?;
        }

        if !self.footers.is_empty() {
            writeln!(f)?;
            for footer in &self.footers {
                writeln!(f, "{}", footer)?;
            }
        }

        Ok(())
    }
}

pub fn run_commit(
    commit_config: config::Commit,
    fixup: bool,
    amend: bool,
) -> Result<(), Box<dyn Error>> {
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

    let message = if commit_config.conventional {
        create_conventional_commit(commit_config)?
    } else {
        // TODO refactor this
        let commit = Text::new("Enter commit message:").prompt()?;
        let trailer = if commit_config.ticket {
            format!("\n\n{}", get_ticket()?)
        } else {
            "".to_owned()
        };
        format!("{}{}", commit, trailer)
    };

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

#[allow(clippy::field_reassign_with_default)]
fn create_conventional_commit(config: config::Commit) -> Result<String, Box<dyn Error>> {
    let mut commit = ConventionalCommit::default();

    commit.r#type = Select::new("Select commit type", config.types).prompt()?;

    let scope = Text::new("Scope:").prompt()?;
    if !scope.trim().is_empty() {
        commit.scope = Some(scope);
    }

    commit.description = Text::new("Enter commit message:").prompt()?;

    let body = Text::new("Body:").prompt()?;
    if !body.trim().is_empty() {
        commit.body = Some(body);
    };

    if config.ticket {
        let ticket = get_ticket()?;
        commit.footers.push(ticket);
    }

    let is_breaking_change = Confirm::new("BREAKING CHANGE?")
        .with_default(false)
        .prompt()?;
    if is_breaking_change {
        commit.breaking_change = true;
        let breaking_change_desc = Text::new("Breaking change description:").prompt()?;
        commit
            .footers
            .push(format!("BREAKING CHANGE: {}", breaking_change_desc));
    }

    Ok(commit.to_string())
}

fn get_ticket() -> Result<String, Box<dyn Error + 'static>> {
    let re = Regex::new(r"[A-Z]+-[0-9]+")?;
    let branch = get_current_branch()?;
    let ticket = re
        .find(&branch.name)
        .map(|regex_match| format!("Refs: {}", regex_match.as_str()))
        .unwrap_or_default();
    Ok(ticket)
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

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn format_conventional_commit() {
        let commit = ConventionalCommit {
            r#type: "feat".to_owned(),
            scope: Some("api".to_owned()),
            description: "send an email to the customer when a product is shipped".to_owned(),
            body: None,
            footers: vec![],
            breaking_change: true,
        };

        let message = format!("{}", commit);

        assert_eq!(
            message,
            "feat(api)!: send an email to the customer when a product is shipped"
        );
    }
}
