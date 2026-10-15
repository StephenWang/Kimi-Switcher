# SDD ledger — plan: docs/superpowers/plans/2026-10-15-kimi-switcher.md

Base: b8571ce. Execution: Native, approved by user.

Pre-flight: tasks 1→2→3→4 share Credential/AppError; tasks 4→5→6 share snapshot and login commands. No conflicting names.

Ruling: Work in the explicitly designated kimi-switcher subdirectory on codex/kimi-switcher — preserve visibility in the user's workspace and all unrelated staged files; no external worktree — cost if wrong: checkout shares the parent repository.

Ruling: Windows CLI has no OAuth cross-process lock; block all detected Kimi CLI writers, exclusively protect credential files, recheck before commit, and document concurrent-start limitation — evidence in installed OAuthManager.resolveLockTarget — cost if wrong: a concurrently started CLI may require re-login.

Task 1: in progress; tests written before adapter implementation.

Task 1: complete — adapter 5/5, frontend build and cargo check passed. Task 2: complete — repository 6/6 and combined suite 11/11. Ruling: test temp directories are inside project; host short-name TEMP path makes OS rename fail with error 6 even in .NET, while project paths work — cost: tests need writable project directory.

Task 3: complete — 7 quota/OAuth cases passed, combined suite 18/18. Ruling: persist rotated credentials before requesting quota — losing the usage response must not discard a rotated refresh token — cost if wrong: additional disk transaction per refresh.

Task 4: switch tests 5/5 passed; failure injection restores both files, startup restores interrupted transaction. Task 5: login argument and authorization URL tests written, RED observed.

Tasks 4–6: implemented. Rust suite 32/32, UI suite 5/5; desktop cargo check passed. Added rotated-token roll-forward journal and per-region account deduplication. Ruling: refresh-all uses concurrency 1 (within maximum 3) to serialize token writes — cost: slower refresh across many accounts. Initial NSIS download timeout resolved by fetching official tool archive and plugin with independently verified upstream SHA1 hashes. Final independent review pending.

Final: fixed missing providers panic — fresh_cli_directory_has_no_active_account RED→GREEN. Final: fixed stale refresh recovery — refresh_recovery_preserves_newer_external_token_for_same_account RED→GREEN. Final: fixed external changes lost on switch recovery — switch_recovery_preserves_external_changes_and_keeps_journal and switch_recovery_checks_all_files_before_restoring_any RED→GREEN. Final: regraded bulk-refresh false success to Important and fixed — bulk_refresh_cannot_report_success_when_some_requests_failed RED→GREEN. Suite: Rust 37/37; UI 5/5. Final review: independent reviewer completed; no deferred minors. Real authorization/live quota and installer execution remain explicitly unverified; visual QA passed and executable startup smoke passed. Windows simultaneous-start exclusion remains the documented limitation.
