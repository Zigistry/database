import os
import concurrent
import requests


API_KEY = os.getenv("GH_API_KEY")
HEADERS = {
    "Authorization": f"Bearer {API_KEY}",
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "Python",
}


def fetch_readme_content(repo_full_name) -> str:
    base_url = f"https://raw.githubusercontent.com/{repo_full_name}/HEAD/"
    POSSIBLE_FILENAMES = (
        "README.md",
        "README.txt",
        "README",
        "readme.md",
        "readme.txt",
        "README.markdown",
        "readme.markdown",
    )

    def fetch(url):
        try:
            response = requests.get(url, timeout=10)
            if response.status_code == 200:
                return response.text
        except requests.exceptions.RequestException:
            pass
        return None

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = {
            executor.submit(fetch, base_url + filename): filename
            for filename in POSSIBLE_FILENAMES
        }
        for future in concurrent.futures.as_completed(futures):
            result = future.result()
            if result:
                return result

    return "404"

def has_build_file(owner, repo, filename) -> bool:
    url = f"https://raw.githubusercontent.com/{owner}/{repo}/HEAD/{filename}"
    response = requests.get(url, timeout=10)
    return response.status_code == 200



def process_repo(repo):
    x, y = repo["full_name"].split("/")
    license = (
        repo["license"]["spdx_id"]
        if repo["license"] and repo["license"].get("spdx_id")
        else "None"
    )
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
