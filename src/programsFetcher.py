import requests
import json
import concurrent.futures
from libs import utils, constants


def fetch_url(url):
    res = requests.get(url, headers=constants.GITHUB_FETCH_HEADERS)
    if not res.ok:
        print("RESPONSE NOT OK")
        exit(1)
    repos = res.json().get("items", [])
    with concurrent.futures.ThreadPoolExecutor() as executor:
        return list(executor.map(utils.process_repo, repos))


with concurrent.futures.ThreadPoolExecutor() as executor:
    results = list(executor.map(fetch_url, constants.PROGRAMS_URL))

data = [item for sublist in results for item in sublist]

with open("./jsons/programs.json", "w") as f:
    json.dump(data, f)
