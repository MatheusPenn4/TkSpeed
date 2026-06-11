-- PL-2d · Confidence Engine: qualidade/térmica por sessão de benchmark.
ALTER TABLE benchmark_sessions ADD COLUMN confidence INTEGER NOT NULL DEFAULT 0;
ALTER TABLE benchmark_sessions ADD COLUMN contaminated INTEGER NOT NULL DEFAULT 0;
ALTER TABLE benchmark_sessions ADD COLUMN temp_start REAL;
ALTER TABLE benchmark_sessions ADD COLUMN temp_end REAL;
