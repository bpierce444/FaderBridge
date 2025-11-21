-- FaderBridge Database Schema v1
-- This schema stores user projects, device configurations, and parameter mappings

-- Schema version tracking for migrations
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial version
INSERT OR IGNORE INTO schema_version (version) VALUES (1);

-- Projects table: stores named mapping configurations
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_opened_at TEXT,
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1))
);

-- Devices table: stores MIDI and UCNet device configurations
CREATE TABLE IF NOT EXISTS devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    device_type TEXT NOT NULL CHECK (device_type IN ('midi', 'ucnet')),
    device_name TEXT NOT NULL,
    device_id TEXT NOT NULL, -- MIDI port name or UCNet device ID
    connection_type TEXT CHECK (connection_type IN ('usb', 'network', 'virtual')),
    config_json TEXT, -- Device-specific configuration as JSON
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Index for faster device lookups by project
CREATE INDEX IF NOT EXISTS idx_devices_project ON devices(project_id);

-- Mappings table: stores parameter mappings between MIDI and UCNet
CREATE TABLE IF NOT EXISTS mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    midi_device_id INTEGER NOT NULL,
    ucnet_device_id INTEGER NOT NULL,
    
    -- MIDI source
    midi_channel INTEGER NOT NULL CHECK (midi_channel BETWEEN 0 AND 15),
    midi_cc INTEGER NOT NULL CHECK (midi_cc BETWEEN 0 AND 127),
    
    -- UCNet target
    ucnet_parameter_id INTEGER NOT NULL,
    ucnet_parameter_name TEXT NOT NULL,
    
    -- Mapping configuration
    taper_curve TEXT NOT NULL DEFAULT 'linear' CHECK (taper_curve IN ('linear', 'logarithmic', 'exponential', 's-curve')),
    min_value REAL NOT NULL DEFAULT 0.0,
    max_value REAL NOT NULL DEFAULT 1.0,
    invert INTEGER NOT NULL DEFAULT 0 CHECK (invert IN (0, 1)),
    
    -- Bidirectional sync
    bidirectional INTEGER NOT NULL DEFAULT 1 CHECK (bidirectional IN (0, 1)),
    
    -- Metadata
    label TEXT, -- User-friendly name for this mapping
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (midi_device_id) REFERENCES devices(id) ON DELETE CASCADE,
    FOREIGN KEY (ucnet_device_id) REFERENCES devices(id) ON DELETE CASCADE,
    
    -- Ensure unique MIDI CC per project/device
    UNIQUE (project_id, midi_device_id, midi_channel, midi_cc)
);

-- Indexes for faster mapping lookups
CREATE INDEX IF NOT EXISTS idx_mappings_project ON mappings(project_id);
CREATE INDEX IF NOT EXISTS idx_mappings_midi ON mappings(midi_device_id, midi_channel, midi_cc);
CREATE INDEX IF NOT EXISTS idx_mappings_ucnet ON mappings(ucnet_device_id, ucnet_parameter_id);

-- User preferences table: app-wide settings
CREATE TABLE IF NOT EXISTS preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default preferences
INSERT OR IGNORE INTO preferences (key, value) VALUES 
    ('auto_save_enabled', 'true'),
    ('auto_save_interval_seconds', '30'),
    ('theme', 'dark'),
    ('show_recent_projects', 'true'),
    ('max_recent_projects', '10');
