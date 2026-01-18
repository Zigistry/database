cargo fmt
taplo fmt
npx prettier --write "**/*.{md,yaml,yml,gql}"
npx sql-formatter -l sqlite --fix ./Database_SQL_Files/database_schema.sql
npx sql-formatter -l sqlite --fix ./Database_SQL_Files/dependents_calculator.sql
zig fmt ./src/search_preparer.zig