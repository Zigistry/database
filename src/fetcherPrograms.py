# --------- Imports --------- #
from libs import constants as const
from libs.types import Repo
from libs.utils import convertGithubRepoFormToZigistryRepoForm
import requests
from dataclasses import asdict
import json
from concurrent.futures import ThreadPoolExecutor, as_completed
import time
import os
# --------------------------- #

# --- NEW: Path to additions file ---
GITHUB_ADDITIONS_PATH = "./database/github_additions.json"

def fetch_convert_with_delay(url: str, delay: float) -> list[Repo]:
    time.sleep(delay)
    try:
        print(f"Fetching: {url}")
        response = requests.get(url, headers=const.GITHUB_FETCH_HEADERS)
        response.raise_for_status()
        items = response.json()["items"]
        return [convertGithubRepoFormToZigistryRepoForm(item) for item in items]
    except Exception as e:
        print(f"Error fetching {url}: {e}")
        return []

# --- NEW: fetch repo by full name (owner/repo) ---
def fetch_repo_by_full_name(owner_repo: str) -> Repo | None:
    url = f"https://api.github.com/repos/{owner_repo}"
    try:
        print(f"Fetching single repo: {url}")
        response = requests.get(url, headers=const.GITHUB_FETCH_HEADERS)
        response.raise_for_status()
        repo_data = response.json()
        return convertGithubRepoFormToZigistryRepoForm(repo_data)
    except Exception as e:
        print(f"Error fetching single repo {url}: {e}")
        return None

if __name__ == "__main__":
    delay_interval = 0.3  # seconds between thread launches

    with ThreadPoolExecutor() as executor:
        futures = []
        for index, url in enumerate(const.PROGRAMS_URLS):
            delay = index * delay_interval
            futures.append(executor.submit(fetch_convert_with_delay, url, delay))

        all_repos_nested = []
        for future in as_completed(futures):
            all_repos_nested.append(future.result())

    # Flatten the results
    flat_repos = [repo for sublist in all_repos_nested for repo in sublist]

    # --- NEW: Read github_additions.json and append manually listed program repos ---
    manual_addition_repos = []
    if os.path.exists(GITHUB_ADDITIONS_PATH):
        with open(GITHUB_ADDITIONS_PATH, "r") as f:
            additions = json.load(f)
            program_repos = additions.get("programs", [])
            with ThreadPoolExecutor() as executor:
                manual_futures = [
                    executor.submit(fetch_repo_by_full_name, repo_full_name)
                    for repo_full_name in program_repos
                ]
                for future in as_completed(manual_futures):
                    repo = future.result()
                    if repo is not None:
                        manual_addition_repos.append(repo)
    else:
        print(f"Manual additions file {GITHUB_ADDITIONS_PATH} not found, skipping manual additions.")

    # Append manual program additions to repos list
    flat_repos.extend(manual_addition_repos)

    # Convert to dicts concurrently
    with ThreadPoolExecutor() as executor:
        dict_repos = list(executor.map(asdict, flat_repos))

    # Write to file
    with open("./database/programs.json", "w") as output_file:
        json.dump(dict_repos, output_file)