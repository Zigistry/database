import concurrent.futures
from dataclasses import dataclass
import json
import requests
from libs import constants as const
from libs.types import Repo
from libs.z2j import get_repo_zon_metadata
from typing import List, Optional, Dict, Any
import re
from dataclasses import dataclass, field

def fileExistsOnGitHubRepo(full_name: str, filename: str) -> bool:
    url = f"https://raw.githubusercontent.com/{full_name}/HEAD/{filename}"
    response = requests.get(url, timeout=10)
    return response.status_code == 200


# def fetch_readme_content(repo_full_name:str) -> str:
#     base_url = f"https://raw.githubusercontent.com/{repo_full_name}/HEAD/"

#     def fetch(url):
#         try:
#             response = requests.get(
#                 url, headers=const.GITHUB_FETCH_HEADERS, timeout=10
#             )
#             if response.status_code == 200:
#                 return response.text
#         except requests.exceptions.RequestException:
#             pass
#         return None

#     with concurrent.futures.ThreadPoolExecutor() as executor:
#         futures = {
#             executor.submit(fetch, base_url + filename): filename
#             for filename in const.POSSIBLE_README_FILENAMES
#         }
#         for future in concurrent.futures.as_completed(futures):
#             result = future.result()
#             if result:
#                 return result

#     return "404"

# def extract_repo_info(dependency: Dict[str, str]) -> Optional[Dict[str, str]]:
#     """
#     Extracts repository information from a dependency location URL.
#     Handles both git URLs and archive URLs.
    
#     Args:
#         dependency: Dictionary containing dependency info with 'location' field
        
#     Returns:
#         Dictionary with 'owner', 'repo', and 'ref' if successful, None otherwise
#     """
#     location = dependency.get('location', '')
#     if not location:
#         return None

#     # Pattern for git+https URLs (e.g., git+https://github.com/owner/repo#ref)
#     git_pattern = r'git\+https://github\.com/([^/]+)/([^#?]+)(?:#([^#\s]+))?'
    
#     # Pattern for archive URLs (e.g., https://github.com/owner/repo/archive/ref.tar.gz)
#     archive_pattern = r'https://github\.com/([^/]+)/([^/]+)/archive/([^/]+)(?:\.tar\.gz)?'
    
#     # Try git URL pattern first
#     git_match = re.match(git_pattern, location)
#     if git_match:
#         owner, repo, ref = git_match.groups()
#         return {
#             'owner': owner,
#             'repo': repo.rstrip('.git'),
#             'ref': ref or 'master'  # Default to master if no ref specified
#         }
    
#     # Try archive URL pattern
#     archive_match = re.match(archive_pattern, location)
#     if archive_match:
#         owner, repo, ref = archive_match.groups()
#         return {
#             'owner': owner,
#             'repo': repo,
#             'ref': ref
#         }
    
#     return None


@dataclass
class Dependency:
    name: str
    url: str
    commit: Optional[str] = None
    tar_url: Optional[str] = None
    type: str = "unknown"


def convertGithubRepoFormToZigistryRepoForm(g: Dict[str, Any]) -> Repo:
    """
    Convert GitHub repository data to Zigistry repository format.
    
    Args:
        g: GitHub repository data dictionary
    
    Returns:
        Repo: Converted repository data
    """
    has_build_zig = fileExistsOnGitHubRepo(g["full_name"], "build.zig")
    has_build_zig_zon = fileExistsOnGitHubRepo(g["full_name"], "build.zig.zon")
    
    # Initialize default values
    zig_minimum_version = "unknown"
    dependencies: List[Dependency] = []
    
    # Process build.zig.zon if it exists
    if has_build_zig_zon:
        try:
            zon_metadata = get_repo_zon_metadata(g["full_name"])
            
            # Extract zig version
            zig_minimum_version = zon_metadata.get("zig_version", "unknown")
            
            # Process dependencies
            for dep_info in zon_metadata.get("dependencies", []):
                dependency = process_dependency_url(dep_info, extract_repo_info_func=extract_repo_info)
                dependencies.append(dependency)
                
        except Exception as e:
            print(f"Error processing build.zig.zon for {g['full_name']}: {str(e)}")
    
    return Repo(
        avatar_url=g["owner"]["avatar_url"],
        name=g["name"],
        full_name=g["full_name"],
        created_at=g["created_at"],
        description=g["description"],
        default_branch=g["default_branch"],
        open_issues=g["open_issues"],
        stargazers_count=g["stargazers_count"],
        forks_count=g["forks_count"],
        watchers_count=g["watchers_count"],
        tags_url=g["tags_url"],
        license=getattr(g["license"], "spdx_id", "-") if g["license"] else "-",
        topics=g["topics"],
        size=g["size"],
        fork=g["fork"],
        updated_at=g["updated_at"],
        has_build_zig=has_build_zig,
        has_build_zig_zon=has_build_zig_zon,
        zig_minimum_version=zig_minimum_version,
        repo_from="github",
        dependencies=dependencies,
        readme_content=fetch_readme_content("https://github.com", g["full_name"], const.POSSIBLE_README_FILENAMES, "github"),
    )



def remove_duplicates_from_json_list(repos: list[dict]) -> list[dict]:
    seen = set()
    unique_repos = []
    for repo in repos:
        full_name = repo.get('full_name')
        if full_name not in seen:
            seen.add(full_name)
            unique_repos.append(repo)
    return unique_repos




