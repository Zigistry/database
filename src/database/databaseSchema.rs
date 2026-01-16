
pub const INIT_REPOS = r#"
CREATE TABLE repos (
  id TEXT PRIMARY KEY,
  owner TEXT NOT NULL,
  avatar_id TEXT NOT NULL,
  description TEXT,
  issues_count INTEGER NOT NULL,
  default_branch_name TEXT NOT NULL,
  fork_count INTEGER NOT NULL,
  stargazer_count INTEGER NOT NULL,
  watchers_count INTEGER NOT NULL,
  pushed_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  is_archived INTEGER NOT NULL,
  is_disabled INTEGER NOT NULL,
  is_fork INTEGER NOT NULL,
  license TEXT NOT NULL,
  minimum_zig_version TEXT,
  readme_url TEXT,
  primary_language TEXT NOT NULL,
  FOREIGN KEY (owner) REFERENCES users(id)
);
"#;

pub const INIT_REPO_TOPICS = r#"
  CREATE TABLE repo_topics (
    repo_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    FOREIGN KEY (repo_id) REFERENCES repos(id)
  )
"#;

pub const INIT_USERS = r#"
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  avatar_id TEXT NOT NULL,
  bio TEXT,
  company TEXT,
  followers INTEGER NOT NULL,
  following INTEGER NOT NULL,
  location TEXT,
  description TEXT,
  website_url TEXT
)
"#;
pub const INIT_REPO_DEPENDENTS = r#"
CREATE TABLE repo_dependents (
  repo_id TEXT NOT NULL,
  dependent TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos(id)
)
"#;
pub const INIT_RELEASES = r#"
CREATE TABLE releases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id TEXT NOT NULL,
  version TEXT NOT NULL,
  is_prerelease INTEGER NOT NULL,
  published_at TEXT NOT NULL,
  minimum_zig_version TEXT NOT NULL,
  readme_url TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos(id)
)
"#;
pub const INIT_RELEASE_DEPENDENCIES = r#"
CREATE TABLE release_dependencies (
  release_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  hash TEXT NOT NULL,
  lazy TEXT NOT NULL,
  url TEXT NOT NULL,
  path TEXT NOT NULL,
  FOREIGN KEY (release_id) REFERENCES releases(id)
)
"#;
pub const INIT_INDEX_SECTION= r#"CREATE TABLE index_sections (
  section_name TEXT NOT NULL,
  repo_id TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos(id)
)
"#;

