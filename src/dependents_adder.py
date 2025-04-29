import json

def add_dependents(filepath: str):
    with open(filepath, "r") as f:
        repos = json.load(f)

    # Create maps for both name-to-repo and full_name-to-repo lookups
    name_to_repo = {repo["name"]: repo for repo in repos}
    full_name_to_repo = {repo["full_name"]: repo for repo in repos}

    # Initialize dependents for each repo
    for repo in repos:
        repo["dependents"] = []

    # Populate dependents with GitHub URLs
    for repo in repos:
        for dep in repo.get("dependencies", []):
            dep_name = dep.get("name")
            if dep_name in name_to_repo:
                # Get the dependent repo's full GitHub URL
                dependent_url = f"https://github.com/{repo['full_name']}"
                name_to_repo[dep_name]["dependents"].append(dependent_url)

    # Overwrite the original file
    with open(filepath, "w") as f:
        json.dump(repos, f, indent=2)

# Process both files
add_dependents("./database/packages.json")
add_dependents("./database/programs.json")