CREATE TABLE profiles_new (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    name_normalized TEXT NOT NULL UNIQUE,
    dimmer INTEGER NOT NULL CHECK(dimmer BETWEEN 1 AND 100),
    mode TEXT NOT NULL CHECK(mode IN ('rgb','color_temperature','mixed')),
    rgb_color TEXT,
    color_temperature_kelvin INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    mixed_color TEXT,
    CHECK(
        (mode='rgb' AND rgb_color IS NOT NULL AND rgb_color GLOB '#[0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]' AND color_temperature_kelvin IS NULL AND mixed_color IS NULL)
        OR (mode='color_temperature' AND rgb_color IS NULL AND color_temperature_kelvin IS NOT NULL AND color_temperature_kelvin BETWEEN 2000 AND 6000 AND mixed_color IS NULL)
        OR (mode='mixed' AND rgb_color IS NULL AND color_temperature_kelvin IS NULL AND mixed_color IS NOT NULL AND length(mixed_color)=11 AND substr(mixed_color,1,1)='#' AND substr(mixed_color,2) NOT GLOB '*[^0-9A-F]*')
    )
);
INSERT INTO profiles_new (id,name,name_normalized,dimmer,mode,rgb_color,color_temperature_kelvin,created_at,updated_at)
    SELECT id,name,name_normalized,dimmer,mode,rgb_color,color_temperature_kelvin,created_at,updated_at FROM profiles;
DROP TABLE profiles;
ALTER TABLE profiles_new RENAME TO profiles;

CREATE TABLE reset_settings_new (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    dimmer INTEGER NOT NULL CHECK(dimmer BETWEEN 1 AND 100),
    mode TEXT NOT NULL CHECK(mode IN ('rgb','color_temperature','mixed')),
    rgb_color TEXT,
    color_temperature_kelvin INTEGER,
    fade INTEGER DEFAULT 1,
    speed INTEGER DEFAULT 4,
    mixed_color TEXT,
    CHECK(
        (mode='rgb' AND rgb_color IS NOT NULL AND rgb_color GLOB '#[0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]' AND color_temperature_kelvin IS NULL AND mixed_color IS NULL)
        OR (mode='color_temperature' AND rgb_color IS NULL AND color_temperature_kelvin IS NOT NULL AND color_temperature_kelvin BETWEEN 2000 AND 6000 AND mixed_color IS NULL)
        OR (mode='mixed' AND rgb_color IS NULL AND color_temperature_kelvin IS NULL AND mixed_color IS NOT NULL AND length(mixed_color)=11 AND substr(mixed_color,1,1)='#' AND substr(mixed_color,2) NOT GLOB '*[^0-9A-F]*')
    )
);
INSERT INTO reset_settings_new (singleton,dimmer,mode,rgb_color,color_temperature_kelvin,fade,speed)
    SELECT singleton,dimmer,mode,rgb_color,color_temperature_kelvin,fade,speed FROM reset_settings;
DROP TABLE reset_settings;
ALTER TABLE reset_settings_new RENAME TO reset_settings;
