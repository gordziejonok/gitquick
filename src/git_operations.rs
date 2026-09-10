use core::fmt;
use git2::Repository;
use std::{
    error::Error,
    fmt::Display,
    process::{Command, Stdio},
};

#[derive(Clone)]
pub struct Change {
    pub path: String,
    status: Status,
}

#[derive(Clone)]
enum Status {
    WtNew,
    WtModified,
    WtDeleted,
    IndexNew,
    IndexModified,
    IndexDeleted,
}

#[derive(Debug)]
enum StatusParseError {
    UnknownStatus(String),
}

impl Display for StatusParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownStatus(status) => write!(f, "Unknown status: {status}"),
        }
    }
}

impl std::error::Error for StatusParseError {}

impl TryFrom<&str> for Status {
    type Error = StatusParseError;

    fn try_from(value: &str) -> Result<Status, Self::Error> {
        match value {
            "??" => Ok(Status::WtNew),
            " M" => Ok(Status::WtModified),
            " D" => Ok(Status::WtDeleted),
            "A " => Ok(Status::IndexNew),
            "M " => Ok(Status::IndexModified),
            "D " => Ok(Status::IndexDeleted),
            _ => Err(StatusParseError::UnknownStatus(value.to_owned())),
        }
    }
}

impl Change {
    pub fn is_worktree(&self) -> bool {
        matches!(
            self.status,
            Status::WtNew | Status::WtModified | Status::WtDeleted
        )
    }
    pub fn is_staged(&self) -> bool {
        matches!(
            self.status,
            Status::IndexNew | Status::IndexModified | Status::IndexDeleted
        )
    }
}

pub struct CommitLog {
    pub hash: String,
    pub message: String,
}

impl fmt::Display for CommitLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status_str = match self.status {
            Status::WtNew => "new",
            Status::WtModified => "modified",
            Status::WtDeleted => "deleted",
            _ => "?",
        };
        write!(f, "{}: {}", status_str, self.path)
    }
}

pub fn get_log() -> Result<Vec<CommitLog>, String> {
    let output = Command::new("git")
        .arg("log")
        .arg("--pretty=format:%H%x00%s")
        .output()
        .map_err(|e| format!("Failed to log messages: {}", e))?;
    let commits = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| {
            let data: Vec<&str> = s.split('\0').collect();
            CommitLog {
                hash: data[0].to_string(),
                message: data[1].to_string(),
            }
        })
        .collect();
    Ok(commits)
}

pub fn push_stash(selected_files: Vec<Change>, message: &str) -> Result<(), Box<dyn Error>> {
    let paths: Vec<&String> = selected_files.iter().map(|change| &change.path).collect();
    let mut cmd = Command::new("git");
    cmd.arg("stash")
        .arg("push")
        .arg("--include-untracked")
        .args(paths);

    if !message.is_empty() {
        cmd.arg("--message").arg(message);
    }

    let _output = cmd
        .output()
        .map_err(|e| format!("Failed to stash files: {}", e))?;
    Ok(())
}

pub struct Branch {
    pub name: String,
    #[allow(dead_code)]
    pub upstream: Option<String>,
    pub gone: bool,
    pub head: bool,
    pub ahead: u16,
    pub behind: u16,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;

        if self.ahead != 0 || self.behind != 0 {
            write!(f, " ")?;

            if self.ahead != 0 {
                write!(f, "↑{}", self.ahead)?;
            }

            if self.behind != 0 {
                write!(f, "↓{}", self.behind)?;
            }
        }

        if self.gone {
            write!(f, " [gone]")?;
        }
        Ok(())
    }
}

