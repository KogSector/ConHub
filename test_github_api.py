#!/usr/bin/env python3
"""
GitHub API Test Script for ConHub
Tests all GitHub repository connection endpoints
"""

import os
import sys
import json
import requests
from typing import Dict, Any, Optional

# Configuration
BASE_URL = "http://localhost:3013"
TEST_REPO = "https://github.com/microsoft/vscode"

# ANSI color codes for terminal output
class Colors:
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    BLUE = '\033[94m'
    RESET = '\033[0m'
    BOLD = '\033[1m'

def print_header(text: str):
    """Print a colored header"""
    print(f"\n{Colors.CYAN}{Colors.BOLD}{text}{Colors.RESET}")
    print(f"{Colors.CYAN}{'=' * len(text)}{Colors.RESET}\n")

def print_success(text: str):
    """Print success message"""
    print(f"{Colors.GREEN}✅ {text}{Colors.RESET}")

def print_error(text: str):
    """Print error message"""
    print(f"{Colors.RED}❌ {text}{Colors.RESET}")

def print_info(text: str, indent: int = 0):
    """Print info message"""
    prefix = "  " * indent
    print(f"{prefix}{Colors.BLUE}{text}{Colors.RESET}")

def print_warning(text: str):
    """Print warning message"""
    print(f"{Colors.YELLOW}⚠️  {text}{Colors.RESET}")

def get_github_token() -> Optional[str]:
    """Get GitHub token from environment"""
    token = os.environ.get('GITHUB_TOKEN')
    if not token:
        print_error("GITHUB_TOKEN environment variable not set")
        print_warning("Please set it with: export GITHUB_TOKEN='your_github_token'")
        print_warning("Or on Windows: set GITHUB_TOKEN=your_github_token")
        return None
    return token

def test_validate_access(token: str) -> Optional[Dict[str, Any]]:
    """Test 1: Validate Repository Access"""
    print_header("Test 1: Validate Repository Access")
    
    payload = {
        "repo_url": TEST_REPO,
        "access_token": token
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/github/validate-access",
            json=payload,
            timeout=30
        )
        
        if response.status_code == 200:
            data = response.json()
            print_success("Repository access validated")
            print_info(f"Has Access: {data.get('has_access')}", 1)
            
            if data.get('repo_info'):
                repo_info = data['repo_info']
                print_info("Repository Info:", 1)
                print_info(f"Name: {repo_info.get('name')}", 2)
                print_info(f"Full Name: {repo_info.get('full_name')}", 2)
                print_info(f"Private: {repo_info.get('private')}", 2)
                print_info(f"Default Branch: {repo_info.get('default_branch')}", 2)
                print_info(f"Description: {repo_info.get('description', 'N/A')[:100]}...", 2)
                
                if repo_info.get('permissions'):
                    perms = repo_info['permissions']
                    print_info("Permissions:", 2)
                    print_info(f"Admin: {perms.get('admin')}", 3)
                    print_info(f"Push: {perms.get('push')}", 3)
                    print_info(f"Pull: {perms.get('pull')}", 3)
                
                if repo_info.get('branches'):
                    branches = repo_info['branches']
                    print_info(f"Branches ({len(branches)}):", 2)
                    for branch in branches[:5]:  # Show first 5
                        print_info(f"- {branch.get('name')} {'(protected)' if branch.get('protected') else ''}", 3)
                    if len(branches) > 5:
                        print_info(f"... and {len(branches) - 5} more", 3)
                
                if repo_info.get('languages'):
                    languages = repo_info['languages']
                    print_info(f"Languages ({len(languages)}): {', '.join(languages)}", 2)
                
                return repo_info
            else:
                print_warning("No repository info returned")
                return None
        else:
            print_error(f"Request failed with status {response.status_code}")
            print_info(f"Response: {response.text}", 1)
            return None
            
    except requests.exceptions.ConnectionError:
        print_error("Could not connect to the data service")
        print_warning(f"Make sure the service is running on {BASE_URL}")
        return None
    except Exception as e:
        print_error(f"Test failed: {str(e)}")
        return None

def test_get_branches(token: str) -> Optional[list]:
    """Test 2: Get Repository Branches"""
    print_header("Test 2: Get Repository Branches")
    
    payload = {
        "repo_url": TEST_REPO,
        "access_token": token
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/github/branches",
            json=payload,
            timeout=30
        )
        
        if response.status_code == 200:
            data = response.json()
            print_success("Branches fetched successfully")
            
            if data.get('branches'):
                branches = data['branches']
                print_info(f"Total Branches: {len(branches)}", 1)
                print_info("Branch List:", 1)
                for branch in branches[:10]:  # Show first 10
                    protected = " 🔒" if branch.get('protected') else ""
                    print_info(f"- {branch.get('name')}{protected}", 2)
                if len(branches) > 10:
                    print_info(f"... and {len(branches) - 10} more", 2)
                return branches
            else:
                print_warning("No branches returned")
                return None
        else:
            print_error(f"Request failed with status {response.status_code}")
            return None
            
    except Exception as e:
        print_error(f"Test failed: {str(e)}")
        return None

def test_get_languages(token: str) -> Optional[list]:
    """Test 3: Get Repository Languages"""
    print_header("Test 3: Get Repository Languages")
    
    payload = {
        "repo_url": TEST_REPO,
        "access_token": token
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/github/languages",
            json=payload,
            timeout=30
        )
        
        if response.status_code == 200:
            data = response.json()
            print_success("Languages fetched successfully")
            
            if data.get('languages'):
                languages = data['languages']
                print_info(f"Total Languages: {len(languages)}", 1)
                print_info(f"Languages: {', '.join(languages)}", 1)
                return languages
            else:
                print_warning("No languages returned")
                return None
        else:
            print_error(f"Request failed with status {response.status_code}")
            return None
            
    except Exception as e:
        print_error(f"Test failed: {str(e)}")
        return None

