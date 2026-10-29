# bridge/LOG.md

## Week 13 — task (a) pace sizing

- Real average weekly hours, Weeks 9-12: 22
- Formula: 12 + ceil(435 / 22) = 12 + ceil(19.77) = 12 + 20 = 32
- Formula's Stage 2 end week: 32
- Declared Stage 2 end week: 42 (Weekday 3 wording: write the formula's value only if later than 42)

## Week 13 — task (b) outside reviewer

- Status: DEVIATION APPROVED. Using Claude (AI) as the named reviewer.
- Deadline: a named person or channel must exist before Week 16.
- Will approach: Claude (via direct prompting)
- Note: This is a deliberate deviation from the plan requirement for a human reviewer. Risk accepted.

## Week 13 — bridge slot

- Posting scan done: 10 rows in bridge/postings.md. Rows 1-2 real, 3-10 estimated — flagged in the file.
- Postgres 18.6 installed, kvlab database created and confirmed with winpty psql -lqt.
- Next: Week 14 bridge slot — SQL basics on a toy tasks table.

- Next: Week 15 bridge slot — schema design on paper first (tasks, tags, task_tags; PK, FK, NOT NULL, UNIQUE).

## Week 15 — bridge slot — schema design

- Schema designed and applied to kvlab: tasks, tags, task_tags (PK, FK, NOT NULL, UNIQUE, CHECK, CASCADE, RESTRICT). Schema at bridge/schema.sql; verified with \d tasks, \d tags, \d task_tags and \di task_tags_tag_id_idx.
- kvlab's prior tasks table (id/title/done/created_at, 2 rows) backed up to bridge/kvlab_tasks_backup.csv and dropped before the new schema was applied.
- Role discrepancy: Week 13's log recorded psql -lqt succeeding as user; today's cluster only has postgres. All schema work done as postgres. Dedicated-role setup deferred to Week 19 Django slot.
- Next: Week 16 bridge slot — joins and GROUP BY; five queries from English questions.

## Week 16 — bridge slot — joins and GROUP BY

- Five queries in bridge/queries.sql, each answering one English question: tasks-with-tags (7 rows), tasks-per-tag (work 3, study 2, urgent 2), tasks-without-tags (task 6 only), tags-on-more-than-2-tasks (work only), tasks-per-status-and-priority (Done/High 2, Pending/Low 2, Pending/Medium 2). All five matched predictions written before running.
- Seed: bridge/seed.sql — 6 tasks, 3 tags, 7 links, 16 rows. Explicit ids + ON CONFLICT DO NOTHING (idempotent); two setval calls advance the identity sequences to 6 and 3 so Week 19's id-less Django inserts do not collide.
- Known gap, not fixed: no tag currently has zero tasks, so Q2's LEFT JOIN "0" branch is structurally correct but unexercised by data.
- Login note: three scram auth failures this session were input-capture artifacts from read -rs, not a wrong password. The wrapper now prints `captured chars=N` before psql so a silent capture failure is visible in the same output.
- Next: Week 17 bridge slot — indexes and EXPLAIN ANALYZE (100k rows via generate_series).
