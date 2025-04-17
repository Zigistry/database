import concurrent.futures
import requests
from libs import constants as const
from libs.types import Repo

def fileExistsOnGitHubRepo(full_name: str, filename: str) -> bool:
    url = f"https://raw.githubusercontent.com/{full_name}/HEAD/{filename}"
    response = requests.get(url, timeout=10)
    return response.status_code == 200


def fetch_readme_content(repo_full_name:str) -> str:
    base_url = f"https://raw.githubusercontent.com/{repo_full_name}/HEAD/"

    def fetch(url):
        try:
            response = requests.get(
                url, headers=const.GITHUB_FETCH_HEADERS, timeout=10
            )
            if response.status_code == 200:
                return response.text
        except requests.exceptions.RequestException:
            pass
        return None

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = {
            executor.submit(fetch, base_url + filename): filename
            for filename in const.POSSIBLE_README_FILENAMES
        }
        for future in concurrent.futures.as_completed(futures):
            result = future.result()
            if result:
                return result

    return "404"


def convertGithubRepoFormToZigistryRepoForm(g) -> Repo:
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
        has_build_zig=fileExistsOnGitHubRepo(g["full_name"], "build.zig"),
        has_build_zig_zon=fileExistsOnGitHubRepo(g["full_name"], "build.zig.zon"),
        readme_content=fetch_readme_content(g["full_name"]),
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
