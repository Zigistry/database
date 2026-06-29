CREATE TABLE users (
  id VARCHAR(45) PRIMARY KEY,
  avatar_id VARCHAR(65) NOT NULL,
  platform VARCHAR(10) NOT NULL,
  bio VARCHAR(260)
);
CREATE TABLE repos (
  -- The username is 40 characters at max, repo name is 100 and the key and slashes is 5
  -- Hence, I can keep this at 150.
  id VARCHAR(150) PRIMARY KEY,
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
  -- I just realised that the boolean type is like an integer only.
  -- I added check that it should only be either true or false. i.e 0, 1
  is_archived BOOLEAN NOT NULL CHECK (
    is_archived IN (0, 1)
  ),
  is_disabled BOOLEAN NOT NULL CHECK (
    is_disabled IN (0, 1)
  ),
  is_fork BOOLEAN NOT NULL CHECK (
    is_fork IN (0, 1)
  ),
  -- Making this only 30 characters limit, it shouldn't be more than that.
  -- Simple google tells, its just 36 characters, so, ok?
  license VARCHAR(40) NOT NULL,
  -- Maybe 50 is perfect here
  primary_language VARCHAR(50) NOT NULL,
  latest_commit_hash VARCHAR(50) NOT NULL,
  last_updated_in_this_database TIMESTAMP NOT NULL,
  FOREIGN KEY (owner) REFERENCES users (id) ON DELETE CASCADE
);
CREATE VIRTUAL TABLE repo_search USING fts5 (
  repo_id, keywords, tokenize = 'porter'
);
-- Doing this to make sure, no duplicate repo_id is added.
-- I will be using this query:
-- INSERT OR REPLACE INTO repo_search (repo_id, keywords) VALUES (?, ?)
CREATE TABLE repo_topics (
  repo_id VARCHAR(150) NOT NULL,
  -- Limit is 50, hence
  topic VARCHAR(60) NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE,
  UNIQUE (repo_id, topic)
);
CREATE TABLE repo_dependents (
  repo_id VARCHAR(150) NOT NULL,
  dependent VARCHAR(260) NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE,
  UNIQUE (repo_id, dependent)
);
CREATE TABLE releases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id VARCHAR(150) NOT NULL,
  version VARCHAR(255),
  is_prerelease BOOLEAN NOT NULL,
  published_at TIMESTAMP NOT NULL,
  -- Making this only 30 characters limit, it shouldn't be more than that.
  minimum_zig_version VARCHAR(30) NOT NULL,
  readme_url TEXT NOT NULL,
  directory_files TEXT NOT NULL DEFAULT '',
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE,
  UNIQUE (repo_id, version)
);
CREATE TABLE release_dependencies (
  release_id INTEGER NOT NULL,
  name VARCHAR(260) NOT NULL,
  hash VARCHAR(260) NOT NULL,
  is_lazy BOOLEAN NOT NULL CHECK (
    is_lazy IN (0, 1)
  ),
  url VARCHAR(260) NOT NULL,
  path VARCHAR(260) NOT NULL,
  FOREIGN KEY (release_id) REFERENCES releases (id) ON DELETE CASCADE,
  UNIQUE (release_id, name)
);
CREATE TABLE index_sections (
  section_name VARCHAR(10) NOT NULL,
  repo_id VARCHAR(150) NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE,
  UNIQUE (section_name, repo_id)
);
CREATE TABLE packages (
  repo_id VARCHAR(150) NOT NULL PRIMARY KEY,
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE
);
CREATE TABLE programs (
  repo_id VARCHAR(150) NOT NULL PRIMARY KEY,
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE
);
CREATE TABLE index_new_repo (
  id VARCHAR(150) PRIMARY KEY,
  type_of_repo VARCHAR(10) NOT NULL -- This is like, package or program.
);

CREATE TABLE safe_to_index_new_repo (
  id VARCHAR(150) PRIMARY KEY,
  type_of_repo VARCHAR(10) NOT NULL -- This is like, package or program.
);
-- For this specifically, I will create a AI model, I will train it to detect
-- all the scam repos, if my AI algorithm flags a repo
-- to review it will be added to this table, if no problem, it will continue.
CREATE TABLE quarantined_repos (
  id VARCHAR(150) PRIMARY KEY,
  type_of_repo VARCHAR(10) NOT NULL
);
-- These are only the users who are either spamming or
-- shipping malware.
CREATE TABLE banned_user_list (
  id VARCHAR(45) PRIMARY KEY
);
-- This is for making it easier to fetch
-- details of these repositories
-- to improve the accuracy of the algorithm
-- which detects such repos.
CREATE TABLE banned_repo_list (
  id VARCHAR(150) PRIMARY KEY
);
-- Needs update
CREATE TABLE needs_updates (
  id VARCHAR(150) PRIMARY KEY,
  -- All the repos that need updates.
  type_of_repo VARCHAR(10) NOT NULL -- This is like, package or program.
  );
-- I do search for "All repos where username is username"
CREATE INDEX repo_owner_index ON repos (owner);
-- Again, I do this, specially for the top 10 latest repos
CREATE INDEX repo_created_at_index ON repos (created_at);
-- again, for repos with most stars
CREATE INDEX repo_star_count_index ON repos (stargazer_count);
-- Whenever someone visits a repo page, that page
-- needs "ALl the releases with of that repo."
CREATE INDEX releases_repo_index ON releases (repo_id, published_at DESC);
-- Also, for every release, I need to get
-- all the dependencies of that release
CREATE INDEX release_dependencies_release_id_index ON release_dependencies (release_id);
-- Ok, on every package I also need number of repo topics.
CREATE INDEX repo_topics_repo_id_index ON repo_topics (repo_id);
-- Ok, on every package I also need number of repo dependents.
CREATE INDEX repo_dependents_repo_id_index ON repo_dependents (repo_id);
