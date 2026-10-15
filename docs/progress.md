# SDD ledger — plan: docs/superpowers/plans/2026-10-15-kimi-switcher.md

Base: b8571ce. Execution: Native, approved by user.

Pre-flight: tasks 1→2→3→4 share Credential/AppError; tasks 4→5→6 share snapshot and login commands. No conflicting names.

Ruling: Work in the explicitly designated kimi-switcher subdirectory on codex/kimi-switcher — preserve visibility in the user's workspace and all unrelated staged files; no external worktree — cost if wrong: checkout shares the parent repository.

Ruling: Windows CLI has no OAuth cross-process lock; block all detected Kimi CLI writers, exclusively protect credential files, recheck before commit, and document concurrent-start limitation — evidence in installed OAuthManager.resolveLockTarget — cost if wrong: a concurrently started CLI may require re-login.

Task 1: in progress; tests written before adapter implementation.

Task 1: complete — adapter 5/5, frontend build and cargo check passed. Task 2: complete — repository 6/6 and combined suite 11/11. Ruling: test temp directories are inside project; host short-name TEMP path makes OS rename fail with error 6 even in .NET, while project paths work — cost: tests need writable project directory.

Task 3: complete — 7 quota/OAuth cases passed, combined suite 18/18. Ruling: persist rotated credentials before requesting quota — losing the usage response must not discard a rotated refresh token — cost if wrong: additional disk transaction per refresh.

Task 4: switch tests 5/5 passed; failure injection restores both files, startup restores interrupted transaction. Task 5: login argument and authorization URL tests written, RED observed.