import concurrent.futures
import requests
import re
from typing import Dict, List, Optional
from libs.types import Dependency


def file_exists_on_repo(base_url: str, full_name: str, filename: str, platform: str) -> bool:
    """Check if a specific file exists in a repository."""
    if platform == "gitlab":
        url = f"{base_url}/{full_name}/-/raw/main/{filename}"  # GitLab-specific structure
    elif platform == "codeberg":
        url = f"{base_url}/{full_name}/raw/branch/master/{filename}"  # Codeberg-specific structure
    elif platform == "github":
        url = f"https://raw.githubusercontent.com/{full_name}/master/{filename}"  # GitHub-specific structure
    else:
        raise ValueError(f"Unsupported platform: {platform}")

    try:
        response = requests.get(url, timeout=10)
        return response.status_code == 200
    except requests.exceptions.RequestException:
        return False

def fetch_readme_content(base_url: str, repo_full_name: str, possible_filenames: List[str], platform: str) -> str:
    """
    Attempts to fetch the README content from a repository.

    :param base_url: The base URL of the platform (e.g., https://gitlab.com, https://codeberg.org, https://github.com)
    :param repo_full_name: The full name of the repository (e.g., owner/repo)
    :param possible_filenames: A list of possible README filenames (e.g., ["README.md", "readme.md"])
    :param platform: The platform name (e.g., "gitlab", "codeberg", "github") to adjust URL structure.
    :return: The content of the README file or "404" if not found.
    """
    def fetch(url):
        try:
            response = requests.get(url, timeout=10)
            if response.status_code == 200:
                return response.text
        except requests.exceptions.RequestException:
            pass
        return None

    # Adjust URL structure based on the platform
    if platform == "gitlab":
        url_template = lambda base, repo, filename: f"{base}/{repo}/-/raw/main/{filename}"  # GitLab-specific structure
    elif platform == "codeberg":
        url_template = lambda base, repo, filename: f"{base}/{repo}/raw/branch/master/{filename}"  # Codeberg-specific structure
    elif platform == "github":
        url_template = lambda base, repo, filename: f"https://raw.githubusercontent.com/{repo}/master/{filename}"  # GitHub-specific structure
    else:
        raise ValueError(f"Unsupported platform: {platform}")

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = {
            executor.submit(fetch, url_template(base_url, repo_full_name, filename)): filename
            for filename in possible_filenames
        }
        for future in concurrent.futures.as_completed(futures):
            result = future.result()
            if result:
                return result

    return "404"

import re
from typing import Dict, Optional
from libs.types import Dependency


def process_dependency_url(dep_info: Dict[str, str], extract_repo_info_func) -> Dependency:
    """Process dependency URL to extract useful information."""
    name = dep_info.get("name", "")
    location = dep_info.get("location", "")
    source = dep_info.get("source", "unknown")

    if not location:
        return Dependency(name=name, url="", type=source)

    # Extract repository information
    repo_info = extract_repo_info_func(location)
    if not repo_info:
        # If repo info extraction fails, return with the original location
        return Dependency(name=name, url=location, type=source)

    platform, owner, repo, commit = (
        repo_info["platform"],
        repo_info["owner"],
        repo_info["repo"],
        repo_info["ref"],
    )

    # Construct repo and tarball URLs dynamically
    repo_url = f"https://{platform}/{owner}/{repo}"
    tar_url_templates = {
        "gitlab.com": f"{repo_url}/-/archive/{commit}/{repo}.tar.gz",
        "github.com": f"{repo_url}/archive/{commit}.tar.gz",
        "codeberg.org": f"{repo_url}/archive/{commit}.tar.gz",
    }
    tar_url = tar_url_templates.get(platform, "")  # Use an empty string if the platform is unsupported

    return Dependency(
        name=name,
        url=repo_url,
        commit=commit,
        tar_url=tar_url,
        type=source,
    )


import re
from typing import Dict, Optional


import re
from typing import Dict, Optional


def extract_repo_info(location: str) -> Optional[Dict[str, str]]:
    """
    Extracts repository information from a dependency location URL for GitHub, GitLab, and Codeberg.
    Handles both git URLs, archive URLs, and other flexible formats.

    Args:
        location: The dependency URL as a string.

    Returns:
        Dictionary with 'platform', 'owner', 'repo', and 'ref' if successful, None otherwise.
    """
    if not location:
        return None

    # Patterns for Git URLs and archive URLs (GitHub, GitLab, Codeberg)
    patterns = [
        # git+https URLs (GitHub, GitLab, Codeberg)
        r'git\+https://(github\.com|gitlab\.com|codeberg\.org)/([^/]+)/([^#?]+)(?:#([^#\s]+))?',
        # Archive URLs (GitHub, GitLab, Codeberg)
        r'https://(github\.com|gitlab\.com|codeberg\.org)/([^/]+)/([^/]+)/archive/([^/]+)(?:\.tar\.gz)?',
        # Flexible GitHub-like URLs
        r'https://(github\.com|gitlab\.com|codeberg\.org)/([^/]+)/([^#?]+)(?:#([^#\s]+))?',
    ]

    # Try matching each pattern
    for pattern in patterns:
        match = re.match(pattern, location)
        if match:
            platform, owner, repo, ref = match.groups()
            return {
                "platform": platform,
                "owner": owner,
                "repo": repo.removesuffix(".git"),
                "ref": ref or "master",  # Default to master if no ref specified
            }

    # No match found
    return None
