import json
from typing import List, Dict, Any


def add_dependents(packages_path: str, programs_path: str):
    """
    Process dependencies and add dependents across packages and programs.

    Valid dependency relationships:
    - Programs can depend on packages
    - Packages can depend on programs
    - Packages can depend on packages
    - Programs can depend on programs
    """
    # Load both files
    with open(packages_path, "r") as f:
        packages = json.load(f)
    with open(programs_path, "r") as f:
        programs = json.load(f)

    # Create maps for both packages and programs separately
    package_map: Dict[str, Dict] = {repo["name"]: repo for repo in packages}
    program_map: Dict[str, Dict] = {repo["name"]: repo for repo in programs}

    # Initialize dependents lists
    for repo in packages + programs:
        repo["dependents"] = []

    def process_dependencies(
        repos: List[Dict[str, Any]],
        package_map: Dict[str, Dict],
        program_map: Dict[str, Dict],
    ):
        """Process dependencies for a list of repositories."""
        for repo in repos:
            for dep in repo.get("dependencies", []):
                dep_name = dep.get("name", "")
                if not dep_name:
                    continue

                dependent_url = f"https://github.com/{repo['full_name']}"

                # Check in packages
                if dep_name in package_map:
                    if dependent_url not in package_map[dep_name]["dependents"]:
                        package_map[dep_name]["dependents"].append(dependent_url)

                # Check in programs
                if dep_name in program_map:
                    if dependent_url not in program_map[dep_name]["dependents"]:
                        program_map[dep_name]["dependents"].append(dependent_url)

    # Process dependencies for both packages and programs
    process_dependencies(packages, package_map, program_map)
    process_dependencies(programs, package_map, program_map)

    # Sort dependents lists for consistency
    for repo in packages + programs:
        repo["dependents"] = sorted(list(set(repo["dependents"])))

    # Save files with formatted JSON
    with open(packages_path, "w") as f:
        json.dump(packages, f)

    with open(programs_path, "w") as f:
        json.dump(programs, f)


if __name__ == "__main__":
    add_dependents("./database/packages.json", "./database/programs.json")
