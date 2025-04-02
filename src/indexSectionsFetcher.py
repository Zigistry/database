import requests
import json
import concurrent.futures
from libs import utils, constants


def fetch_repo(url):
    res = requests.get(url)
    if not res.ok:
        print("RESPONSE NOT OK")
        exit(1)
    return utils.process_repo(res.json())


def process_category(file_name, urls):
    with concurrent.futures.ThreadPoolExecutor() as executor:
        data = list(filter(None, executor.map(fetch_repo, urls)))
    with open(file_name, "w") as f:
        json.dump(data, f)


with concurrent.futures.ThreadPoolExecutor() as executor:
    executor.map(
        lambda item: process_category(*item),
        constants.INDEX_PAGE_SECTION_TOPIC_URLS.items(),
    )