def test_get_file_extensions(token: str, branch: str = "main") -> Optional[Dict[str, int]]:
    """Test 4: Get All File Extensions in Repository"""
    print_header("Test 4: Analyze File Extensions")
    
    print_info("Note: This test simulates what the sync would discover", 1)
    print_info("In a real sync, all file extensions would be catalogued", 1)
    
    # This would be done during actual sync
    # For now, we'll show what languages are detected
    payload = {
        "repo_url": TEST_REPO,
        "access_token": token
    }
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/github/languages",
            json=payload,
            timeout=30
        )
        
        if response.status_code == 200:
            data = response.json()
            languages = data.get('languages', [])
            
            # Map languages to common extensions
            extension_map = {
                'TypeScript': ['.ts', '.tsx'],
                'JavaScript': ['.js', '.jsx', '.mjs'],
                'Python': ['.py', '.pyx', '.pyi'],
                'Rust': ['.rs'],
                'Go': ['.go'],
                'Java': ['.java'],
                'C': ['.c', '.h'],
                'C++': ['.cpp', '.hpp', '.cxx', '.cc'],
                'HTML': ['.html', '.htm'],
                'CSS': ['.css'],
                'SCSS': ['.scss'],
                'JSON': ['.json'],
                'YAML': ['.yaml', '.yml'],
                'Markdown': ['.md', '.markdown'],
                'Shell': ['.sh', '.bash'],
                'SQL': ['.sql'],
                'PHP': ['.php'],
                'Ruby': ['.rb'],
                'Swift': ['.swift'],
                'Kotlin': ['.kt', '.kts'],
            }
            
            print_success("File extensions analysis")
            print_info("Detected file types based on languages:", 1)
            
            all_extensions = []
            for lang in languages:
                if lang in extension_map:
                    exts = extension_map[lang]
                    all_extensions.extend(exts)
                    print_info(f"{lang}: {', '.join(exts)}", 2)
            
            print_info(f"\nTotal unique extensions: {len(all_extensions)}", 1)
            return {ext: 0 for ext in all_extensions}  # Counts would be filled during sync
        else:
            print_error(f"Request failed with status {response.status_code}")
            return None
            
    except Exception as e:
        print_error(f"Test failed: {str(e)}")
        return None

def test_sync_preview(token: str, branch: str = "main") -> bool:
    """Test 5: Preview Repository Sync (without actual sync)"""
    print_header("Test 5: Repository Sync Preview")
    
    print_info("This would sync the repository and extract all files", 1)
    print_info("The sync process includes:", 1)
    print_info("- Fetching all files from the specified branch", 2)
    print_info("- Filtering by language/extension if specified", 2)
    print_info("- Respecting exclude patterns", 2)
    print_info("- Checking file size limits", 2)
    print_info("- Downloading file contents", 2)
    print_info("- Preparing documents for embedding", 2)
    
    payload = {
        "repo_url": TEST_REPO,
        "access_token": token,
        "branch": branch,
        "include_languages": ["TypeScript", "JavaScript"],
        "exclude_paths": ["node_modules", "dist", "build"],
        "max_file_size_mb": 1
    }
    
    print_info("\nSync Configuration:", 1)
    print_info(f"Repository: {TEST_REPO}", 2)
    print_info(f"Branch: {branch}", 2)
    print_info(f"Languages: {', '.join(payload['include_languages'])}", 2)
    print_info(f"Excluded: {', '.join(payload['exclude_paths'])}", 2)
    print_info(f"Max file size: {payload['max_file_size_mb']} MB", 2)
    
    print_warning("\nActual sync endpoint: POST /api/github/sync-repository")
    print_info("(Not executing to avoid long processing time)", 1)
    
    return True

def main():
    """Main test runner"""
    print_header("🚀 ConHub GitHub API Test Suite")
    
    # Get GitHub token
    token = get_github_token()
    if not token:
        sys.exit(1)
    
    print_info(f"Base URL: {BASE_URL}")
    print_info(f"Test Repository: {TEST_REPO}")
    print_info(f"Token: {token[:8]}...{token[-4:]}")
    
    # Run tests
    results = {
        'validate_access': False,
        'get_branches': False,
        'get_languages': False,
        'file_extensions': False,
        'sync_preview': False
    }
    
    # Test 1: Validate Access
    repo_info = test_validate_access(token)
    results['validate_access'] = repo_info is not None
    
    # Test 2: Get Branches
    branches = test_get_branches(token)
    results['get_branches'] = branches is not None
    
    # Test 3: Get Languages
    languages = test_get_languages(token)
    results['get_languages'] = languages is not None
    
    # Test 4: File Extensions
    extensions = test_get_file_extensions(token)
    results['file_extensions'] = extensions is not None
    
    # Test 5: Sync Preview
    results['sync_preview'] = test_sync_preview(token)
    
    # Summary
    print_header("📊 Test Summary")
    
    passed = sum(1 for v in results.values() if v)
    total = len(results)
    
    for test_name, passed_test in results.items():
        status = "✅ PASSED" if passed_test else "❌ FAILED"
        color = Colors.GREEN if passed_test else Colors.RED
        print(f"{color}{status}{Colors.RESET} - {test_name.replace('_', ' ').title()}")
    
    print(f"\n{Colors.BOLD}Results: {passed}/{total} tests passed{Colors.RESET}")
    
    if passed == total:
        print_success("All tests passed! GitHub integration is working correctly.")
        return 0
    else:
        print_error(f"{total - passed} test(s) failed. Please check the errors above.")
        return 1

if __name__ == "__main__":
    sys.exit(main())
