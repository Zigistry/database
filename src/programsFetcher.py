import requests
import json
import concurrent.futures
from libs import utils

URLS = (
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:0&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:0&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:0&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:0&page=4&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:0&page=5&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:0&page=6&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:0&page=7&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:0&page=8&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:1&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:1&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:1&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:1&page=4&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:2&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:2&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:2&page=3&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:3&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:3&page=2&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:4&page=1&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:5&page=1&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:6..10&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:6..10&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:6..10&page=3&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:11..20&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:11..20&page=2&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:21..100&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:21..100&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:21..100&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:21..100&page=4&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:101..1000&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:101..1000&page=2&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:1001..5000&page=1&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:5001..50000&page=1&per_page=100",
    
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+stars:%3E=50000&page=1&per_page=100",
)

def fetch_url(url):
    res = requests.get(url, headers=utils.HEADERS)
    if not res.ok:
        print("RESPONSE NOT OK")
        exit(1)
    repos = res.json().get("items", [])
    with concurrent.futures.ThreadPoolExecutor() as executor:
        return list(executor.map(utils.process_repo, repos))

with concurrent.futures.ThreadPoolExecutor() as executor:
    results = list(executor.map(fetch_url, URLS))

data = [item for sublist in results for item in sublist]

with open("./jsons/programs.json", "w") as f:
    json.dump(data, f)
