import requests
import json
import concurrent.futures
from libs import utils

TOPIC_URLS = {
    "./jsons/games.json": (
        "https://api.github.com/repos/Not-Nik/raylib-zig",
        "https://api.github.com/repos/hexops/mach",
        "https://api.github.com/repos/zig-gamedev/zig-gamedev",
        "https://api.github.com/repos/Jack-Ji/jok",
        "https://api.github.com/repos/prime31/zig-gamekit",
    ),
    "./jsons/web.json": (
        "https://api.github.com/repos/zigzap/zap",
        "https://api.github.com/repos/jetzig-framework/jetzig",
        "https://api.github.com/repos/karlseguin/http.zig",
        "https://api.github.com/repos/karlseguin/websocket.zig",
        "https://api.github.com/repos/ikskuh/zig-network",
    ),
    "./jsons/gui.json": (
        "https://api.github.com/repos/capy-ui/capy",
        "https://api.github.com/repos/webui-dev/zig-webui",
        "https://api.github.com/repos/david-vanderson/dvui",
        "https://api.github.com/repos/kassane/qml_zig",
        "https://api.github.com/repos/MoAlyousef/zfltk",
    ),
}

def fetch_repo(url):
    res = requests.get(url)
    if not res.ok:
        return None
    return utils.process_repo(res.json())

def process_category(file_name, urls):
    with concurrent.futures.ThreadPoolExecutor() as executor:
        data = list(filter(None, executor.map(fetch_repo, urls)))
    with open(file_name, "w") as f:
        json.dump(data, f)

with concurrent.futures.ThreadPoolExecutor() as executor:
    executor.map(lambda item: process_category(*item), TOPIC_URLS.items())
