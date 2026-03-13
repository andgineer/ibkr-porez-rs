#!/usr/bin/env bash
set -euo pipefail

LANGUAGES="en ru uk rs rs-cyr"
SITE_DIR="site"

rm -rf "$SITE_DIR"

for lang in $LANGUAGES; do
    echo "Building docs for: $lang"
    mdbook build "docs/$lang"
done

# Create root redirect to English
cat > "$SITE_DIR/index.html" << 'REDIRECT'
<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0; url=./en/"></head>
<body><a href="./en/">English documentation</a></body>
</html>
REDIRECT

echo "Documentation built in $SITE_DIR/"
