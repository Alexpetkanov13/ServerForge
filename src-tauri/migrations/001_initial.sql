-- ServerForge metadata schema. Server files live on disk, never in SQLite.

CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    path TEXT NOT NULL UNIQUE,
    provider TEXT NOT NULL,
    minecraft_version TEXT NOT NULL,
    loader_version TEXT,
    build TEXT,
    java_path TEXT,
    memory_min_mb INTEGER NOT NULL DEFAULT 1024,
    memory_max_mb INTEGER NOT NULL DEFAULT 4096,
    port INTEGER NOT NULL DEFAULT 25565,
    status TEXT NOT NULL DEFAULT 'offline',
    auto_start INTEGER NOT NULL DEFAULT 0,
    eula_accepted INTEGER NOT NULL DEFAULT 0,
    motd TEXT,
    pid INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_started_at TEXT
);

CREATE TABLE IF NOT EXISTS server_settings (
    server_id TEXT PRIMARY KEY NOT NULL,
    jvm_args TEXT,
    optimized_flags INTEGER NOT NULL DEFAULT 1,
    backup_retention INTEGER NOT NULL DEFAULT 10,
    auto_backup INTEGER NOT NULL DEFAULT 0,
    backup_interval_minutes INTEGER,
    start_after_install INTEGER NOT NULL DEFAULT 0,
    extra_json TEXT,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS java_runtimes (
    id TEXT PRIMARY KEY NOT NULL,
    path TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    major INTEGER NOT NULL,
    vendor TEXT,
    architecture TEXT,
    is_managed INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS backups (
    id TEXT PRIMARY KEY NOT NULL,
    server_id TEXT NOT NULL,
    label TEXT,
    path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    trigger TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS downloads (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    label TEXT NOT NULL,
    url TEXT NOT NULL,
    dest_path TEXT NOT NULL,
    status TEXT NOT NULL,
    bytes_downloaded INTEGER NOT NULL DEFAULT 0,
    bytes_total INTEGER,
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS installation_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    server_id TEXT,
    payload_json TEXT NOT NULL,
    current_step TEXT,
    completed_steps TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL,
    error TEXT,
    logs TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS activity_logs (
    id TEXT PRIMARY KEY NOT NULL,
    server_id TEXT,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    technical TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS server_templates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS favorites (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    ref_id TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS plugins (
    id TEXT PRIMARY KEY NOT NULL,
    server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    version TEXT,
    file_name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    source TEXT,
    source_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS mods (
    id TEXT PRIMARY KEY NOT NULL,
    server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    version TEXT,
    file_name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    source TEXT,
    source_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);
