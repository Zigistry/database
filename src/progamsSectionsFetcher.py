import json
from libs.types import Repo
from typing import List
from dataclasses import asdict, is_dataclass

FIELDS_TO_REMOVE = {
    "dependents",
    "readme_content",
    "dependencies",
    "zig_minimum_version",
    "size",
    "tags_url",
    "default_branch"
}

def load_repos(filename: str) -> List[Repo]:
    with open(filename, 'r', encoding='utf-8') as f:
        data = json.load(f)
    return [Repo(**repo) for repo in data]

def recursive_asdict(obj):
    """Recursively turn dataclasses and custom objects into dicts/lists for JSON serialization."""
    if is_dataclass(obj):
        return {k: recursive_asdict(v) for k, v in asdict(obj).items()}
    elif isinstance(obj, dict):
        return {k: recursive_asdict(v) for k, v in obj.items()}
    elif isinstance(obj, (list, tuple)):
        return [recursive_asdict(v) for v in obj]
    else:
        return obj

def strip_unnecessary_fields(repo_list):
    stripped = []
    for repo in repo_list:
        # repo is a dict at this point
        filtered = {k: v for k, v in repo.items() if k not in FIELDS_TO_REMOVE}
        stripped.append(filtered)
    return stripped

def serialize_repos(repo_list):
    return strip_unnecessary_fields([recursive_asdict(r) for r in repo_list])

def main():
    repos = load_repos('./database/programs.json')

    # Sort by created_at (latest first)
    sorted_by_created = sorted(repos, key=lambda r: r.created_at, reverse=True)
    # Sort by stargazers_count (most used)
    sorted_by_stars = sorted(repos, key=lambda r: r.stargazers_count, reverse=True)

    # Slice top 10
    top_10_latest_created = sorted_by_created[:10]
    top_10_most_used = sorted_by_stars[:10]

    programs_details = {
        "top10latestrepos": serialize_repos(top_10_latest_created),
        "mostused": serialize_repos(top_10_most_used)
    }

    with open("./database/progams_details.json", "w", encoding="utf-8") as f:
        json.dump(programs_details, f, ensure_ascii=False)

if __name__ == "__main__":
    main()