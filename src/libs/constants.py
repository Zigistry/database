import os

HUGGING_FACE_API_KEY = os.getenv("HF_AUTH_TOKEN")
GITHUB_API_KEY = os.getenv("GH_API_KEY")

PACKAGES_JSON_FILES = ("main.json", "web.json", "games.json", "gui.json")
PROGRAMS_JSON_FILES = ("programs.json",)
ALL_JSON_FILES = PACKAGES_JSON_FILES + PROGRAMS_JSON_FILES

POSSIBLE_README_FILENAMES = (
    "README.md",
    "README.txt",
    "README",
    "readme.md",
    "readme.txt",
    "README.markdown",
    "readme.markdown",
)

GITHUB_FETCH_HEADERS = {
    "Authorization": f"Bearer {GITHUB_API_KEY}",
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "Python",
}

INDEX_PAGE_SECTION_TOPIC_URLS = {
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


PROGRAMS_URL = (
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
