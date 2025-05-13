import json
from libs.utils import convert2markdown

INPUT_FILE = "./database/packages.json"
INPUT_FILE2 = "./database/programs.json"

def purify(file_name:str):
    with open(file_name, "r", encoding="utf-8") as f:
        repos = json.load(f)

    for repo in repos:
        readme = repo.get("readme_content")
        if readme is not None:
            repo["readme_content"] = convert2markdown(readme)

    with open(file_name, "w", encoding="utf-8") as f:
        json.dump(repos, f)

if __name__ == "__main__":
    purify(INPUT_FILE)
    purify(INPUT_FILE2)
