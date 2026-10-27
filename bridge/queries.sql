-- bridge/queries.sql
-- Week 16 bridge slot: five queries, each answering one English question
-- about the tasks/tags/task_tags schema. Read-only; safe to rerun.

-- Q1. Which tasks have which tags?
--     One row per (task, tag) pair; a task with several tags appears several times.

SELECT t.id, t.description, g.name AS tag
FROM tasks t
JOIN task_tags tt ON tt.task_id = t.id
JOIN tags g ON g.id = tt.tag_id
ORDER BY t.id, g.name;

-- Q2. How many tasks carry each tag?
--     LEFT JOIN so a tag with no tasks would still appear, with a count of 0.

SELECT g.name AS tag, COUNT(tt.task_id) AS task_count
FROM tags g
LEFT JOIN  task_tags tt ON  tt.tag_id = g.id
GROUP BY g.id, g.name
ORDER BY task_count DESC, tag;

-- Q3. Which tasks have no tags at all?
SELECT t.id, t.description
FROM tasks t
LEFT JOIN task_tags tt ON tt.task_id = t.id
WHERE tt.task_id IS NULL
ORDER BY t.id;

-- Q4. Which tags are attached to more than two tasks?
SELECT g.name AS tag, COUNT(*) as TASK_COUNT
FROM TAGS G
JOIN task_tags tt ON tt.tag_id = g.id
GROUP BY g.id, g.name
HAVING COUNT(*) > 2
ORDER BY task_count DESC;

-- Q5. How are tasks distributed across status and priority?
SELECT status, priority, COUNT(*) AS task_count
FROM tasks
GROUP BY status, priority
ORDER BY status, priority;