import os

PACKAGES_URLS = (
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2016-02-08..2019-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2019-02-09..2020-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2020-02-09..2021-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2021-02-09..2022-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2022-02-09..2023-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2023-02-09..2024-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2023-02-09..2024-02-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2024-02-09..2024-08-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2024-02-09..2024-08-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2024-08-09..2025-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2024-08-09..2025-02-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2025-02-09..2025-08-08&page=1&per_page=100",
)

PROGRAMS_URLS = (
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2016-02-08..2019-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2019-02-09..2020-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2019-02-09..2020-02-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2020-02-09..2021-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2020-02-09..2021-02-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2020-02-09..2021-02-08&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2021-02-09..2022-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2021-02-09..2022-02-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2021-02-09..2022-02-08&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2021-02-09..2022-02-08&page=4&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2022-02-09..2023-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2022-02-09..2023-02-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2022-02-09..2023-02-08&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2022-02-09..2023-02-08&page=4&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2022-02-09..2023-02-08&page=5&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2023-02-09..2024-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2023-02-09..2024-02-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2023-02-09..2024-02-08&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2023-02-09..2024-02-08&page=4&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2023-02-09..2024-02-08&page=5&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2023-02-09..2024-02-08&page=6&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2023-02-09..2024-02-08&page=7&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-02-09..2024-08-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-02-09..2024-08-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-02-09..2024-08-08&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-02-09..2024-08-08&page=4&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-02-09..2024-08-08&page=5&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-08-09..2025-02-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-08-09..2025-02-08&page=2&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-08-09..2025-02-08&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-08-09..2025-02-08&page=4&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-08-09..2025-02-08&page=5&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2024-08-09..2025-02-08&page=6&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2025-02-09..2025-08-08&page=1&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2025-02-09..2025-08-08&page=2&per_page=100",
)

GITHUB_API_KEY = os.getenv("GH_API_KEY")

GITHUB_FETCH_HEADERS = {
    "Authorization": f"Bearer {GITHUB_API_KEY}",
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "Python",
}

POSSIBLE_README_FILENAMES = (
    "README.md",
    "README.txt",
    "README",
    "readme.md",
    "readme.txt",
    "README.markdown",
    "readme.markdown",
)

INDEX_PAGE_SECTION_TOPIC_URLS = {
    "./database/games.json": (
        "https://api.github.com/repos/Not-Nik/raylib-zig",
        "https://api.github.com/repos/hexops/mach",
        "https://api.github.com/repos/zig-gamedev/zig-gamedev",
        "https://api.github.com/repos/Jack-Ji/jok",
        "https://api.github.com/repos/prime31/zig-gamekit",
    ),
    "./database/web.json": (
        "https://api.github.com/repos/zigzap/zap",
        "https://api.github.com/repos/jetzig-framework/jetzig",
        "https://api.github.com/repos/karlseguin/http.zig",
        "https://api.github.com/repos/karlseguin/websocket.zig",
        "https://api.github.com/repos/ikskuh/zig-network",
    ),
    "./database/gui.json": (
        "https://api.github.com/repos/capy-ui/capy",
        "https://api.github.com/repos/webui-dev/zig-webui",
        "https://api.github.com/repos/david-vanderson/dvui",
        "https://api.github.com/repos/kassane/qml_zig",
        "https://api.github.com/repos/MoAlyousef/zfltk",
    ),
}

HUGGING_FACE_API_KEY = os.getenv("HF_AUTH_TOKEN")

PROGRAMS_FILES = ("./database/programs.json",)

PACKAGES_FILES = (
    "./database/packages.json",
    "./database/games.json",
    "./database/gui.json",
    "./database/web.json",
)

ALL_DATABASE_FILES = PROGRAMS_FILES + PACKAGES_FILES
