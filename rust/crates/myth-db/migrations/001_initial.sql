PRAGMA user_version = 1;

-- =============================
-- Lessons (core table)
-- =============================
CREATE TABLE lessons (
    id BLOB PRIMARY KEY,                    -- UUID v4 bytes (16)

    -- Identity
    identity_hash_tier1 BLOB NOT NULL,      -- SHA1 20 bytes

    -- Classification
    level INTEGER NOT NULL CHECK(level BETWEEN 1 AND 5),
    category TEXT NOT NULL CHECK(category IN
        ('security', 'correctness', 'process', 'data_safety', 'temporal')),

    -- Recurrence
    recurrence_count REAL NOT NULL DEFAULT 0,
    missed_hook_count INTEGER NOT NULL DEFAULT 0,

    -- Timestamps (unix seconds)
    first_seen INTEGER NOT NULL,
    last_seen INTEGER NOT NULL,

    -- Lapse
    lapse_score REAL NOT NULL DEFAULT 0,

    -- Appeals
    appeals INTEGER NOT NULL DEFAULT 0,

    -- Status
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN
        ('active', 'lapsed', 'archived', 'superseded')),

    -- Content
    description TEXT NOT NULL,
    rationale TEXT NOT NULL,

    -- Metadata (arbitrary JSON)
    meta_json TEXT
);

CREATE INDEX idx_lessons_identity ON lessons(identity_hash_tier1);
CREATE INDEX idx_lessons_level ON lessons(level);
CREATE INDEX idx_lessons_last_seen ON lessons(last_seen DESC);
CREATE INDEX idx_lessons_status ON lessons(status);
CREATE INDEX idx_lessons_category ON lessons(category);

-- =============================
-- Vector metadata (maps lessons → vectors.bin rows)
-- =============================
CREATE TABLE vector_metadata (
    lesson_id BLOB PRIMARY KEY,
    row_index INTEGER NOT NULL,
    generation INTEGER NOT NULL,
    created_ts INTEGER NOT NULL,
    FOREIGN KEY(lesson_id) REFERENCES lessons(id) ON DELETE CASCADE
);

CREATE INDEX idx_vec_generation ON vector_metadata(generation);

-- Generation counter (single row)
CREATE TABLE vector_generation (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    current_generation INTEGER NOT NULL,
    last_updated INTEGER NOT NULL
);
INSERT INTO vector_generation (id, current_generation, last_updated)
    VALUES (1, 0, strftime('%s', 'now'));

-- =============================
-- Hook events (latency + execution history)
-- =============================
CREATE TABLE hook_events (
    id BLOB PRIMARY KEY,
    session_id BLOB NOT NULL,
    event_type TEXT NOT NULL CHECK(event_type IN (
        'session_start', 'user_prompt',
        'pre_tool', 'post_tool', 'post_tool_failure',
        'stop'
    )),
    tool_name TEXT,
    ts INTEGER NOT NULL,                    -- unix ms
    latency_ms REAL NOT NULL,
    verdict TEXT NOT NULL CHECK(verdict IN ('allow', 'deny', 'ask')),
    lesson_id BLOB,
    FOREIGN KEY(lesson_id) REFERENCES lessons(id)
);

CREATE INDEX idx_events_session ON hook_events(session_id);
CREATE INDEX idx_events_ts ON hook_events(ts DESC);
CREATE INDEX idx_events_type ON hook_events(event_type, ts DESC);

-- =============================
-- Appeals
-- =============================
CREATE TABLE appeal_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lesson_id BLOB NOT NULL,
    appeal_type TEXT NOT NULL CHECK(appeal_type IN ('appeal', 'retrial')),
    ts INTEGER NOT NULL,
    result TEXT NOT NULL CHECK(result IN ('pending', 'granted', 'denied', 'withdrawn')),
    rationale TEXT,
    resolved_ts INTEGER,
    resolver TEXT,
    FOREIGN KEY(lesson_id) REFERENCES lessons(id)
);

CREATE INDEX idx_appeals_lesson ON appeal_history(lesson_id);
CREATE INDEX idx_appeals_status ON appeal_history(result);

-- =============================
-- Grid overrides (Level × Recurrence → Enforcement, admin or observer-suggested)
-- =============================
CREATE TABLE grid_overrides (
    level INTEGER NOT NULL CHECK(level BETWEEN 1 AND 5),
    recurrence INTEGER NOT NULL CHECK(recurrence BETWEEN 1 AND 6),
    enforcement TEXT NOT NULL CHECK(enforcement IN
        ('dismiss', 'note', 'advisory', 'caution', 'warn', 'strike', 'seal')),
    source TEXT NOT NULL CHECK(source IN ('default', 'admin', 'observer_suggested')),
    approved_ts INTEGER,
    rationale TEXT,
    PRIMARY KEY(level, recurrence)
);

-- =============================
-- Sessions (observation only)
-- =============================
CREATE TABLE sessions (
    id BLOB PRIMARY KEY,
    started_ts INTEGER NOT NULL,
    ended_ts INTEGER,
    project_path TEXT,
    event_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_sessions_started ON sessions(started_ts DESC);
