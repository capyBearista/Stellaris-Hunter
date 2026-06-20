CREATE TABLE playsets (
  id TEXT,
  name TEXT,
  isActive INTEGER,
  syncState TEXT,
  state TEXT
);

CREATE TABLE mods (
  id TEXT,
  displayName TEXT,
  gameRegistryId TEXT,
  dirPath TEXT
);

CREATE TABLE playsets_mods (
  playsetId TEXT,
  modId TEXT,
  enabled INTEGER,
  position INTEGER
);

CREATE TABLE dlc (
  id TEXT,
  displayName TEXT,
  gameRegistryId TEXT,
  dirPath TEXT
);

CREATE TABLE playsets_dlcs (
  playsetId TEXT,
  dlcId TEXT,
  enabled INTEGER
);

INSERT INTO playsets (id, name, isActive, syncState, state) VALUES
  ('playset-1', 'Initial playset', 0, 'NOT_ELIGIBLE', 'private'),
  ('playset-2', 'Bare Bones', 1, 'NOT_ELIGIBLE', 'private');

INSERT INTO mods (id, displayName, gameRegistryId, dirPath) VALUES
  ('mod-1', 'Alpha Mod', 'mod/alpha.mod', 'C:\\mods\\alpha'),
  ('mod-2', 'Beta Mod', 'mod/beta.mod', 'C:\\mods\\beta'),
  ('mod-3', 'Disabled Mod', 'mod/disabled.mod', 'C:\\mods\\disabled');

INSERT INTO playsets_mods (playsetId, modId, enabled, position) VALUES
  ('playset-2', 'mod-1', 1, 0),
  ('playset-2', 'mod-2', 1, 1),
  ('playset-2', 'mod-3', 0, 2),
  ('playset-1', 'mod-3', 1, 0);

INSERT INTO dlc (id, displayName, gameRegistryId, dirPath) VALUES
  ('dlc-1', 'Leviathans Story Pack', 'dlc014_leviathans', 'dlc/dlc014_leviathans.dlc'),
  ('dlc-2', 'Ancient Relics Story Pack', 'dlc028_ancient_relics', 'dlc/dlc028_ancient_relics.dlc'),
  ('dlc-3', 'Plantoids Species Pack', 'dlc009_plantoids', 'dlc/dlc009_plantoids.dlc');

INSERT INTO playsets_dlcs (playsetId, dlcId, enabled) VALUES
  ('playset-2', 'dlc-1', 1),
  ('playset-2', 'dlc-2', 1),
  ('playset-2', 'dlc-3', 0),
  ('playset-1', 'dlc-3', 1);
