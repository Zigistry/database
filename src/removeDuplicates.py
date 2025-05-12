import json
from libs.types import Repo


def removeDuplicates(json_path):
    # Read JSON data
    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    # Remove duplicates based on (full_name, repo_from)
    seen = set()
    unique_repos = []
    for repo_dict in data:
        full_name = repo_dict.get('full_name')
        repo_from = repo_dict.get('repo_from')
        key = (full_name, repo_from)
        if key not in seen:
            seen.add(key)
            unique_repos.append(repo_dict)

    # Write back to file
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(unique_repos, f, ensure_ascii=False)

    print(f"Removed duplicates, {len(data) - len(unique_repos)} entries deleted.")


if __name__ == "__main__":
    removeDuplicates("./database/packages.json")
    removeDuplicates("./database/programs.json")
