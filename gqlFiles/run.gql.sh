curl -H "Authorization: bearer " \
     -H "Content-Type: application/json" \
     -X POST \
     -d '{"query": "'"$(cat ./main.gql)"'", "variables": '"$(cat ./request.json)"'}' \
     https://api.github.com/graphql
