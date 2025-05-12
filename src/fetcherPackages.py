# --------- Imports --------- #
from libs import constants as const
from libs.types import Repo
from libs.utils import convertGithubRepoFormToZigistryRepoForm
import requests
from dataclasses import asdict
import json
from concurrent.futures import ThreadPoolExecutor, as_completed
import time
from libs.constants import INDEX_PAGE_SECTION_TOPIC_URLS
# --------------------------- #

ALL_INDEX_PAGE_REPO_URLS = [
    url
    for urls in INDEX_PAGE_SECTION_TOPIC_URLS.values()
    for url in urls
]


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

# --- NEW: fetch and convert a single GitHub repo (not search) ---
def fetch_single_github_repo_and_convert(url: str) -> Repo | None:
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
    delay_interval = 1

    with ThreadPoolExecutor() as executor:
        futures = []
        for index, url in enumerate(const.PACKAGES_URLS):
            delay = index * delay_interval
            futures.append(executor.submit(fetch_convert_with_delay, url, delay))

        all_packages_nested = []
        for future in as_completed(futures):
            all_packages_nested.append(future.result())

    # Flatten the results
    flat_packages = [pkg for sublist in all_packages_nested for pkg in sublist]

    # --- NEW: Fetch and append the INDEX_PAGE_SECTION_TOPIC_URLS repos ---
    with ThreadPoolExecutor() as executor:
        index_page_futures = [
            executor.submit(fetch_single_github_repo_and_convert, url)
            for url in ALL_INDEX_PAGE_REPO_URLS
        ]
        for future in as_completed(index_page_futures):
            repo = future.result()
            if repo is not None:
                flat_packages.append(repo)

    # Convert to dicts concurrently
    with ThreadPoolExecutor() as executor:
        dict_packages = list(executor.map(asdict, flat_packages))

    # Write to file
    with open("./database/packages.json", "w") as output_file:
        json.dump(dict_packages, output_file)