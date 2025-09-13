use crate::types::*;
use git2::{Repository, Oid, Commit, DiffOptions, DiffFormat, Delta};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};
use log::{info, warn, error, debug};

pub struct HistoryAnalyzer {
    repositories: HashMap<PathBuf, Repository>,
}

impl HistoryAnalyzer {
    pub fn new() -> Self {
        Self {
            repositories: HashMap::new(),
        }
    }

    pub fn analyze_repository(&self, project_path: &Path) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        let repo = self.get_or_open_repository(project_path)?;
        let mut history_entries = Vec::new();

        // Walk through all commits
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            
            if let Ok(entry) = self.analyze_commit(&repo, &commit, project_path) {
                history_entries.push(entry);
            }
        }

        info!("Analyzed {} commits for repository: {:?}", history_entries.len(), project_path);
        Ok(history_entries)
    }

    fn get_or_open_repository(&self, path: &Path) -> Result<Repository, Box<dyn std::error::Error>> {
        Ok(Repository::open(path)?)
    }

    fn analyze_commit(&self, repo: &Repository, commit: &Commit, _project_path: &Path) -> Result<HistoryEntry, Box<dyn std::error::Error>> {
        let commit_id = commit.id().to_string();
        let author = commit.author();
        let committer = commit.committer();
        let message = commit.message().unwrap_or("").to_string();
        
        let timestamp = Utc.timestamp_opt(commit.time().seconds(), 0)
            .single()
            .unwrap_or_else(|| Utc::now());

        // Analyze file changes
        let changes = self.analyze_commit_changes(repo, commit)?;

        Ok(HistoryEntry {
            id: Uuid::new_v4(),
            file_id: Uuid::new_v4(), // This would be mapped to actual file IDs
            commit_id,
            author: author.name().unwrap_or("Unknown").to_string(),
            author_email: author.email().unwrap_or("").to_string(),
            committer: committer.name().unwrap_or("Unknown").to_string(),
            committer_email: committer.email().unwrap_or("").to_string(),
            message,
            timestamp,
            changes,
        })
    }

    fn analyze_commit_changes(&self, repo: &Repository, commit: &Commit) -> Result<Vec<FileChange>, Box<dyn std::error::Error>> {
        let mut changes = Vec::new();
        
        // Get the tree for this commit
        let tree = commit.tree()?;
        
        // Get parent commit if it exists
        if let Ok(parent) = commit.parent(0) {
            let parent_tree = parent.tree()?;
            
            // Create diff between parent and current commit
            let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;
            
            // Analyze each delta
            diff.foreach(
                &mut |delta, _progress| {
                    let old_path = delta.old_file().path().map(|p| p.to_path_buf());
                    let new_path = delta.new_file().path().map(|p| p.to_path_buf());
                    let status = delta.status();
                    
                    let change_type = match status {
                        git2::Delta::Added => ChangeType::Added,
                        git2::Delta::Deleted => ChangeType::Deleted,
                        git2::Delta::Modified => ChangeType::Modified,
                        git2::Delta::Renamed => ChangeType::Renamed,
                        git2::Delta::Copied => ChangeType::Copied,
                        _ => return true,
                    };
                    
                    changes.push(FileChange {
                        change_type,
                        old_path,
                        new_path,
                        lines_added: 0,
                        lines_deleted: 0,
                    });
                    true
                },
                None,
                None,
                None,
            )?;
        } else {
            // This is the initial commit, all files are added
            tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
                if entry.kind() == Some(git2::ObjectType::Blob) {
                    let path = PathBuf::from(root).join(entry.name().unwrap_or(""));
                    changes.push(FileChange {
                        change_type: ChangeType::Added,
                        old_path: None,
                        new_path: Some(path),
                        lines_added: 0, // Would need to analyze file content
                        lines_deleted: 0,
                    });
                }
                git2::TreeWalkResult::Ok
            })?;
        }

        Ok(changes)
    }

    fn analyze_diff_delta(&self, delta: &git2::DiffDelta) -> Option<FileChange> {
        let old_file = delta.old_file();
        let new_file = delta.new_file();
        
        let old_path = old_file.path().map(|p| p.to_path_buf());
        let new_path = new_file.path().map(|p| p.to_path_buf());
        
        let change_type = match delta.status() {
            git2::Delta::Added => ChangeType::Added,
            git2::Delta::Deleted => ChangeType::Deleted,
            git2::Delta::Modified => ChangeType::Modified,
            git2::Delta::Renamed => ChangeType::Renamed,
            git2::Delta::Copied => ChangeType::Copied,
            _ => return None,
        };

        Some(FileChange {
            change_type,
            old_path,
            new_path,
            lines_added: 0, // TODO: Analyze actual line changes
            lines_deleted: 0,
        })
    }

    pub fn get_file_history(&self, project_path: &Path, file_path: &Path) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        let repo = self.get_or_open_repository(project_path)?;
        let mut history_entries = Vec::new();

        // Walk through commits that affected this file
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            
            if self.commit_affects_file(&repo, &commit, file_path)? {
                if let Ok(entry) = self.analyze_commit(&repo, &commit, project_path) {
                    history_entries.push(entry);
                }
            }
        }

        Ok(history_entries)
    }

    fn commit_affects_file(&self, repo: &Repository, commit: &Commit, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        let tree = commit.tree()?;
        
        // Check if file exists in this commit
        if tree.get_path(file_path).is_ok() {
            // If this is not the first commit, check if file was changed
            if let Ok(parent) = commit.parent(0) {
                let parent_tree = parent.tree()?;
                let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;
                
                let mut file_changed = false;
                diff.foreach(
                    &mut |delta, _progress| {
                        if let Some(path) = delta.new_file().path() {
                            if path == file_path {
                                file_changed = true;
                            }
                        }
                        if let Some(path) = delta.old_file().path() {
                            if path == file_path {
                                file_changed = true;
                            }
                        }
                        true
                    },
                    None,
                    None,
                    None,
                )?;
                
                Ok(file_changed)
            } else {
                // First commit, file was added
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    pub fn get_blame_info(&self, project_path: &Path, file_path: &Path) -> Result<Vec<BlameInfo>, Box<dyn std::error::Error>> {
        let repo = self.get_or_open_repository(project_path)?;
        let blame = repo.blame_file(file_path, None)?;
        
        let mut blame_info = Vec::new();
        
        for i in 0..blame.len() {
            if let Some(hunk) = blame.get_index(i) {
                let commit_id = hunk.final_commit_id();
                
                if let Ok(commit) = repo.find_commit(commit_id) {
                    let author = commit.author();
                    let timestamp = Utc.timestamp_opt(commit.time().seconds(), 0)
                        .single()
                        .unwrap_or_else(|| Utc::now());
                    
                    blame_info.push(BlameInfo {
                        line_number: hunk.final_start_line(),
                        commit_id: commit_id.to_string(),
                        author: author.name().unwrap_or("Unknown").to_string(),
                        author_email: author.email().unwrap_or("").to_string(),
                        timestamp,
                        message: commit.message().unwrap_or("").to_string(),
                    });
                }
            }
        }
        
        Ok(blame_info)
    }

    pub fn get_branches(&self, project_path: &Path) -> Result<Vec<BranchInfo>, Box<dyn std::error::Error>> {
        let repo = self.get_or_open_repository(project_path)?;
        let mut branches = Vec::new();
        
        // Get local branches
        let local_branches = repo.branches(Some(git2::BranchType::Local))?;
        for branch_result in local_branches {
            if let Ok((branch, _)) = branch_result {
                if let Some(name) = branch.name()? {
                    let is_head = branch.is_head();
                    let commit_id = branch.get().target().map(|oid| oid.to_string());
                    
                    branches.push(BranchInfo {
                        name: name.to_string(),
                        is_current: is_head,
                        is_remote: false,
                        commit_id,
                    });
                }
            }
        }
        
        // Get remote branches
        let remote_branches = repo.branches(Some(git2::BranchType::Remote))?;
        for branch_result in remote_branches {
            if let Ok((branch, _)) = branch_result {
                if let Some(name) = branch.name()? {
                    let commit_id = branch.get().target().map(|oid| oid.to_string());
                    
                    branches.push(BranchInfo {
                        name: name.to_string(),
                        is_current: false,
                        is_remote: true,
                        commit_id,
                    });
                }
            }
        }
        
        Ok(branches)
    }

    pub fn get_tags(&self, project_path: &Path) -> Result<Vec<TagInfo>, Box<dyn std::error::Error>> {
        let repo = self.get_or_open_repository(project_path)?;
        let mut tags = Vec::new();
        
        repo.tag_foreach(|oid, name| {
            if let Ok(name_str) = std::str::from_utf8(name) {
                // Remove refs/tags/ prefix
                let tag_name = name_str.strip_prefix("refs/tags/").unwrap_or(name_str);
                
                tags.push(TagInfo {
                    name: tag_name.to_string(),
                    commit_id: oid.to_string(),
                    message: None, // TODO: Get tag message if it's an annotated tag
                });
            }
            true
        })?;
        
        Ok(tags)
    }

    pub fn get_commit_diff(&self, project_path: &Path, commit_id: &str) -> Result<CommitDiff, Box<dyn std::error::Error>> {
        let repo = self.get_or_open_repository(project_path)?;
        let oid = Oid::from_str(commit_id)?;
        let commit = repo.find_commit(oid)?;
        
        let tree = commit.tree()?;
        let mut file_diffs = Vec::new();
        
        if let Ok(parent) = commit.parent(0) {
            let parent_tree = parent.tree()?;
            let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;
            
            diff.foreach(
                &mut |delta, _progress| {
                    let old_path = delta.old_file().path().map(|p| p.to_path_buf());
                    let new_path = delta.new_file().path().map(|p| p.to_path_buf());
                    let status = delta.status();
                    
                    let file_diff = FileDiff {
                        old_path,
                        new_path,
                        status: match status {
                            git2::Delta::Added => DiffStatus::Added,
                            git2::Delta::Deleted => DiffStatus::Deleted,
                            git2::Delta::Modified => DiffStatus::Modified,
                            git2::Delta::Renamed => DiffStatus::Renamed,
                            _ => DiffStatus::Modified,
                        },
                        changes: Vec::new(),
                    };
                    file_diffs.push(file_diff);
                    true
                },
                None,
                None,
                None,
            )?;
        }
        
        Ok(CommitDiff {
            commit_id: commit_id.to_string(),
            files: file_diffs,
        })
    }


}

#[derive(Debug, Clone)]
pub struct BlameInfo {
    pub line_number: usize,
    pub commit_id: String,
    pub author: String,
    pub author_email: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub commit_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub commit_id: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommitDiff {
    pub commit_id: String,
    pub files: Vec<FileDiff>,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub status: DiffStatus,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum DiffStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
}