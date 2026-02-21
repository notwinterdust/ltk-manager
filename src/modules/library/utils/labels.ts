const TAG_LABELS: Record<string, string> = {
  "league-of-legends": "League of Legends",
  tft: "TFT",
  "champion-skin": "Champion Skin",
  "map-skin": "Map Skin",
  "ward-skin": "Ward Skin",
  ui: "UI",
  hud: "HUD",
  font: "Font",
  sfx: "SFX",
  announcer: "Announcer",
  structure: "Structure",
  minion: "Minion",
  "jungle-monster": "Jungle Monster",
  misc: "Misc",
};

const MAP_LABELS: Record<string, string> = {
  "summoners-rift": "Summoner's Rift",
  aram: "ARAM",
  "teamfight-tactics": "Teamfight Tactics",
  arena: "Arena",
  swarm: "Swarm",
};

function kebabToTitleCase(s: string): string {
  return s
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

export function getTagLabel(tag: string): string {
  return TAG_LABELS[tag] ?? kebabToTitleCase(tag);
}

export function getMapLabel(map: string): string {
  return MAP_LABELS[map] ?? kebabToTitleCase(map);
}
