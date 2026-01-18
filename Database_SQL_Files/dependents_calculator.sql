INSERT INTO
  repo_dependents (repo_id, dependent)
SELECT DISTINCT
  lower(
    concat (
      CASE
        WHEN url LIKE '%github.com/%' THEN 'gh/'
        ELSE 'cb/'
      END,
      rtrim(substr(url, instr(url, '.com/') + 5), '/')
    )
  ) AS repo_id_of_dependency,
  repo_id
FROM
  release_dependencies
WHERE
  (
    url LIKE '%github.com/%'
    OR url LIKE '%codeberg.org/%'
  )
  AND repo_id_of_dependency IN (
    SELECT
      id
    FROM
      repos
  );
