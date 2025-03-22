import os
import requests


API_KEY = os.getenv("GITHUB_API_KEY")
HEADERS = {
    "Authorization": f"Bearer {API_KEY}",
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "Python",
}

def fetch_readme_content(repo_full_name) -> str:
    url = f"https://api.github.com/repos/{repo_full_name}/readme"
    try:
        response = requests.get(url, headers=HEADERS, timeout=5)
        if response.ok and (readme_url := response.json().get("download_url")):
            return requests.get(readme_url, timeout=5).text
    except Exception:
        pass
    return "404"

def has_build_file(owner, repo, filename) -> bool:
    url = f"https://api.github.com/repos/{owner}/{repo}/contents/{filename}"
    response = requests.get(url, headers=HEADERS)
    return response.status_code == 200

def process_repo(repo):
    x, y = repo["full_name"].split("/")
    license = repo["license"]["spdx_id"] if repo["license"] and repo["license"].get("spdx_id") else "None"
    return {
        "avatar_url": repo["owner"]["avatar_url"],
        "name": repo["name"],
        "full_name": repo["full_name"],
        "created_at": repo["created_at"],
        "description": repo["description"],
        "default_branch": repo["default_branch"],
        "open_issues": repo["open_issues"],
        "stargazers_count": repo["stargazers_count"],
        "forks_count": repo["forks"],
        "watchers_count": repo["watchers"],
        "tags_url": repo["tags_url"],
        "license": license,
        "topics": repo["topics"],
        "size": repo["size"],
        "fork": repo["fork"],
        "updated_at": repo["updated_at"],
        "has_build_zig": has_build_file(x, y, "build.zig"),
        "has_build_zig_zon": has_build_file(x, y, "build.zig.zon"),
        "readme_content": fetch_readme_content(repo["full_name"]),
    }