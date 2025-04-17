# --------- Imports --------- #
from libs import constants as const
from libs.types import Repo
from libs.utils import convertGithubRepoFormToZigistryRepoForm
import requests
from dataclasses import asdict
import json
from concurrent.futures import ThreadPoolExecutor, as_completed
import time
# --------------------------- #

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

    # Convert to dicts concurrently
    with ThreadPoolExecutor() as executor:
        dict_repos = list(executor.map(asdict, flat_repos))

    # Write to file
    with open("programs.json", "w") as output_file:
        json.dump(dict_repos, output_file)
