import re
import requests
import json

def remove_comments(input_str):
    def replacement(match):
        string_match = match.group(1)
        if string_match is not None:
            return string_match
        return ""

    pattern = r"(\"(?:\\.|[^\"\\])*\")|\/\/.*|\/\*[\s\S]*?\*\/"
    return re.sub(pattern, replacement, input_str)


def zon2json(input_str):
    input_str = remove_comments(input_str)

    # Replace .{ with {
    input_str = re.sub(r"\.{", "{", input_str)

    # Replace .field = "value" or .field: "value" with "field": "value"
    input_str = re.sub(r"\.([a-zA-Z0-9_-]+)\s*(=|:)\s*", r'"\1": ', input_str)

    # Remove the .@ prefix and handle it correctly
    input_str = re.sub(r"\.@([a-zA-Z0-9_-]+)\s*(=|:)\s*", r'"\1": ', input_str)

    # Remove unnecessary dots before braces
    input_str = re.sub(r"\.\s*\{", "{", input_str)

    # Handle .@"key" as "key"
    input_str = re.sub(r'\.@"([a-zA-Z0-9_-]+)"\s*(=|:)\s*', r'"\1": ', input_str)

    # Convert Zon-style arrays to JSON arrays
    def array_handler(match):
        array_content = match.group(1)
        if ":" in array_content:
            return f"{{{array_content}}}"  # It's an object, leave it as is
        else:
            # Handle Zon arrays: format content as JSON array
            formatted_array = ", ".join(
                item.strip() for item in array_content.split(",")
            )
            return f"[{formatted_array}]"  # Convert to JSON array

    input_str = re.sub(r"\{([^:]+?)\}", array_handler, input_str)

    # Remove trailing commas in objects or arrays
    input_str = re.sub(r",(\s*[}\]])", r"\1", input_str)

    return input_str


import requests
import re
import json
from typing import Dict, List, Any, Optional


def get_repo_zon_metadata(repo_full_name: str) -> Dict[str, Any]:
    """
    Fetches build.zig.zon from `repo_full_name` and extracts available metadata.
    All fields in build.zig.zon are optional, so this function handles missing fields gracefully.
    Never raises—on any parse/network issue it just returns what it found.

    Args:
        repo_full_name: The full repository name (owner/repo)

    Returns:
        Dict containing:
        - zig_version: String version or "unknown"
        - dependencies: List of dependency objects with name, source, and location
    """
    zig_version = "unknown"
    dependencies: List[Dict[str, str]] = []

    url = f"https://raw.githubusercontent.com/{repo_full_name}/master/build.zig.zon"
    try:
        r = requests.get(url, timeout=5)
        if r.status_code != 200:
            return {"zig_version": zig_version, "dependencies": dependencies}

        zon_raw = r.text

        # Try parsing with zon2json first
        try:
            js = zon2json(zon_raw)
            data = json.loads(js)

            # Extract minimum_zig_version if present
            if isinstance(data, dict):
                zig_version = data.get("minimum_zig_version", zig_version)

                # Handle dependencies if present
                deps = data.get("dependencies", {})
                if isinstance(deps, dict):
                    for name, dep in deps.items():
                        if not isinstance(dep, dict):
                            continue

                        dep_info = {"name": name}

                        # Remote dependency with URL
                        if "url" in dep:
                            dep_info.update(
                                {"source": "remote", "location": dep["url"]}
                            )
                        # Local dependency with path
                        elif "path" in dep:
                            dep_info.update(
                                {"source": "relative", "location": dep["path"]}
                            )
                        # System dependency or other types
                        else:
                            dep_info.update({"source": "system", "location": ""})

                        dependencies.append(dep_info)

        except (json.JSONDecodeError, Exception):
            # Fallback to regex for minimum_zig_version if JSON parsing fails
            m = re.search(r'\.minimum_zig_version\s*=\s*"([^"]+)"', zon_raw)
            if m:
                zig_version = m.group(1)

            # Try to extract dependencies using regex as fallback
            dep_matches = re.finditer(
                r'\.([a-zA-Z0-9_-]+)\s*=\s*\.{\s*(?:\.url\s*=\s*"([^"]+)"|\.path\s*=\s*"([^"]+)")',
                zon_raw,
            )
            for match in dep_matches:
                name = match.group(1)
                url = match.group(2)
                path = match.group(3)

                if url:
                    dependencies.append(
                        {"name": name, "source": "remote", "location": url}
                    )
                elif path:
                    dependencies.append(
                        {"name": name, "source": "relative", "location": path}
                    )

    except requests.RequestException:
        # Handle network errors silently
        pass

    return {"zig_version": zig_version, "dependencies": dependencies}
