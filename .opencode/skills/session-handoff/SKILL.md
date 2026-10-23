---
name: session-handoff
description: Maintain continuity across OpenCode sessions by reading and updating PROJECT_STATE, SESSION_LOG, and DECISIONS. Use at the start of substantive work and when finishing or stopping a meaningful task.
---

# Skill: session-handoff

Use this to keep the project's memory intact across sessions. The goal is that
the next agent can continue without re-deriving context, and that nothing
unfinished is ever presented as done.

## At the start of substantive work

1. **Read `docs/ai/PROJECT_STATE.md`.** It is the one-screen snapshot: objective,
   verified functionality, decisions, limitations, next task, verification
   commands.
2. Read the relevant sections of:
   - `docs/ai/SESSION_LOG.md` (what was actually done, and what failed),
   - `docs/ai/DECISIONS.md` (why the architecture is the way it is),
   - `docs/research/hexagonal-grid.md` (for any mathematical work).
   Read only what the task needs — do not re-read everything every time.
3. **Check `git status`** (and the branch) before editing. Note pre-existing
   modifications and untracked files, and do not disturb them.
4. Skim the actual source files you intend to change. Never assume a file,
   type, or command exists because it would be convenient.

## While working

- Keep a mental (or scratch) record of the exact commands you run and their
  real outcomes, so the session log is factual.
- If you make a **significant architectural or mathematical decision**,
  prepare a decision record for it.
- If you find an unresolved question, note it rather than silently choosing an
  arbitrary answer.

## When a meaningful task is completed (or blocked)

1. **Append one concise entry to `docs/ai/SESSION_LOG.md`.** Never rewrite or
   reorder history; append only, and do not duplicate previous entries. Include:
   - date, objective;
   - files changed (created vs. updated);
   - the changes made (code vs. documentation — keep them distinct);
   - commands/tests executed;
   - results **including failures**;
   - findings;
   - remaining work.
2. **Update `docs/ai/PROJECT_STATE.md`** so it still fits on one screen:
   current objective, verified functionality, decisions, known limitations,
   the next concrete task, and the verification commands.
3. **Add a decision record** to `docs/ai/DECISIONS.md` if a significant
   decision was made.
4. **Update the next-step recommendation** when the current task is finished or
   blocked, so the next session has a clear entry point.

## Rules

- **Never mark incomplete tasks as complete.** If something is half done, say
  what is done and what is not.
- **Never report a test as passing unless it was actually executed and passed.**
  Record the exact command and its real result, including errors.
- **Do not rewrite the entire history or duplicate entries.**
- Record only tasks that were actually performed. Do not log chores you merely
  thought about.
- **Leave a useful handoff before ending a substantial session**, including when
  work stops unexpectedly (context limit, interruption, failure). A short,
  honest entry beats a stale state file.
- If you changed no files and nothing meaningful happened, do not create
  documentation — these documents are for real, durable progress, not for
  trivial conversational responses.
- Keep `PROJECT_STATE.md` **short**. If it grows, move detail into
  `docs/architecture.md` or the research spec and link to it.
