# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Version Control

This repository uses **Jujutsu (jj)** as the primary VCS with Git as the backend. Prefer `jj` commands over `git` for version control operations.

Key jj commands:
- `jj log` — view history
- `jj status` — working copy status
- `jj diff` — see changes
- `jj commit -m "message"` — commit changes
- `jj new` — create a new change
- `jj describe -m "message"` — update the current change description
