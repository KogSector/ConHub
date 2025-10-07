use crate::models::{Repository, RepositoryType};
use uuid::Uuid;
use std::error::Error;
use std::collections::HashMap;

/// Repository service for managing data repositories
pub struct RepositoryService {
    repositories: HashMap<Uuid, Repository>,
}

impl RepositoryService {
    pub fn new() -> Self {
        Self {
            repositories: HashMap::new(),
        }
    }

    pub fn get_repository(&self, id: &Uuid) -> Option<&Repository> {
        self.repositories.get(id)
    }

    pub fn create_repository(&mut self, name: String, url: String, repo_type: RepositoryType) -> Result<Uuid, Box<dyn Error>> {
        let id = Uuid::new_v4();
        let repository = Repository {
            id,
            name,
            url,
            repo_type,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        self.repositories.insert(id, repository);
        Ok(id)
    }

    pub fn update_repository(&mut self, id: &Uuid, name: Option<String>, url: Option<String>) -> Result<(), Box<dyn Error>> {
        if let Some(repository) = self.repositories.get_mut(id) {
            if let Some(name) = name {
                repository.name = name;
            }
            
            if let Some(url) = url {
                repository.url = url;
            }
            
            repository.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err("Repository not found".into())
        }
    }

    pub fn delete_repository(&mut self, id: &Uuid) -> Result<(), Box<dyn Error>> {
        if self.repositories.remove(id).is_some() {
            Ok(())
        } else {
            Err("Repository not found".into())
        }
    }

    pub fn list_repositories(&self) -> Vec<&Repository> {
        self.repositories.values().collect()
    }
}

#[derive(Debug, Clone)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub repo_type: RepositoryType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryType {
    Git,
    Svn,
    Mercurial,
    Local,
}