pub fn get_branches() -> Result<Vec<Branch>, Box<dyn Error>> {
    let head_out = Command::new("git")
        .arg("symbolic-ref")
        .arg("refs/remotes/origin/HEAD")
        .output()?;

    if !head_out.status.success() {
        return Err(String::from_utf8_lossy(&head_out.stderr).into());
    }

    let binding = String::from_utf8(head_out.stdout)?;
    let head = binding.trim();

    let ahead_behind = format!("ahead-behind:{}", head);

    let field_names = [
        "refname:short",
        "upstream:short",
        "upstream:track,nobracket",
        &ahead_behind,
        "HEAD",
    ];

    let git_format = field_names
        .iter()
        .map(|f| format!("%({})", f))
        .collect::<Vec<String>>()
        .join("%00");

    let output = Command::new("git")
        .arg("for-each-ref")
        .arg("--format")
        .arg(git_format)
        .arg("refs/heads/")
        .arg("refs/remotes/")
        .output()?;

    let output_str = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let branches: Vec<Branch> = output_str
        .lines()
        .map(|l| {
            let info: Vec<&str> = l.split("\0").collect();
            let (ahead, behind) = info[3].split_once(" ").unwrap();
            Branch {
                name: info[0].to_owned(),
                upstream: (!info[1].is_empty()).then(|| info[1].to_owned()),
                gone: info[2] == "gone",
                ahead: ahead.parse().unwrap_or_default(),
                behind: behind.parse().unwrap_or_default(),
                head: info[4].is_empty(),
            }
        })
        .collect();
    Ok(branches)
}

#[allow(dead_code)]
fn fetch_with_prune() -> Result<(), std::io::Error> {
    let status = Command::new("git").arg("fetch").arg("--prune").status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("git fetch with prune failed"))
    }
}

pub fn rebase(hash: &str, interactive: bool) -> Result<(), Box<dyn Error>> {
    let mut rebase_command = Command::new("git");
    rebase_command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("rebase")
        .arg("--autosquash")
        .arg(hash);

    if interactive {
        rebase_command.arg("--interactive");
    }

    rebase_command.status()?;

    Ok(())
}

pub fn get_repository() -> Result<Repository, git2::Error> {
    Repository::discover(".")
}

fn get_changes() -> Result<Vec<Change>, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(Box::new(std::io::Error::other(err.to_string())));
    }

    let commits: Vec<Change> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| -> Result<Change, StatusParseError> {
            Ok(Change {
                path: line[3..].trim().to_owned(),
                status: line[..2].try_into()?,
            })
        })
        .collect::<Result<Vec<Change>, StatusParseError>>()?;

    Ok(commits)
}

pub fn get_unstaged_changes() -> Result<Vec<Change>, Box<dyn Error>> {
    let changes = get_changes()?;
    let unstaged: Vec<Change> = changes
        .iter()
        .filter(|change| change.is_worktree())
        .cloned()
        .collect();
    Ok(unstaged)
}

pub fn get_staged_changes() -> Result<Vec<Change>, Box<dyn Error>> {
    let changes = get_changes()?;
    let staged: Vec<Change> = changes
        .iter()
        .filter(|change| change.is_staged())
        .cloned()
        .collect();
    Ok(staged)
}

pub fn add_files(selected_files: Vec<Change>) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("add")
        .args(selected_files.iter().map(|f| &f.path))
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(Box::new(std::io::Error::other(err.to_string())))
    } else {
        Ok(())
    }
}

pub fn delete_branch(name: &str, flag: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("branch")
        .arg(flag)
        .arg(name)
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(Box::new(std::io::Error::other(err.to_string())))
    } else {
        Ok(())
    }
}

pub fn commit(message: &str, amend: bool) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("git");

    command.arg("commit");

    if amend {
        command.arg("--amend");
    };

    let output = command.arg("-m").arg(message).output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(Box::new(std::io::Error::other(err.to_string())))
    } else {
        Ok(())
    }
}

pub fn commit_fixup(hash: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("commit")
        .arg("--fixup")
        .arg(hash)
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(Box::new(std::io::Error::other(err.to_string())))
    } else {
        Ok(())
    }
}

pub fn revert(hash: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("revert")
        .arg("--no-commit")
        .arg(hash)
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(Box::new(std::io::Error::other(err.to_string())))
    } else {
        Ok(())
    }
}

pub fn checkout_branch(branch: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git").arg("checkout").arg(branch).output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(Box::new(std::io::Error::other(err.to_string())))
    } else {
        Ok(())
    }
}

pub fn get_current_branch() -> Result<String, git2::Error> {
    let repo = get_repository()?;

    let head = repo.head()?;
    head.shorthand()
        .map(|s| s.to_string())
        .ok_or_else(|| git2::Error::from_str("Failed to get branch name"))
}

pub fn create_and_checkout_branch(branch_name: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(branch_name)
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(Box::new(std::io::Error::other(err.to_string())))
    } else {
        Ok(())
    }
}
