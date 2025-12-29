#!/bin/bash
# Approve all pending workflow runs

cd repos/syster || exit 1

echo "🔍 Finding workflow runs waiting for approval..."

# Get all workflow runs waiting for approval and approve them
gh run list --json databaseId,status,name --jq '.[] | select(.status=="waiting") | .databaseId' | while read -r run_id; do
    echo "✅ Approving run $run_id..."
    gh run approve "$run_id"
done

echo "✨ Done!"
