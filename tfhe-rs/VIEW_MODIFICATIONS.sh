#!/bin/bash
# Quick commands to view your TFHE modifications

echo "======================================"
echo "Your Custom TFHE Modifications"
echo "======================================"
echo ""

echo "Modified commits:"
git log --oneline main ^origin/main
echo ""

echo "Modified files:"
git diff origin/main --stat
echo ""

echo "Detailed changes:"
echo "----------------"
git diff origin/main
echo ""

echo "======================================"
echo "To save modifications as patch:"
echo "  git diff origin/main > my_mods.patch"
echo ""
echo "To see this again, run:"
echo "  ./VIEW_MODIFICATIONS.sh"
echo "======================================"
