#!/bin/bash
# Non-blocking browser opener
if [ -n "$1" ]; then
    nohup "$BROWSER" "$1" >/dev/null 2>&1 &
    disown
    echo "📊 Dashboard opened in browser (background)"
fi
