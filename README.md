[![Large file guard](https://github.com/MrCiocchetti/0xv7/actions/workflows/large-file-guard.yml/badge.svg)](https://github.com/MrCiocchetti/0xv7/actions/workflows/large-file-guard.yml)

## Local Git hooks
After cloning, enable repo hooks:
bash scripts/setup-git-hooks.sh

The pre-commit hook blocks files larger than 50MB. CI enforces the same rule on PRs.