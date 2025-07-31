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
    "https://api.github.com/search/repositories?q=topic:zig-package+fork:true+created:2025-02-09..2025-08-08&page=2&per_page=100",
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
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2025-02-09..2025-08-08&page=3&per_page=100",
    "https://api.github.com/search/repositories?q=topic:zig+fork:true+created:2025-02-09..2025-08-08&page=4&per_page=100",
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
    "games": (
        "https://api.github.com/repos/Not-Nik/raylib-zig",
        "https://api.github.com/repos/hexops/mach",
        "https://api.github.com/repos/zig-gamedev/zig-gamedev",
        "https://api.github.com/repos/Jack-Ji/jok",
        "https://api.github.com/repos/prime31/zig-gamekit",
        "https://api.github.com/repos/prime31/zig-ecs",
        "https://api.github.com/repos/JonSnowbd/slingworks",
        "https://api.github.com/repos/michal-z/zig-gamedev",
        "https://api.github.com/repos/Kiakra/Alka",
        "https://api.github.com/repos/leroycep/seizer",
        "https://api.github.com/repos/zenith391/didot",
        "https://api.github.com/repos/danielabbott/Zig-Game-Engine",
        "https://api.github.com/repos/fubark/cosmic",
        "https://api.github.com/repos/bootradev/cupcake",
        "https://api.github.com/repos/gamercade-io/zig-template",
        "https://api.github.com/repos/Guigui220D/zig-sfml-wrapper",
        "https://api.github.com/repos/MasterQ32/SDL.zig",
        "https://api.github.com/repos/Snektron/vulkan-zig",
        "https://api.github.com/repos/linuxy/godot-zig",
        "https://api.github.com/repos/silversquirl/phyz",
    ),
    "web": (
        "https://api.github.com/repos/zigzap/zap",
        "https://api.github.com/repos/jetzig-framework/jetzig",
        "https://api.github.com/repos/karlseguin/http.zig",
        "https://api.github.com/repos/karlseguin/websocket.zig",
        "https://api.github.com/repos/ikskuh/zig-network",
        "https://api.github.com/repos/frmdstryr/zhp",
        "https://api.github.com/repos/Vexu/routez",
        "https://api.github.com/repos/Luukdegram/apple_pie",
        "https://api.github.com/repos/ducdetronquito/requestz",
        "https://api.github.com/repos/ducdetronquito/h11",
        "https://api.github.com/repos/truemedian/zfetch",
        "https://api.github.com/repos/lun-4/ziget",
    ),
    "gui": (
        "https://api.github.com/repos/capy-ui/capy",
        "https://api.github.com/repos/webui-dev/zig-webui",
        "https://api.github.com/repos/david-vanderson/dvui",
        "https://api.github.com/repos/kassane/qml_zig",
        "https://api.github.com/repos/MoAlyousef/zfltk",
        "https://api.github.com/repos/Aransentin/ZWL",
        "https://api.github.com/repos/prime31/zig-upaya",
        "https://api.github.com/repos/fubark/cosmic",
        "https://api.github.com/repos/batiati/IUPforZig",
        "https://api.github.com/repos/olexij-christian/zgtk3",
        "https://api.github.com/repos/andrewrk/zig-sdl",
        "https://api.github.com/repos/Snektron/vulkan-zig",
        "https://api.github.com/repos/rcalixte/libqt6zig",
    ),
}

HUGGING_FACE_API_KEY = os.getenv("HF_AUTH_TOKEN")

PROGRAMS_FILES = ("./database/programs.json",)

PACKAGES_FILES = ("./database/packages.json",)

ALL_DATABASE_FILES = PROGRAMS_FILES + PACKAGES_FILES
