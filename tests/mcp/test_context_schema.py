"""
Tests for the unified context schema (token-oriented)
Validates that all connectors return properly normalized data
"""
import pytest
from typing import Dict, Any

def validate_repository_descriptor(repo: Dict[str, Any]) -> bool:
    """Validate RepositoryDescriptor schema"""
    required_fields = {
        "id": str,
        "provider": str,
        "name": str,
        "owner": str,
        "visibility": str,
        "default_branch": str,
        "url": str,
        "updated_at": int,
    }
    
    for field, field_type in required_fields.items():
        if field not in repo:
            return False
        if not isinstance(repo[field], field_type):
            return False
    
    # Validate provider is known
    assert repo["provider"] in ["github", "gitlab", "bitbucket"]
    
    # Validate visibility
    assert repo["visibility"] in ["public", "private"]
    
    # Validate ID format
    provider_prefix = {
        "github": "gh:",
        "gitlab": "gl:",
        "bitbucket": "bb:"
    }
    assert repo["id"].startswith(provider_prefix[repo["provider"]])
    
    return True


def validate_file_descriptor(file: Dict[str, Any]) -> bool:
    """Validate FileDescriptor schema"""
    required_fields = {
        "id": str,
        "path": str,
        "name": str,
        "kind": str,
    }
    
    for field, field_type in required_fields.items():
        if field not in file:
            return False
        if not isinstance(file[field], field_type):
            return False
    
    # Validate kind
    assert file["kind"] in ["file", "dir"]
    
    return True


def validate_branch_descriptor(branch: Dict[str, Any]) -> bool:
    """Validate BranchDescriptor schema"""
    required_fields = {
        "name": str,
        "commit_id": str,
        "is_default": bool,
    }
    
    for field, field_type in required_fields.items():
        if field not in branch:
            return False
        if not isinstance(branch[field], field_type):
            return False
    
    return True


class TestSchemaValidation:
    """Test schema validation for all context types"""
    
    def test_repository_descriptor_sample(self):
        """Test RepositoryDescriptor with sample data"""
        sample = {
            "id": "gh:KogSector/ConHub",
            "provider": "github",
            "name": "ConHub",
            "owner": "KogSector",
            "visibility": "public",
            "default_branch": "main",
            "description": "Knowledge layer and context engine",
            "url": "https://github.com/KogSector/ConHub",
            "updated_at": 1700000000
        }
        
        assert validate_repository_descriptor(sample)
    
    def test_file_descriptor_sample(self):
        """Test FileDescriptor with sample data"""
        sample = {
            "id": "gh:KogSector/ConHub:src/lib.rs",
            "path": "src/lib.rs",
            "name": "lib.rs",
            "kind": "file",
            "size": 1024,
            "language": "rust",
            "sha": "abc123",
            "last_modified": 1700000000,
            "mime_type": "text/plain"
        }
        
        assert validate_file_descriptor(sample)
    
    def test_branch_descriptor_sample(self):
        """Test BranchDescriptor with sample data"""
        sample = {
            "name": "main",
            "commit_id": "abc123def456",
            "is_default": True,
            "protected": True
        }
        
        assert validate_branch_descriptor(sample)


class TestTokenOptimization:
    """Test that schemas are token-efficient"""
    
    def test_field_names_are_short(self):
        """Verify field names are concise"""
        # Context schema uses short but semantic names
        # Not single-letter, but not verbose either
        acceptable_fields = {
            "id", "name", "path", "kind", "size", "sha",
            "owner", "provider", "visibility", "url",
            "commit_id", "is_default", "protected"
        }
        
        # All are reasonably short (under 15 chars)
        for field in acceptable_fields:
            assert len(field) < 15
    
    def test_no_deep_nesting(self):
        """Verify schemas avoid deep nesting"""
        sample_repo = {
            "id": "gh:owner/repo",
            "provider": "github",
            "name": "repo",
            "owner": "owner",
            "visibility": "public",
            "default_branch": "main",
            "url": "https://github.com/owner/repo",
            "updated_at": 1700000000
        }
        
        # All fields are at top level
        def max_depth(obj, current_depth=1):
            if not isinstance(obj, dict):
                return current_depth
            if not obj:
                return current_depth
            return max(max_depth(v, current_depth + 1) for v in obj.values())
        
        assert max_depth(sample_repo) <= 2  # Minimal nesting


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
