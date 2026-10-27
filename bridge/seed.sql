-- bridge/seed.sql
-- Small, fixed, idempotent seed for the Week 16 bridge slot (joins + GROUP BY).
-- Explicit ids + ON CONFLICT DO NOTHING: rerunning this file is safe.
-- The two setval calls advance each identity sequence past the seeded ids, so a
-- later INSERT that omits the id (Django, Week 19) does not collide with them.

BEGIN;

INSERT INTO tasks (id, description, status, priority) VALUES
    (1, 'write seed script', 'Done',    'High'),
    (2, 'read SQL book',    'Pending', 'Medium'),
    (3, 'fix mojibake',     'Pending', 'Low'),
    (4, 'review skip list', 'Done',    'High'),
    (5, 'call reviewer',    'Pending', 'Medium'),
    (6, 'plan Week 17',     'Pending', 'Low')
ON CONFLICT (id) DO NOTHING;

INSERT INTO tags (id, name) VALUES
    (1, 'work'),
    (2, 'study'),
    (3, 'urgent')
ON CONFLICT (id) DO NOTHING;

INSERT INTO task_tags (task_id, tag_id) VALUES
    (1, 1), (1, 3),
    (2, 2),
    (3, 1),
    (4, 1), (4, 3),
    (5, 2)
ON CONFLICT DO NOTHING;

SELECT setval(pg_get_serial_sequence('tasks', 'id'), (SELECT max(id) FROM tasks));
SELECT setval(pg_get_serial_sequence('tags', 'id'), (SELECT max(id) FROM tags));

COMMIT;