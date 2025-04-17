from dataclasses import dataclass

@dataclass
class Repo:
    avatar_url: str
    name: str
    full_name: str
    created_at: str
    description: str
    default_branch: str
    open_issues: int
    stargazers_count: int
    forks_count: int
    watchers_count: int
    tags_url: str
    license: str
    topics: list[str] | None
    size: int
    fork: bool
    updated_at: str
    has_build_zig: bool
    has_build_zig_zon: bool
    readme_content: str
