#!/bin/bash
# Script to update TFHE library while preserving custom modifications

set -e  # Exit on error

echo "=========================================="
echo "TFHE Library Update Script"
echo "=========================================="

# Check current status
echo ""
echo "1. Checking current branch and modifications..."
git status
echo ""

# Show your current modifications
echo "2. Your custom modifications:"
git log --oneline main ^origin/main
echo ""
git diff origin/main --stat
echo ""

# Save a patch of your modifications
echo "3. Creating backup patch of your modifications..."
git diff origin/main > /tmp/my_tfhe_modifications_$(date +%Y%m%d_%H%M%S).patch
echo "   Backup saved to: /tmp/my_tfhe_modifications_$(date +%Y%m%d_%H%M%S).patch"
echo ""

# Fetch updates
echo "4. Fetching latest updates from upstream..."
git fetch origin
echo ""

# Show how many new commits
NEW_COMMITS=$(git rev-list --count main..origin/main)
echo "5. Number of new commits available: $NEW_COMMITS"
if [ $NEW_COMMITS -eq 0 ]; then
    echo "   Already up to date!"
    exit 0
fi
echo ""

# Show first 10 new commits
echo "6. Preview of new commits:"
git log --oneline main..origin/main | head -10
echo ""

# Ask for confirmation
read -p "Do you want to proceed with the update? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Update cancelled."
    exit 1
fi

# Create a backup branch with timestamp
BACKUP_BRANCH="backup-before-update-$(date +%Y%m%d_%H%M%S)"
echo "7. Creating backup branch: $BACKUP_BRANCH"
git branch $BACKUP_BRANCH
echo ""

# Create update branch
UPDATE_BRANCH="update-$(date +%Y%m%d_%H%M%S)"
echo "8. Creating update branch: $UPDATE_BRANCH"
git checkout -b $UPDATE_BRANCH origin/main
echo ""

# Cherry-pick your modifications
echo "9. Applying your modifications on top of latest code..."
if git cherry-pick main; then
    echo "   SUCCESS: Modifications applied cleanly!"
else
    echo ""
    echo "   ⚠️  CONFLICTS DETECTED!"
    echo "   Please resolve conflicts manually, then run:"
    echo "   1. git add <resolved-files>"
    echo "   2. git cherry-pick --continue"
    echo "   3. git checkout main"
    echo "   4. git reset --hard $UPDATE_BRANCH"
    exit 1
fi
echo ""

# Update main branch
echo "10. Updating main branch to the new version..."
git checkout main
git reset --hard $UPDATE_BRANCH
echo ""

echo "=========================================="
echo "✅ UPDATE COMPLETE!"
echo "=========================================="
echo ""
echo "Summary:"
echo "- Old version backed up to: $BACKUP_BRANCH"
echo "- Main branch updated with your modifications"
echo "- Your modifications preserved:"
git log --oneline main ^origin/main
echo ""
echo "To test: cd ../HDM_rs && cargo build --release"
echo ""
