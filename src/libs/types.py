from dataclasses import dataclass, field
from typing import List, Literal, Optional


@dataclass
class Dependency:
    name: str
    url: str
    commit: Optional[str] = None
    tar_url: Optional[str] = None
    type: str = "unknown"  # "remote", "system", or "local"


@dataclass
class Repo:
    avatar_url: str
    name: str
    full_name: str
    created_at: str
    description: Optional[str]
    default_branch: str
    open_issues: int
    stargazers_count: int
    forks_count: int
    watchers_count: int
    tags_url: str
    license: str
    topics: List[str]
    size: int
    fork: bool
    updated_at: str
    has_build_zig: bool
    has_build_zig_zon: bool
    zig_minimum_version: str
    repo_from: Literal["github", "gitlab", "codeberg"]  # moved above
    dependencies: List[Dependency] = field(default_factory=list)
    readme_content: Optional[str] = None
    dependents: Optional[str] = None


# Rest of the code remains the same...
