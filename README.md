# Usage

run `cargo run -- 'path/to/config/file.toml'`

## Refactoring plans

- add error handling
- add multithread processing for repos

## artifact creation command

git fetch && git reset --hard origin/dev-510 && npm version 10.0.0 && git add . && git commit -m "@10.0.0 release" && git tag -a "release/10.0.0" -m "Added '@10.0.0 release' version" && git push -f && git push --tags
