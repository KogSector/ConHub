use regex::Regex;
use url::Url;
use crate::models::{VcsType, VcsProvider};

pub struct VcsDetector;

impl VcsDetector {
    
    pub fn detect_from_url(url: &str) -> Result<(VcsType, VcsProvider), String> {
        let parsed_url = Url::parse(url)
            .map_err(|_| format!("Invalid URL: {}", url))?;
        
        let host = parsed_url.host_str()
            .ok_or_else(|| "No host found in URL")?;
        
        
        let provider = Self::detect_provider(host, url)?;
        
        
        let vcs_type = Self::detect_vcs_type(&provider, url)?;
        
        Ok((vcs_type, provider))
    }
    
    
    fn detect_provider(host: &str, url: &str) -> Result<VcsProvider, String> {
        match host.to_lowercase().as_str() {
            h if h.contains("github.com") => Ok(VcsProvider::GitHub),
            h if h.contains("gitlab.com") || h.contains("gitlab.") => Ok(VcsProvider::GitLab),
            h if h.contains("bitbucket.org") || h.contains("bitbucket.") => Ok(VcsProvider::Bitbucket),
            h if h.contains("dev.azure.com") || h.contains("visualstudio.com") => Ok(VcsProvider::Azure),
            h if h.contains("gitea.") => Ok(VcsProvider::Gitea),
            h if h.contains("sourceforge.net") => Ok(VcsProvider::SourceForge),
            h if h.contains("amazonaws.com") && url.contains("codecommit") => Ok(VcsProvider::CodeCommit),
            "localhost" | "127.0.0.1" => Ok(VcsProvider::Local),
            _ => {
                
                if Self::is_self_hosted_git(url) {
                    Ok(VcsProvider::SelfHosted)
                } else {
                    Ok(VcsProvider::SelfHosted) 
                }
            }
        }
    }
    
    
    fn detect_vcs_type(provider: &VcsProvider, url: &str) -> Result<VcsType, String> {
        match provider {
            VcsProvider::GitHub | VcsProvider::GitLab | VcsProvider::Bitbucket | 
            VcsProvider::Azure | VcsProvider::Gitea | VcsProvider::CodeCommit => {
                Ok(VcsType::Git)
            }
            VcsProvider::SourceForge => {
                
                Self::detect_sourceforge_vcs(url)
            }
            VcsProvider::SelfHosted | VcsProvider::Local => {
                Self::detect_self_hosted_vcs(url)
            }
        }
    }
    
    
    fn detect_sourceforge_vcs(url: &str) -> Result<VcsType, String> {
        if url.contains("/git/") || url.ends_with(".git") {
            Ok(VcsType::Git)
        } else if url.contains("/svn/") {
            Ok(VcsType::Subversion)
        } else if url.contains("/hg/") {
            Ok(VcsType::Mercurial)
        } else if url.contains("/bzr/") {
            Ok(VcsType::Bazaar)
        } else {
            Ok(VcsType::Git) 
        }
    }
    
    
    fn detect_self_hosted_vcs(url: &str) -> Result<VcsType, String> {
        let url_lower = url.to_lowercase();
        
        
        if url_lower.ends_with(".git") || 
           url_lower.contains("git") ||
           url_lower.contains("cgit") ||
           url_lower.contains("gitweb") {
            return Ok(VcsType::Git);
        }
        
        
        if url_lower.contains("svn") || 
           url_lower.contains("subversion") ||
           url_lower.contains("/trunk/") ||
           url_lower.contains("/branches/") ||
           url_lower.contains("/tags/") {
            return Ok(VcsType::Subversion);
        }
        
        
        if url_lower.contains("hg") || 
           url_lower.contains("mercurial") {
            return Ok(VcsType::Mercurial);
        }
        
        
        if url_lower.contains("bzr") || 
           url_lower.contains("bazaar") {
            return Ok(VcsType::Bazaar);
        }
        
        
        if url_lower.contains("perforce") || 
           url_lower.contains("p4") {
            return Ok(VcsType::Perforce);
        }
        
        
        Ok(VcsType::Git)
    }
    
    
    fn is_self_hosted_git(url: &str) -> bool {
        let patterns = [
            r"\.git$",
            r"/git/",
            r"cgit",
            r"gitweb",
            r"gitiles",
            r"gitea",
            r"gitlab",
        ];
        
        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(url) {
                    return true;
                }
            }
        }
        false
    }
    
    
    pub fn extract_repo_info(url: &str) -> Result<(String, String), String> {
        let parsed_url = Url::parse(url)
            .map_err(|_| format!("Invalid URL: {}", url))?;
        
        let path = parsed_url.path().trim_start_matches('/').trim_end_matches('/');
        
        
        let path = if path.ends_with(".git") {
            &path[..path.len() - 4]
        } else {
            path
        };
        
        let parts: Vec<&str> = path.split('/').collect();
        
        if parts.len() >= 2 {
            let owner = parts[0].to_string();
            let repo = parts[1].to_string();
            Ok((owner, repo))
        } else {
            Err("Unable to extract owner and repository name from URL".to_string())
        }
    }
    
    
    pub fn generate_clone_urls(original_url: &str, provider: &VcsProvider) -> Result<CloneUrls, String> {
        let parsed_url = Url::parse(original_url)
            .map_err(|_| format!("Invalid URL: {}", original_url))?;
        
        let host = parsed_url.host_str()
            .ok_or_else(|| "No host found in URL")?;
        
        let (owner, repo) = Self::extract_repo_info(original_url)?;
        
        let https_url = match provider {
            VcsProvider::GitHub => format!("https://github.com/{}/{}.git", owner, repo),
            VcsProvider::GitLab => format!("https://gitlab.com/{}/{}.git", owner, repo),
            VcsProvider::Bitbucket => format!("https://bitbucket.org/{}/{}.git", owner, repo),
            VcsProvider::Azure => {
                
                if host.contains("dev.azure.com") {
                    format!("https://dev.azure.com/{}/{}/_git/{}", owner, repo, repo)
                } else {
                    original_url.to_string()
                }
            }
            _ => original_url.to_string(),
        };
        
        let ssh_url = match provider {
            VcsProvider::GitHub => Some(format!("git@github.com:{}/{}.git", owner, repo)),
            VcsProvider::GitLab => Some(format!("git@gitlab.com:{}/{}.git", owner, repo)),
            VcsProvider::Bitbucket => Some(format!("git@bitbucket.org:{}/{}.git", owner, repo)),
            _ => None,
        };
        
        Ok(CloneUrls {
            https: https_url,
            ssh: ssh_url,
            original: original_url.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CloneUrls {
    pub https: String,
    pub ssh: Option<String>,
    #[allow(dead_code)]
    pub original: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_github_detection() {
        let url = "https://github.com/octocat/Hello-World";
        let (vcs_type, provider) = VcsDetector::detect_from_url(url).unwrap();
        assert_eq!(vcs_type, VcsType::Git);
        assert_eq!(provider, VcsProvider::GitHub);
    }
    
    #[test]
    fn test_gitlab_detection() {
        let url = "https://gitlab.com/gitlab-org/gitlab";
        let (vcs_type, provider) = VcsDetector::detect_from_url(url).unwrap();
        assert_eq!(vcs_type, VcsType::Git);
        assert_eq!(provider, VcsProvider::GitLab);
    }
    
    #[test]
    fn test_bitbucket_detection() {
        let url = "https://bitbucket.org/atlassian/stash";
        let (vcs_type, provider) = VcsDetector::detect_from_url(url).unwrap();
        assert_eq!(vcs_type, VcsType::Git);
        assert_eq!(provider, VcsProvider::Bitbucket);
    }
    
    #[test]
    fn test_repo_info_extraction() {
        let url = "https://github.com/octocat/Hello-World.git";
        let (owner, repo) = VcsDetector::extract_repo_info(url).unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }
    
    #[test]
    fn test_svn_detection() {
        let url = "https://svn.apache.org/repos/asf/httpd/httpd/trunk/";
        let (vcs_type, _provider) = VcsDetector::detect_from_url(url).unwrap();
        assert_eq!(vcs_type, VcsType::Subversion);
    }
}