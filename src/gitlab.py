import json
from typing import Dict, List
from dataclasses import asdict
import requests
from libs.types import Dependency, Repo
from libs.z2j import get_repo_zon_metadata_gitlab
from libs.utils import (
    file_exists_on_repo,
    fetch_readme_content,
    process_dependency_url,
    extract_repo_info,
)


def convert_gitlab_response_to_repo(gitlab_response: Dict) -> Repo:
    """
    Convert a GitLab API response (as a dictionary) to a Repo object.
    """
    zig_minimum_version = "unknown"
    dependencies: List[Dependency] = []

    # Check for build.zig and build.zig.zon files
    path_with_namespace = gitlab_response["path_with_namespace"]
    base_url = "https://gitlab.com"
    has_build_zig = file_exists_on_repo(
        base_url, path_with_namespace, "build.zig", "gitlab"
    )
    has_build_zig_zon = file_exists_on_repo(
        base_url, path_with_namespace, "build.zig.zon", "gitlab"
    )

    # Process build.zig.zon if it exists
    if has_build_zig_zon:
        try:
            zon_metadata = get_repo_zon_metadata_gitlab(path_with_namespace)
            zig_minimum_version = zon_metadata.get("zig_version", "unknown")
            dependencies = [
                process_dependency_url(dep_info, extract_repo_info)
                for dep_info in zon_metadata.get("dependencies", [])
            ]
        except Exception as e:
            print(f"Error processing build.zig.zon for {path_with_namespace}: {e}")

    # Build Repo object
    readme_url = gitlab_response.get("readme_url")
    return Repo(
        avatar_url=gitlab_response["namespace"]["avatar_url"],
        name=gitlab_response["path"],
        full_name=path_with_namespace,
        created_at=gitlab_response["created_at"],
        default_branch=gitlab_response["default_branch"],
        dependencies=dependencies,
        description=gitlab_response["description"],
        fork=False,
        forks_count=gitlab_response["forks_count"],
        has_build_zig=has_build_zig,
        has_build_zig_zon=has_build_zig_zon,
        license="-",
        open_issues=0,
        readme_content=(
            fetch_readme_content(
                base_url,
                path_with_namespace,
                ["README.md", "readme.md", "ReadMe.md"],
                "gitlab",
            )
            if readme_url
            else "No readme found"
        ),
        repo_from="gitlab",
        size=0,
        stargazers_count=gitlab_response["star_count"],
        tags_url="",
        topics=gitlab_response.get("topics", []),
        updated_at=gitlab_response["last_activity_at"],
        watchers_count=0,
        zig_minimum_version=zig_minimum_version,
    )


if __name__ == "__main__":
    """Main entry point for fetching and saving GitLab repository data."""
    url = "https://gitlab.com/api/v4/projects?per_page=100&topic=zig"
    response = requests.get(url, timeout=10)

    if response.status_code == 200:
        try:
            # Load existing data from programs.json
            existing_data = []
            try:
                with open("./database/programs.json", "r") as file:
                    existing_data = json.load(file)
            except FileNotFoundError:
                print("programs.json not found, will create new file")

            # Process GitLab data
            repos = [convert_gitlab_response_to_repo(repo) for repo in response.json()]

            # Combine existing data with new data
            combined_data = existing_data + [asdict(repo) for repo in repos]

            # Write back to programs.json
            with open("./database/programs.json", "w") as file:
                json.dump(combined_data, file, indent=4)

        except Exception as e:
            print(f"Error processing GitLab repositories: {e}")
    else:
        print(
            f"Failed to fetch projects from GitLab API. Status code: {response.status_code}"
        )
        exit(1)
