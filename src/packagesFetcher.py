import concurrent.futures
import json
import requests

from libs import utils

def fetch_repos():
    INITIAL_URL = "https://api.github.com/search/repositories?q=topic:zig-package"
    res = requests.get(INITIAL_URL, headers=utils.HEADERS)
    if not res.ok:
        exit()
    
    total_count = res.json()["total_count"]
    data = []
    urls = [f"{INITIAL_URL}&page={i}&per_page=100" for i in range(1, total_count // 100 + 2)]
    
    with concurrent.futures.ThreadPoolExecutor() as executor:
        results = executor.map(lambda url: requests.get(url, headers=utils.HEADERS).json().get("items", []), urls)
    
    repos = [repo for result in results for repo in result]
    
    with concurrent.futures.ThreadPoolExecutor() as executor:
        data = list(executor.map(utils.process_repo, repos))

    with open("./jsons/main.json", "w") as f:
        json.dump(data, f)

fetch_repos()
