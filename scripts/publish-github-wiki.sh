#!/bin/sh
set -eu

repository=${1:-mikeortman/wrf-rs}
repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
wiki_checkout=$(mktemp -d "${TMPDIR:-/tmp}/wrf-rs-wiki.XXXXXX")
trap 'rm -rf "$wiki_checkout"' EXIT HUP INT TERM

git clone "https://github.com/${repository}.wiki.git" "$wiki_checkout"
cp "$repository_root/docs/wiki/README.md" "$wiki_checkout/Home.md"

for source_page in "$repository_root"/docs/wiki/*.md; do
    page_name=$(basename "$source_page")
    if [ "$page_name" != 'README.md' ]; then
        cp "$source_page" "$wiki_checkout/$page_name"
    fi
done

git -C "$wiki_checkout" add --all
if git -C "$wiki_checkout" diff --cached --quiet; then
    echo "GitHub Wiki already matches docs/wiki"
    exit 0
fi

git -C "$wiki_checkout" commit -m "Sync wiki from main repository"
git -C "$wiki_checkout" push origin HEAD:master
echo "Published docs/wiki to https://github.com/${repository}/wiki"
