SELECT count(*) AS rows FROM bench_kv;
SELECT count(*) AS indexes FROM pg_indexes WHERE tablename = 'bench_kv';
SELECT 'k000001' < 'k100000' AS collation_agrees_with_bytes;
