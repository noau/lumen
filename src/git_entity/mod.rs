use commit::Commit;
use diff::Diff;
use indoc::formatdoc;

use crate::provider::LumenProvider;

pub mod commit;
pub mod diff;

#[derive(Debug, Clone)]
pub enum GitEntity {
    Commit(Commit),
    Diff(Diff),
}

pub fn get_pathspecs() -> Vec<String> {
    let mut args = vec!["--".to_string(), ".".to_string()];

    // Add user defined exclusions from .lumenignore
    if let Ok(content) = std::fs::read_to_string(".lumenignore") {
        args.extend(parse_lumenignore(&content));
    }

    args
}

fn parse_lumenignore(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed_trailing = line.trim_end();
            if trimmed_trailing.is_empty() || trimmed_trailing.trim_start().starts_with('#') {
                None
            } else {
                Some(format!(":(exclude){}", trimmed_trailing))
            }
        })
        .collect()
}

impl GitEntity {
    pub fn format_static_details(&self, provider: &LumenProvider) -> String {
        match self {
            GitEntity::Commit(commit) => formatdoc! {"
                # Entity: Commit
                # Provider: {provider}
                `commit {hash}` | {author} <{email}> | {date}

                {message}
                -----",
                hash = commit.full_hash,
                author = commit.author_name,
                email = commit.author_email,
                date = commit.date,
                message = commit.message,
                provider = provider
            },
            GitEntity::Diff(Diff::WorkingTree { staged, .. }) => formatdoc! {"
                # Entity: Working Tree Diff{staged}
                # Provider: {provider}",
                staged = if *staged { " (staged)" } else { "" }
            },
            GitEntity::Diff(Diff::CommitsRange { from, to, .. }) => formatdoc! {"
                # Entity: Range
                `{from}` -> `{to}`
                # Provider: {provider}
            "},
        }
    }
}

impl AsRef<Commit> for GitEntity {
    fn as_ref(&self) -> &Commit {
        match self {
            GitEntity::Commit(commit) => commit,
            _ => panic!("Not a Commit"),
        }
    }
}

impl AsRef<Diff> for GitEntity {
    fn as_ref(&self) -> &Diff {
        match self {
            GitEntity::Diff(diff) => diff,
            _ => panic!("Not a Diff"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lumenignore_basic() {
        let content = "target\n*.log\n# a comment\n  \nconfig.json";
        let expected = vec![
            ":(exclude)target".to_string(),
            ":(exclude)*.log".to_string(),
            ":(exclude)config.json".to_string(),
        ];
        assert_eq!(parse_lumenignore(content), expected);
    }

    #[test]
    fn test_parse_lumenignore_no_trailing_comments() {
        // In .gitignore, # later in the line is part of the pattern
        let content = "config.json # not a comment";
        let expected = vec![":(exclude)config.json # not a comment".to_string()];
        assert_eq!(parse_lumenignore(content), expected);
    }

    #[test]
    fn test_parse_lumenignore_empty() {
        assert!(parse_lumenignore("").is_empty());
        assert!(parse_lumenignore("\n\n").is_empty());
        assert!(parse_lumenignore("# comment only").is_empty());
    }

    #[test]
    fn test_parse_lumenignore_with_hash_in_filename() {
        // This currently fails to preserve the # in filename because it treats it as a comment
        let content = "my#file.txt";
        let expected = vec![":(exclude)my#file.txt".to_string()];
        assert_eq!(parse_lumenignore(content), expected);
    }

    #[test]
    fn test_parse_lumenignore_leading_spaces() {
        let content = "  leading_space_file";
        let expected = vec![":(exclude)  leading_space_file".to_string()];
        assert_eq!(parse_lumenignore(content), expected);
    }
}
