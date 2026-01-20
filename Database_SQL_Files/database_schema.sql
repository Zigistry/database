-- Also, I have added avatar_id
-- to both repos and userm even though they are the same
-- Just for normalization
-- I don't want, sql to process 2 queries on every request,
-- too much latency. To cut down on latency, avatar_id will
-- be duplicated for both repos and users.
CREATE TABLE repos (
  -- The username is 40 characters at max, repo name is 100 and the key and slashes is 5
  -- Hence, I can keep this at 150.
  id VARCHAR(150) PRIMARY KEY,
  -- for github its the username itself, for codeberg its 64 bits, hence, I'll keep it at 65
  avatar_id VARCHAR(65) NOT NULL,
  -- 40 + the key and the slash i.e 3, so I think best is 45 for htis.
  owner VARCHAR(45) NOT NULL,
  -- GITHUB CODEBERG like, 8 characters, so 10
  platform VARCHAR(10) NOT NULL,
  -- This has afaik, 255, hence, 260
  description VARCHAR(260),
  issues_count INTEGER NOT NULL,
  -- This should also be 255, hence, 260
  default_branch_name VARCHAR(260) NOT NULL,
  fork_count INTEGER NOT NULL,
  stargazer_count INTEGER NOT NULL,
  watchers_count INTEGER NOT NULL,
  pushed_at TIMESTAMP NOT NULL,
  created_at TIMESTAMP NOT NULL,
  is_archived BOOLEAN NOT NULL,
  is_disabled BOOLEAN NOT NULL,
  is_fork BOOLEAN NOT NULL,
  -- Simple google tells, its just 36 characters, so, ok?
  license VARCHAR(40) NOT NULL,
  -- Maybe 50 is perfect here
  primary_language VARCHAR(50) NOT NULL,
  search_keywords TEXT NOT NULL,
  FOREIGN KEY (owner) REFERENCES users (id)
);

CREATE TABLE repo_topics (
  repo_id VARCHAR(150) NOT NULL,
  -- Limit is 50, hence
  topic VARCHAR(60) NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id)
);

CREATE TABLE users (
  id VARCHAR(45) PRIMARY KEY,
  avatar_id VARCHAR(65) NOT NULL,
  platform VARCHAR(10) NOT NULL,
  bio VARCHAR(260)
);

CREATE TABLE repo_dependents (
  repo_id VARCHAR(150) NOT NULL,
  dependent TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id)
);

CREATE TABLE releases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id VARCHAR(150) NOT NULL,
  version VARCHAR(255),
  is_prerelease BOOLEAN NOT NULL,
  published_at TIMESTAMP NOT NULL,
  minimum_zig_version TEXT NOT NULL,
  readme_url TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id)
);

CREATE TABLE release_dependencies (
  release_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  hash TEXT NOT NULL,
  lazy TEXT NOT NULL,
  url TEXT NOT NULL,
  path TEXT NOT NULL,
  FOREIGN KEY (release_id) REFERENCES releases (id)
);

CREATE TABLE index_sections (
  section_name VARCHAR(10) NOT NULL,
  repo_id VARCHAR(150) NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id)
);

CREATE TABLE packages (
  repo_id VARCHAR(150) NOT NULL PRIMARY KEY,
  FOREIGN KEY (repo_id) REFERENCES repos (id)
);

CREATE TABLE programs (
  repo_id VARCHAR(150) NOT NULL PRIMARY KEY,
  FOREIGN KEY (repo_id) REFERENCES repos (id)
);
