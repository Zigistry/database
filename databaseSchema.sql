CREATE TABLE repo (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL,
    issue_count INT NOT NULL,
    fork_count INT NOT NULL,
    stargazers_count INT NOT NULL,
    watchers_count INT NOT NULL,
    pushed_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    is_archived BOOLEAN NOT NULL,
    is_disabled BOOLEAN NOT NULL,
    is_fork BOOLEAN NOT NULL,
    license TEXT NOT NULL,
)
CREATE TABLE 
