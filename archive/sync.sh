#!/usr/bin/env bash


REMOTE_URL=git@github.com:raiment-studios/sea-git-publish.git

if [ ! -f .git-sync-snapshot.tar.gz ]; then
    echo "No snapshot found, creating initial clone..."
    mkdir -p git-remote
    pushd git-remote
    git clone $REMOTE_URL .
    tar -czf ../.git-sync-snapshot.tar.gz .git
    popd
    rm -rf git-remote
fi

echo "Syncing changes to remote repository..."
mkdir -p .git
tar -xzf .git-sync-snapshot.tar.gz -C .git --strip-components=1
ls -la
sleep 2
git add .
git commit -m "Sync changes"
git pull $REMOTE_URL main --rebase
git push $REMOTE_URL main
if [ $? -eq 0 ]; then
    tar -czf .git-sync-snapshot.tar.gz .git
fi
rm -rf .git

