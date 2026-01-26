-- Also, I have added avatar_id
-- to both repos and userm even though they are the same
-- Just for normalization
-- I don't want, sql to process 2 queries on every request,
-- too much latency. To cut down on latency, avatar_id will
-- be duplicated for both repos and users.
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
  -- I just realised that the boolean type is like an integer only.
  -- I added check that it should only be either true or false. i.e 0, 1
  is_archived BOOLEAN NOT NULL CHECK (is_archived IN (0,1)),
  is_disabled BOOLEAN NOT NULL CHECK (is_disabled IN (0,1)),
  is_fork BOOLEAN NOT NULL CHECK (is_fork IN (0,1)),
  minimum_zig_version TEXT NOT NULL, -- Again, this I have added a join to make sure
  -- I don't have to read the default branch information just to get this info.
  -- Simple google tells, its just 36 characters, so, ok?
  license VARCHAR(40) NOT NULL,
  -- Maybe 50 is perfect here
  primary_language VARCHAR(50) NOT NULL,
  FOREIGN KEY (owner) REFERENCES users (id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE repo_search USING fts5 (
  repo_id,
  keywords,
  content = '',
  tokenize = 'porter'
);

-- Doing this to make sure, no duplicate repo_id is added.
CREATE TRIGGER repo_search_unique_insert BEFORE INSERT ON repo_search BEGIN
DELETE FROM repo_search
WHERE
  repo_id = NEW.repo_id;

END;

CREATE TRIGGER repo_delete_cleanup
AFTER DELETE ON repos
BEGIN
  DELETE FROM repo_search
  WHERE repo_id = OLD.id;
END;

CREATE TABLE repo_topics (
  repo_id VARCHAR(150) NOT NULL,
  -- Limit is 50, hence
  topic VARCHAR(60) NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE,
  UNIQUE (repo_id, topic)
);

CREATE TABLE repo_dependents (
  repo_id VARCHAR(150) NOT NULL,
  dependent TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE,
  UNIQUE (repo_id, dependent)
);

CREATE TABLE releases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  repo_id VARCHAR(150) NOT NULL,
  version VARCHAR(255),
  is_prerelease BOOLEAN NOT NULL,
  published_at TIMESTAMP NOT NULL,
  minimum_zig_version TEXT NOT NULL,
  readme_url TEXT NOT NULL,
  FOREIGN KEY (repo_id) REFERENCES repos (id) ON DELETE CASCADE,
  UNIQUE (repo_id, version)
);

CREATE TABLE release_dependencies (
  release_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  hash TEXT NOT NULL,
  lazy TEXT NOT NULL,
  url TEXT NOT NULL,
  path TEXT NOT NULL,
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

-- I do search for "All packages where username is username", and also
-- for  "All programs where username is username"
CREATE INDEX packages_repo_id_index ON packages (repo_id);
CREATE INDEX programs_repo_id_index ON programs (repo_id);
