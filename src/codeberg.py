import json
from typing import Dict, List
from dataclasses import asdict
import requests

from libs.types import Dependency, Repo
from libs.z2j import get_repo_zon_metadata
from libs.utils import (
    file_exists_on_repo,
    process_dependency_url,
    extract_repo_info,
)


def convert_codeberg_response_to_repo(codeberg_response: Dict) -> Repo:
    """
    Convert a Codeberg API response (as a dictionary) to a Repo object.
    """
    zig_minimum_version = "unknown"
    dependencies: List[Dependency] = []

    # Check for build.zig and build.zig.zon files
    full_name = codeberg_response["full_name"]
    base_url = "https://codeberg.org"
    has_build_zig = file_exists_on_repo(base_url, full_name, "build.zig", "codeberg")
    has_build_zig_zon = file_exists_on_repo(
        base_url, full_name, "build.zig.zon", "codeberg"
    )

    if has_build_zig_zon:
        try:
            zon_metadata = get_repo_zon_metadata(full_name, platform="codeberg")
            zig_minimum_version = zon_metadata.get("zig_version", "unknown")
            dependencies = [
                process_dependency_url(dep_info, extract_repo_info)
                for dep_info in zon_metadata.get("dependencies", [])
            ]
        except Exception as e:
            print(f"Error processing build.zig.zon for {full_name}: {e}")

    return Repo(
        avatar_url=codeberg_response["owner"]["avatar_url"],
        name=codeberg_response["name"],
        full_name=full_name,
        created_at=codeberg_response["created_at"],
        default_branch=codeberg_response["default_branch"],
        dependencies=dependencies,
        description=codeberg_response["description"],
        fork=codeberg_response["fork"],
        forks_count=codeberg_response["forks_count"],
        has_build_zig=has_build_zig,
        has_build_zig_zon=has_build_zig_zon,
        license="-",
        open_issues=codeberg_response["open_issues_count"],
        readme_content="404",
        repo_from="codeberg",
        size=codeberg_response["size"],
        stargazers_count=codeberg_response["stars_count"],
        tags_url="",
        topics=codeberg_response.get("topics", []),
        updated_at=codeberg_response["updated_at"],
        watchers_count=codeberg_response["watchers_count"],
        zig_minimum_version=zig_minimum_version,
    )


if __name__ == "__main__":
    """Main entry point for fetching and saving Codeberg repository data."""
    url = "https://codeberg.org/api/v1/repos/search?q=zig&topic=true"
    response = requests.get(url, timeout=10)
    repos: List[Repo] = []

    if response.status_code == 200:
        try:
            # Load existing data from programs.json
            with open("./database/programs.json", "r") as file:
                existing_data = json.load(file)

            # Process new Codeberg data
            codeberg_data: List[Dict] = response.json()["data"]
            for repo_data in codeberg_data:
                repos.append(convert_codeberg_response_to_repo(repo_data))

            # Combine existing data with new data
            combined_data = existing_data + [asdict(repo) for repo in repos]

            # Write back to programs.json
            with open("./database/programs.json", "w") as file:
                json.dump(combined_data, file, indent=4)

        except Exception as e:
            print(f"Error processing Codeberg repositories: {e}")
            exit(1)
    else:
        print(
            f"Failed to fetch repositories from Codeberg API. Status code: {response.status_code}"
        )
        exit(1)
