import json

def add_dependents(filepath: str):
    with open(filepath, "r") as f:
        repos = json.load(f)

    # Create a map from repo name to the repo object
    name_to_repo = {repo["name"]: repo for repo in repos}

    # Initialize dependents for each repo
    for repo in repos:
        repo["dependents"] = []

    # Populate dependents
    for repo in repos:
        for dep in repo.get("dependencies", []):
            dep_name = dep.get("name")
            if dep_name in name_to_repo:
                name_to_repo[dep_name]["dependents"].append(repo["name"])

    # Overwrite the original file
    with open(filepath, "w") as f:
        json.dump(repos, f, indent=2)

# Process both files
add_dependents("./database/packages.json")
add_dependents("./database/programs.json")
