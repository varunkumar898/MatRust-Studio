#!/usr/bin/env bash
# Storage & Artifact Footprint Audit Report (Bash)
# Enforces budgets:
# - Tracked repo size <= 50 MB
# - Largest tracked file <= 500 KB
# - Release binary <= 13 MB
# - Shared target directory <= 20 GB
# - Free disk space >= 20 GB

set -euo pipefail

EXIT_CODE=0
echo "=================================================="
echo "      MATRUST STUDIO STORAGE AUDIT REPORT        "
echo "=================================================="

# 1. Tracked Files Size & Top 15
echo -e "\n--- 1. Tracked Repository Files ---"
TRACKED_TOTAL_BYTES=0
TRACKED_COUNT=0

# Temporary file to store file sizes
TMP_SIZES=$(mktemp)
trap 'rm -f "$TMP_SIZES"' EXIT

while IFS= read -r file; do
    if [ -f "$file" ]; then
        size=$(wc -c < "$file")
        TRACKED_TOTAL_BYTES=$((TRACKED_TOTAL_BYTES + size))
        TRACKED_COUNT=$((TRACKED_COUNT + 1))
        echo "$size $file" >> "$TMP_SIZES"
    fi
done < <(git ls-files)

TRACKED_TOTAL_MB=$(awk "BEGIN {printf \"%.2f\", $TRACKED_TOTAL_BYTES / 1048576}")
echo "Total Tracked Files: $TRACKED_COUNT"
echo "Total Tracked Size:  ${TRACKED_TOTAL_MB} MB ($TRACKED_TOTAL_BYTES bytes) [Budget: <= 50 MB]"

if [ "$TRACKED_TOTAL_BYTES" -gt 52428800 ]; then
    echo "[ERROR] Tracked repository size exceeds 50 MB budget!"
    EXIT_CODE=1
else
    echo "[OK] Tracked repository size within budget."
fi

echo -e "\nTop 15 Largest Tracked Files:"
sort -nr -k1,1 "$TMP_SIZES" | head -n 15 | while read -r size file; do
    kb=$(awk "BEGIN {printf \"%.1f\", $size / 1024}")
    printf "  %8s KB  %s\n" "$kb" "$file"
    if [ "$size" -gt 512000 ]; then
        echo "    [ERROR] File exceeds 500 KB threshold: $file"
        EXIT_CODE=1
    fi
done

# 2. Target Directory Size
echo -e "\n--- 2. Build Target Directory ---"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
if [ -d "$TARGET_DIR" ]; then
    TARGET_BYTES=$(du -sb "$TARGET_DIR" 2>/dev/null | awk '{print $1}' || du -sk "$TARGET_DIR" | awk '{print $1 * 1024}')
    TARGET_GB=$(awk "BEGIN {printf \"%.2f\", $TARGET_BYTES / 1073741824}")
    echo "Target Directory: $TARGET_DIR"
    echo "Target Total Size: ${TARGET_GB} GB ($TARGET_BYTES bytes) [Alarm threshold: <= 20 GB]"
    if [ "$TARGET_BYTES" -gt 21474836480 ]; then
        echo "[ALARM] Target directory exceeds 20 GB alarm threshold! Consider running 'cargo clean'."
    else
        echo "[OK] Target directory within alarm threshold."
    fi
else
    echo "Target Directory ($TARGET_DIR) does not exist (clean state)."
fi

# 3. Release Executable Size
echo -e "\n--- 3. Release Executable Size ---"
RELEASE_EXE="${TARGET_DIR}/release/matrust_studio"
if [ -f "${RELEASE_EXE}.exe" ]; then
    RELEASE_EXE="${RELEASE_EXE}.exe"
fi

if [ -f "$RELEASE_EXE" ]; then
    EXE_BYTES=$(wc -c < "$RELEASE_EXE")
    EXE_MB=$(awk "BEGIN {printf \"%.2f\", $EXE_BYTES / 1048576}")
    echo "Release Binary: $RELEASE_EXE"
    echo "Binary Size:    ${EXE_MB} MB ($EXE_BYTES bytes) [Budget: <= 13.0 MB]"
    if [ "$EXE_BYTES" -gt 13631488 ]; then
        echo "[ERROR] Release binary size exceeds budget!"
        EXIT_CODE=1
    else
        echo "[OK] Release binary within budget."
    fi
else
    echo "Release binary ($RELEASE_EXE) not found; run cargo build --release -p matrust_studio to measure."
fi

# 4. Git Worktrees
echo -e "\n--- 4. Active Git Worktrees ---"
git worktree list

# 5. Free Disk Space
echo -e "\n--- 5. Free Disk Space ---"
FREE_KB=$(df -k . | awk 'NR==2 {print $4}')
FREE_GB=$(awk "BEGIN {printf \"%.2f\", $FREE_KB / 1048576}")
echo "Free Disk Space: ${FREE_GB} GB [Threshold: >= 20 GB]"
if [ "$FREE_KB" -lt 20971520 ]; then
    echo "[ERROR] Free disk space is below 20 GB minimum!"
    EXIT_CODE=1
else
    echo "[OK] Free disk space meets minimum requirement."
fi

echo -e "\n=================================================="
if [ "$EXIT_CODE" -ne 0 ]; then
    echo "AUDIT RESULT: FAILED (Budget thresholds violated)"
    exit "$EXIT_CODE"
else
    echo "AUDIT RESULT: PASSED (All storage budgets respected)"
    exit 0
fi
