import requests
import json
import concurrent.futures
from dataclasses import asdict
from libs import utils, constants


def fetch_repo(url):
    res = requests.get(url, headers=constants.GITHUB_FETCH_HEADERS)
    if not res.ok:
        print(f"Failed to fetch: {url} → {res.status_code}")
        return None
    raw_repo = res.json()
    converted = utils.convertGithubRepoFormToZigistryRepoForm(raw_repo)
    return asdict(converted) if converted else None


def process_category(file_name, urls):
    with concurrent.futures.ThreadPoolExecutor() as executor:
        data = list(filter(None, executor.map(fetch_repo, urls)))
    with open(file_name, "w") as f:
        json.dump(data, f)


def main():
    with concurrent.futures.ThreadPoolExecutor() as executor:
        executor.map(
            lambda item: process_category(*item),
            constants.INDEX_PAGE_SECTION_TOPIC_URLS.items(),
        )


if __name__ == "__main__":
    main()
