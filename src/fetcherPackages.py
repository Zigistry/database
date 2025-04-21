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
    # I added this delay so that there can be some delay between calling GitHub's api twice.
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

    # Convert to dicts concurrently
    with ThreadPoolExecutor() as executor:
        dict_packages = list(executor.map(asdict, flat_packages))

    # Write to file
    with open("./database/packages.json", "w") as output_file:
        json.dump(dict_packages, output_file)
