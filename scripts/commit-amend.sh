#!/usr/bin/env sh
set -eu

branch=$(git branch --show-current)
if [ -z "$branch" ]; then
    echo "Not on a branch; refusing to amend and force-push."
    exit 1
fi

echo "Current branch: $branch"
echo "Amending commit: $(git log -1 --oneline)"
echo "Current changes:"
git status --short

attempt=1
max_attempts=3

while [ "$attempt" -le "$max_attempts" ]; do
    echo "Staging changes..."
    git add .

    echo "Running git commit --amend --no-edit (attempt $attempt/$max_attempts)..."
    if git commit --amend --no-edit; then
        if [ -z "$(git status --short)" ]; then
            break
        fi

        echo "Commit succeeded, but hooks modified files. Re-amending..."
    else
        if [ -z "$(git status --short)" ] || [ "$attempt" -eq "$max_attempts" ]; then
            echo "Commit amend failed. Fix the reported error and rerun make commit-amend."
            exit 1
        fi

        echo "Commit amend failed and files changed, likely from lint/pre-commit hooks. Re-staging and retrying..."
    fi

    attempt=$((attempt + 1))
done

if [ "$attempt" -gt "$max_attempts" ]; then
    echo "Commit amend did not stabilize after $max_attempts attempts."
    exit 1
fi

echo "Pushing amended commit with --force-with-lease..."
git push --force-with-lease
