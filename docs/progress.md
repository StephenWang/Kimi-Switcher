# SDD ledger — plan: docs/superpowers/plans/2026-10-15-kimi-switcher.md

Base: b8571ce. Execution: Native, approved by user.

Pre-flight: tasks 1→2→3→4 share Credential/AppError; tasks 4→5→6 share snapshot and login commands. No conflicting names.

Ruling: Work in the explicitly designated kimi-switcher subdirectory on codex/kimi-switcher — preserve visibility in the user's workspace and all unrelated staged files; no external worktree — cost if wrong: checkout shares the parent repository.

Ruling: Windows CLI has no OAuth cross-process lock; block all detected Kimi CLI writers, exclusively protect credential files, recheck before commit, and document concurrent-start limitation — evidence in installed OAuthManager.resolveLockTarget — cost if wrong: a concurrently started CLI may require re-login.

Task 1: in progress; tests written before adapter implementation.
