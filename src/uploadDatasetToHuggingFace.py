from datasets import Dataset
from libs import constants as const
from typing import List, Iterable
import json


def remove_duplicates(repos: List[dict]) -> List[dict]:
    seen = set()
    unique = []
    for repo in repos:
        name = repo["full_name"]
        if name not in seen:
            seen.add(name)
            unique.append(repo)
    return unique


def load_repos(file_paths: Iterable[str]) -> List[dict]:
    all_repos = []
    for path in file_paths:
        with open(path, "r") as f:
            repos = json.load(f)
            all_repos.extend(repos)
    return all_repos


packages = remove_duplicates(load_repos(const.PACKAGES_FILES))
Dataset.from_list(packages).push_to_hub(
    "Zigistry/packages", token=const.HUGGING_FACE_API_KEY
)

programs = remove_duplicates(load_repos(const.PROGRAMS_FILES))
Dataset.from_list(programs).push_to_hub(
    "Zigistry/programs", token=const.HUGGING_FACE_API_KEY
)

all_data = remove_duplicates(load_repos(const.ALL_DATABASE_FILES))
Dataset.from_list(all_data).push_to_hub(
    "Zigistry/Zigistry-complete-dataset", token=const.HUGGING_FACE_API_KEY
)
