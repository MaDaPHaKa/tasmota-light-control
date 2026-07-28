PRAGMA foreign_keys=OFF;

ALTER TABLE profiles RENAME TO profiles_old;
CREATE TABLE profiles (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    name_normalized TEXT NOT NULL UNIQUE,
    dimmer INTEGER NOT NULL CHECK(dimmer BETWEEN 1 AND 100),
    mode TEXT NOT NULL CHECK(mode IN ('rgb','color_temperature')),
    rgb_color TEXT,
    color_temperature_kelvin INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK((mode='rgb' AND rgb_color GLOB '#[0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]' AND color_temperature_kelvin IS NULL) OR (mode='color_temperature' AND rgb_color IS NULL AND color_temperature_kelvin BETWEEN 2000 AND 6000))
);
INSERT INTO profiles SELECT * FROM profiles_old;
DROP TABLE profiles_old;

ALTER TABLE reset_settings RENAME TO reset_settings_old;
CREATE TABLE reset_settings (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    dimmer INTEGER NOT NULL CHECK(dimmer BETWEEN 1 AND 100),
    mode TEXT NOT NULL CHECK(mode IN ('rgb','color_temperature')),
    rgb_color TEXT,
    color_temperature_kelvin INTEGER,
    fade INTEGER DEFAULT 1,
    speed INTEGER DEFAULT 4,
    CHECK((mode='rgb' AND rgb_color GLOB '#[0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]' AND color_temperature_kelvin IS NULL) OR (mode='color_temperature' AND rgb_color IS NULL AND color_temperature_kelvin BETWEEN 2000 AND 6000))
);
INSERT INTO reset_settings SELECT * FROM reset_settings_old;
DROP TABLE reset_settings_old;

PRAGMA foreign_keys=ON;
