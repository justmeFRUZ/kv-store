INSERT INTO bench_kv (key, value) SELECT 'k' || lpad(n::text, 6, '0'), 'v' || n::text FROM generate_series(1, 100000) AS n;
