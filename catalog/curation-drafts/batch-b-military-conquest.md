# Batch B — Military, Conquest, Fleet, Purge, Colossus, Crisis, and Combat Achievements

> Human-review draft. Provides replacement `curation` blocks (not full achievement objects).
> All conditions use `source: "wiki-reviewed"`. DLC conditions added per CURATION_STYLE.md.

---

## conquer_homeworld — A Home Away From Home

Source requirement: Conquer another species' homeworld. Pre-FTL species count.

```json
{
  "tags": [
    "conquest",
    "war",
    "pre-ftl"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "conquered_homeworld",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires conquering the homeworld of any sapient species including pre-FTLs."
    }
  ],
  "warnings": [],
  "planner_notes": "Pre-FTL homeworlds count and are easier targets; high Pre-FTL galaxy settings increase odds of finding one.",
  "known_limitations": [
    "The current save parser does not yet identify planetary homeworld origin or conquest flags."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## winwar — Supremacy

Source requirement: Win a war against another empire. Must be the empire that declared war or was declared war against.

```json
{
  "tags": [
    "war"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "wars_won",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires winning at least one war."
    }
  ],
  "warnings": [
    "Rival empires that refuse to surrender can delay war conclusion."
  ],
  "planner_notes": "Nearly automatic in any run that prosecutes at least one war.",
  "known_limitations": [
    "The current save parser does not yet track war outcomes."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## grandfleet — The Grand Fleet

Source requirement: Build a fleet with total fleet-size above 500 (command limit, not naval capacity).

```json
{
  "tags": [
    "fleet",
    "war",
    "mid-game"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "fleet_count",
      "operator": "greater_than",
      "value": 500,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires a single fleet exceeding 500 command limit (not total naval capacity)."
    }
  ],
  "warnings": [
    "Requires high command limit from technology, traditions, and officer council positions."
  ],
  "planner_notes": "Focus on technologies and traditions that increase command limit; Titan and Juggernaut bonuses help.",
  "known_limitations": [
    "The current save parser does not yet inspect individual fleet command limit values."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## grandadmiral — Grand Admiral

Source requirement: Empire total fleet military power equals 100,000.

```json
{
  "tags": [
    "fleet",
    "war",
    "late-game"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "fleet_power",
      "operator": "greater_than",
      "value": 99999,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires total fleet power exceeding 99,999."
    }
  ],
  "warnings": [],
  "planner_notes": "Natural in any mid-to-late-game economy; focus on alloys and naval capacity.",
  "known_limitations": [
    "The current save parser does not yet sum fleet power from all active fleets."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## citadel_of_death — Citadel of Death

Source requirement: Own a Citadel with 40k fleet power.

```json
{
  "tags": [
    "starbase",
    "war",
    "mid-game"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "fleet_power",
      "operator": "greater_than",
      "value": 39999,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires a Citadel-class starbase with >39,999 fleet power."
    }
  ],
  "warnings": [
    "Starbase command limit techs, anchorage modules, and naval logistics buildings are needed."
  ],
  "planner_notes": "Upgrade a key choke-point starbase to Citadel and stack anchorage + defensive modules.",
  "known_limitations": [
    "The current save parser does not yet inspect individual starbase fleet power values."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The dimension `fleet_power` is used generically; a future starbase-specific dimension may be clearer.

---

## allseeing_eye — All-Seeing Eye

Source requirement: Have Intel Level of 100 on five different empires simultaneously.

```json
{
  "tags": [
    "espionage",
    "late-game",
    "megastructures"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "intel_level_count",
      "operator": "at_least",
      "value": 5,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires full intel (level 100) on five empires at the same time."
    }
  ],
  "warnings": [
    "A fully-upgraded Sentry Array provides +40 passive intel per array; multiple arrays stack."
  ],
  "planner_notes": "Building a Sentry Array dramatically simplifies this; otherwise rely on espionage networks and codebreaking.",
  "known_limitations": [
    "The current save parser does not yet track per-empire intel levels."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `intel_level_count` dimension may need refinement if it refers to number of empires at full intel rather than intel-level tiers.

---

## machine_supremacy — Machine Supremacy

Source requirement: Win the game as a machine uprising after switching to the rebellion during an AI uprising.

```json
{
  "tags": [
    "machine",
    "war",
    "event-limited",
    "story"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "machine_uprising_victory",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires triggering an AI rebellion, switching to the uprising, and achieving victory."
    }
  ],
  "warnings": [
    "The AI rebellion event chain (AI-Related Incidents Situation) may not trigger in every run.",
    "Synthetic ascension or owning AI servitude techs raise the chance."
  ],
  "planner_notes": "Play as a materialist empire with full AI rights research; the rebellion triggers only under specific conditions.",
  "known_limitations": [
    "The current save parser does not yet detect machine uprising events or victory flags."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: The exact trigger conditions for the AI-Related Incidents Situation are complex and involve multiple variables (AI policy, robot tech level, etc.). Rule confidence is low because the event chain is hard to force reliably.

---

## clash_of_the_titans — Clash of the Titans

Source requirement: Defeat a Fallen Empire Titan fleet with a Titan of your own.

```json
{
  "tags": [
    "war",
    "leviathans",
    "late-game",
    "apocalypse",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Apocalypse",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Apocalypse DLC."
    },
    {
      "condition_type": "required",
      "dimension": "titan_built",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building at least one Titan."
    },
    {
      "condition_type": "required",
      "dimension": "war_type",
      "operator": "equals",
      "value": "defensive_war_against_fallen_empire",
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires a Fallen Empire fleet destroyed in battle while the player has a Titan present."
    }
  ],
  "warnings": [
    "Fallen Empire Titan fleets only exist if the Fallen Empire has built Titans; must engage before they awaken.",
    "Set end-game year as late as possible to avoid Awakened Empires."
  ],
  "planner_notes": "Declare war on a Fallen Empire before they awaken; ensure at least one Titan participates in a fleet battle where an FE Titan is destroyed.",
  "known_limitations": [
    "The current save parser does not yet identify Titan participation in battles or Fallen Empire fleet composition."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: The specific condition dimension for Titan-vs-Fallen-Empire-Titan is not directly representable with current dimensions. `war_type` is a best-effort proxy.

---

## no_khan_do — No Khan Do

Source requirement: Kill the Great Khan in battle by destroying their fleet.

```json
{
  "tags": [
    "war",
    "great-khan",
    "mid-game",
    "apocalypse",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Apocalypse",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Apocalypse DLC."
    },
    {
      "condition_type": "required",
      "dimension": "great_khan_spawned",
      "operator": "equals",
      "value": true,
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Great Khan must appear (requires a Marauder empire to survive to mid-game)."
    },
    {
      "condition_type": "required",
      "dimension": "great_khan_killed_in_battle",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires destroying the Great Khan's flagship in combat."
    }
  ],
  "warnings": [
    "If the Great Khan dies of old age or is assassinated, the achievement cannot be earned in that run."
  ],
  "planner_notes": "Keep a strong fleet near Marauder territory; the Khan spawns from the strongest Marauder clan after mid-game starts.",
  "known_limitations": [
    "The current save parser does not yet detect Great Khan state or Khan-in-battle flags."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `great_khan_killed_in_battle` is not in the existing dimension list but follows the pattern of boolean past-tense event dimensions. DIMENSIONS.md has `great_khan_spawned` so this is an extension.

---

## exterminatus — Exterminatus

Source requirement: Use a World Cracker Colossus to destroy another empire's capital planet.

```json
{
  "tags": [
    "colossus",
    "war",
    "conquest",
    "apocalypse",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Apocalypse",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Apocalypse DLC."
    },
    {
      "condition_type": "required",
      "dimension": "colossus_built",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Colossus."
    },
    {
      "condition_type": "required",
      "dimension": "colossus_weapon_type",
      "operator": "equals",
      "value": "world_cracker",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the World Cracker Colossus weapon."
    },
    {
      "condition_type": "required",
      "dimension": "conquered_homeworld",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must target another empire's capital planet specifically."
    }
  ],
  "warnings": [
    "Destroying a capital inflicts a heavy opinion penalty and removes the target empire's capital."
  ],
  "planner_notes": "The wiki suggests creating a vassal, releasing it, and destroying their capital after truce expires.",
  "known_limitations": [
    "The current save parser does not yet detect Colossus construction or weapon assignment."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `conquered_homeworld` dimension is reused here but with the specific nuance that the homeworld must be destroyed, not conquered. This may need a separate dimension in future.

---

## pandoras_world — Pandora's World

Source requirement: Use a Global Pacifier to shield a planet belonging to Fanatic Purifiers, Ravenous Swarms, or Determined Exterminators.

```json
{
  "tags": [
    "colossus",
    "war",
    "apocalypse",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Apocalypse",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Apocalypse DLC."
    },
    {
      "condition_type": "required",
      "dimension": "colossus_built",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Colossus."
    },
    {
      "condition_type": "required",
      "dimension": "colossus_weapon_type",
      "operator": "equals",
      "value": "global_pacifier",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Global Pacifier Colossus weapon."
    },
    {
      "condition_type": "required",
      "dimension": "target_empire_civic",
      "operator": "contains",
      "value": "civic_fanatic_purifiers",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Target must be a genocidal empire (Fanatic Purifiers, Devouring Swarm, or Determined Exterminator)."
    }
  ],
  "warnings": [
    "Only the three specified genocidal civics count; similar civics (e.g. Terravore) do not qualify."
  ],
  "planner_notes": "Force-spawn or keep a genocidal empire alive until Colossus tech is unlocked.",
  "known_limitations": [
    "The current save parser does not yet detect Colossus construction or weapon types."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `target_empire_civic` is not in the existing dimension list; it extends the dimension catalog with a generic target-empire matching dimension.

---

## stay_on_target — Stay on Target

Source requirement: Destroy an enemy Colossus while it is in the process of firing on a planet (or, per wiki hint, simply orbiting a planet).

```json
{
  "tags": [
    "colossus",
    "war",
    "timing-window",
    "apocalypse",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Apocalypse",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Apocalypse DLC."
    },
    {
      "condition_type": "required",
      "dimension": "colossus_destroyed_while_firing",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires destroying an enemy Colossus while it is orbiting a planet (per wiki, not strictly while firing)."
    }
  ],
  "warnings": [
    "Relies on the AI building and fielding a Colossus; can be missed if the target is destroyed elsewhere."
  ],
  "planner_notes": "Once an enemy Colossus is spotted near a planet, station a fleet in the system and attack before it jumps away.",
  "known_limitations": [
    "The current save parser does not yet track Colossus presence, destruction events, or firing state."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: The wiki hint says the Colossus only needs to be in orbit, not actively firing. The timing window is hard to guarantee in a normal run.

---

## holy_water — Holy Water

Source requirement: Drench a Fallen Empire's holy world using a Deluge Machine Colossus.

```json
{
  "tags": [
    "colossus",
    "war",
    "aquatics",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Aquatics",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Aquatics DLC (Deluge Machine is exclusive to this DLC)."
    },
    {
      "condition_type": "required",
      "dimension": "colossus_built",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Colossus."
    },
    {
      "condition_type": "required",
      "dimension": "colossus_weapon_type",
      "operator": "equals",
      "value": "deluge_machine",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Deluge Machine Colossus weapon."
    },
    {
      "condition_type": "required",
      "dimension": "target_planet_class",
      "operator": "equals",
      "value": "pc_holy_world",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must target a Holy World owned by a Fallen Empire."
    }
  ],
  "warnings": [
    "Holy worlds only spawn with the Holy Guardians or Doctrinal Enforcers Fallen Empire present."
  ],
  "planner_notes": "Set Fallen Empires to maximum in galaxy setup to guarantee holy world availability.",
  "known_limitations": [
    "The current save parser does not yet detect Colossus weapon type or holy world modifiers."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `target_planet_class` with value `pc_holy_world` is not in the existing dimension list; it extends planet-class target dimensions.

---

## dreadnought — Dreadnought

Source requirement: Defeat the Automated Dreadnought and choose to repair it.

```json
{
  "tags": [
    "leviathans",
    "war",
    "late-game",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Leviathans",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Leviathans DLC."
    },
    {
      "condition_type": "required",
      "dimension": "ancient_leviathan",
      "operator": "equals",
      "value": "automated_dreadnought",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Automated Dreadnought to spawn in the galaxy."
    },
    {
      "condition_type": "required",
      "dimension": "dreadnought_repaired",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must defeat and then choose to repair/restore the Dreadnought."
    }
  ],
  "warnings": [
    "The Automated Dreadnought leviathan must be present in the galaxy (RNG spawn)."
  ],
  "planner_notes": "Build a strong late-game fleet, locate the Dreadnought, defeat it, and pick the repair option in the event.",
  "known_limitations": [
    "The current save parser does not yet identify leviathan defeat or event choice outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `dreadnought_repaired` is a new boolean dimension. The repair choice is part of the post-battle event chain.

---

## hear_me_roar — Hear me Roar

Source requirement: Defeat the Ether Drake, place a mining station on Dragon Hoard, and incubate the dragon egg.

```json
{
  "tags": [
    "leviathans",
    "war",
    "ether-drake",
    "story",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Leviathans",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Leviathans DLC."
    },
    {
      "condition_type": "required",
      "dimension": "ancient_leviathan",
      "operator": "equals",
      "value": "ether_drake",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Ether Drake leviathan to spawn."
    },
    {
      "condition_type": "required",
      "dimension": "dragon_egg_incubated",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires defeating the Ether Drake, placing a mining station, and hatching the egg via event (MTTH 600 months)."
    }
  ],
  "warnings": [
    "The Ether Drake spawns on a random shielded system; not guaranteed to be in every game."
  ],
  "planner_notes": "After defeating the Ether Drake, place a mining station on the Dragon Hoard and wait for the incubation event (long MTTH).",
  "known_limitations": [
    "The current save parser does not yet identify leviathan defeat or egg-related event flags."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `dragon_egg_incubated` is a new boolean dimension. The MTTH (600 months) means the event can take a very long time to fire.

---

## stellar_performance — Stellar Performance

Source requirement: Defeat the Stellar Devourer and obtain the Devourer Egg Sac planetary modifier.

```json
{
  "tags": [
    "leviathans",
    "war",
    "stellarite-devourer",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Leviathans",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Leviathans DLC."
    },
    {
      "condition_type": "required",
      "dimension": "ancient_leviathan",
      "operator": "equals",
      "value": "stellarite_devourer",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Stellar Devourer leviathan to spawn."
    },
    {
      "condition_type": "required",
      "dimension": "devourer_egg_sac_obtained",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires choosing to retrieve the egg sac after defeating the Stellar Devourer and completing the parade situation."
    }
  ],
  "warnings": [
    "The Stellar Devourer must spawn in the galaxy; not guaranteed."
  ],
  "planner_notes": "After defeat, pick the 'Retrieve the egg sac' option and wait for the parade situation to succeed.",
  "known_limitations": [
    "The current save parser does not yet detect leviathan-specific event outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `devourer_egg_sac_obtained` is a new boolean dimension for this specific event outcome.

---

## unravelling_enigma — Unravelling Enigma

Source requirement: Defeat and complete the event chain of the Enigmatic Fortress.

```json
{
  "tags": [
    "leviathans",
    "war",
    "enigma-fortress",
    "story",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Leviathans",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Leviathans DLC."
    },
    {
      "condition_type": "required",
      "dimension": "ancient_leviathan",
      "operator": "equals",
      "value": "enigma_fortress",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Enigmatic Fortress leviathan to spawn."
    },
    {
      "condition_type": "required",
      "dimension": "enigma_fortress_completed",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the full Enigmatic Fortress event chain (defeat + puzzle)."
    }
  ],
  "warnings": [
    "The Enigmatic Fortress spawn is RNG-dependent."
  ],
  "planner_notes": "After defeating the fortress, work through the multi-stage event chain to unlock the secrets.",
  "known_limitations": [
    "The current save parser does not yet track Enigmatic Fortress completion flags."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `enigma_fortress_completed` is a new boolean dimension.

---

## wraith — Warrior of Light

Source requirement: Defeat the Spectral Wraith.

```json
{
  "tags": [
    "leviathans",
    "war",
    "event-limited",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Leviathans",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Leviathans DLC."
    },
    {
      "condition_type": "required",
      "dimension": "spectral_wraith_defeated",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires defeating the Spectral Wraith."
    }
  ],
  "warnings": [
    "The Spectral Wraith spawns only after an empire researches Psi Jump Drives; it may not appear in every game."
  ],
  "planner_notes": "The Spectral Wraith spawns from the empire that first researches Psi Jump Drives, attacking its territory. Build up fleet power in anticipation.",
  "known_limitations": [
    "The current save parser does not yet detect Spectral Wraith presence or defeat."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: The Spectral Wraith is not a standard leviathan spawn — it triggers as a consequence of researching Psi Jump Drives, making it event-limited with specific tech requirements.

---

## whence_it_came — Whence It Came

Source requirement: Defeat the Dimensional Horror.

```json
{
  "tags": [
    "leviathans",
    "war",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Leviathans",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Leviathans DLC."
    },
    {
      "condition_type": "required",
      "dimension": "ancient_leviathan",
      "operator": "equals",
      "value": "dimensional_horror",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Dimensional Horror leviathan to spawn."
    },
    {
      "condition_type": "required",
      "dimension": "dimensional_horror_defeated",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires defeating the Dimensional Horror."
    }
  ],
  "warnings": [
    "The Dimensional Horror spawn is RNG-dependent."
  ],
  "planner_notes": "Build a strong mid-to-late-game fleet and locate the Dimensional Horror's system.",
  "known_limitations": [
    "The current save parser does not yet detect leviathan defeat flags."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `dimensional_horror_defeated` is a new boolean dimension following the pattern of `ancient_leviathan` + outcome.

---

## last_best_hope — Last, Best Hope

Source requirement: Be selected as president of the League of Non-Aligned Empires during a War in Heaven and defeat both Awakened Empires.

```json
{
  "tags": [
    "war",
    "leviathans",
    "late-game",
    "diplomacy",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Leviathans",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Leviathans DLC."
    },
    {
      "condition_type": "required",
      "dimension": "war_type",
      "operator": "equals",
      "value": "war_in_heaven",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the War in Heaven event (two Fallen Empires awaken and go to war)."
    },
    {
      "condition_type": "required",
      "dimension": "federation_formed",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must be selected as president of the League of Non-Aligned Empires."
    },
    {
      "condition_type": "required",
      "dimension": "awakened_empires_defeated",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Both Awakened Empires must be destroyed or subjugated."
    }
  ],
  "warnings": [
    "War in Heaven requires at least two Fallen Empires that awaken; not guaranteed.",
    "The League president position depends on diplomatic weight among non-aligned powers."
  ],
  "planner_notes": "Maintain high diplomatic weight and strong fleet to be elected League president. Must defeat both Awakened Empires, not just win the war.",
  "known_limitations": [
    "The current save parser does not yet detect War in Heaven, League formation, or Awakened Empire defeat."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: `awakened_empires_defeated` is a new boolean dimension. The War in Heaven event chain itself has complex preconditions (two Fallen Empires that can awaken).

---

## hatchling — 1999 A.D.

Source requirement: Defeat the Voidspawn.

```json
{
  "tags": [
    "leviathans",
    "war",
    "story",
    "distant-stars",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Distant Stars",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Distant Stars DLC."
    },
    {
      "condition_type": "required",
      "dimension": "voidspawn_defeated",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires defeating the Voidspawn."
    }
  ],
  "warnings": [
    "The Voidspawn emerges from a planetary colony via the 'Wormhole' anomaly chain; requires a specific anomaly chain to trigger."
  ],
  "planner_notes": "The Voidspawn event starts with a special anomaly found on habitable planets during surveying.",
  "known_limitations": [
    "The current save parser does not yet detect the Voidspawn event chain or its defeat."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `voidspawn_defeated` is a new boolean dimension for this specific Distant Stars encounter.

---

## tiyanki — A Hump Like a Snow-Hill

Source requirement: Defeat the Tiyanki Matriarch.

```json
{
  "tags": [
    "leviathans",
    "war",
    "tiyanki",
    "distant-stars",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Distant Stars",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Distant Stars DLC."
    },
    {
      "condition_type": "required",
      "dimension": "ancient_leviathan",
      "operator": "equals",
      "value": "tiyanki_matriarch",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Tiyanki Matriarch leviathan to spawn."
    },
    {
      "condition_type": "required",
      "dimension": "tiyanki_matriarch_defeated",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires defeating the Tiyanki Matriarch."
    }
  ],
  "warnings": [
    "The Tiyanki Matriarch spawn is RNG-dependent."
  ],
  "planner_notes": "Locate and defeat the Tiyanki Matriarch with a mid-game fleet.",
  "known_limitations": [
    "The current save parser does not yet detect leviathan defeat flags."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `tiyanki_matriarch_defeated` is a new boolean dimension.

---

## scrapper — Who Scraps the Scrapper

Source requirement: Defeat the Scavenger Bot.

```json
{
  "tags": [
    "leviathans",
    "war",
    "distant-stars",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Distant Stars",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Distant Stars DLC."
    },
    {
      "condition_type": "required",
      "dimension": "ancient_leviathan",
      "operator": "equals",
      "value": "scavenger_bot",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Scavenger Bot leviathan to spawn."
    },
    {
      "condition_type": "required",
      "dimension": "scavenger_bot_defeated",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires defeating the Scavenger Bot."
    }
  ],
  "warnings": [
    "The Scavenger Bot spawn is RNG-dependent."
  ],
  "planner_notes": "Locate and defeat the Scavenger Bot with a mid-to-late-game fleet.",
  "known_limitations": [
    "The current save parser does not yet detect leviathan defeat flags."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `scavenger_bot_defeated` is a new boolean dimension.

---

## directive_67 — Directive 67

Source requirement: As a Clone Army origin empire, denounce the Spiritualist Fallen Empire in the Galactic Community and destroy them.

```json
{
  "tags": [
    "war",
    "origin-gated",
    "galactic-community",
    "federations",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Federations",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Federations DLC (Clone Army origin)."
    },
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_clone_army",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Player empire must have the Clone Army origin."
    },
    {
      "condition_type": "required",
      "dimension": "galactic_community_exists",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "The Galactic Community must exist."
    },
    {
      "condition_type": "required",
      "dimension": "fallen_empire_spiritualist_present",
      "operator": "equals",
      "value": true,
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Holy Guardians or Doctrinal Enforcers Fallen Empire to be present."
    },
    {
      "condition_type": "required",
      "dimension": "fallen_empire_destroyed",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires destroying the Spiritualist Fallen Empire after denouncing them."
    }
  ],
  "warnings": [
    "The Spiritualist Fallen Empire type must be present (RNG). Force-spawning a Scion with Spiritualist ethics can help."
  ],
  "planner_notes": "Pick Clone Army origin, join the Galactic Community, denounce the Spiritualist FE, then declare war and destroy them.",
  "known_limitations": [
    "The current save parser does not yet detect origin, denounce actions, or Fallen Empire destruction."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: Multiple dimensions here are new (`fallen_empire_spiritualist_present`, `fallen_empire_destroyed`). The denounce action is part of the galactic custodian/Galactic Community resolution system.

---

## humble_pie — Humble Pie

Source requirement: Defeat an empire using the Supremacist diplomatic stance with an Animosity or Imperial Rebuke casus belli, forcing them out of Supremacist stance.

```json
{
  "tags": [
    "war",
    "diplomacy",
    "federations",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Federations",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Federations DLC."
    },
    {
      "condition_type": "required",
      "dimension": "wars_won",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires winning a war against an empire with Supremacist stance."
    },
    {
      "condition_type": "required",
      "dimension": "war_type",
      "operator": "equals",
      "value": "animosity_war",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must use the Animosity or Imperial Rebuke casus belli specifically."
    }
  ],
  "warnings": [
    "Target must be using Supremacist diplomatic stance; RNG whether such an empire exists."
  ],
  "planner_notes": "Keep an eye on empires using Supremacist stance and rival them to unlock the correct casus belli.",
  "known_limitations": [
    "The current save parser does not yet track diplomatic stance, casus belli types, or war resolution details."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: The war type dimension value `animosity_war` is speculative. The achievement requirement is specifically about forcing the empire out of Supremacist stance, which may need a more specific outcome dimension.

---

## shoot_to_kill — Shoot To Kill

Source requirement: Destroy 10 ships or conquer a planet of an empire you have not established contact with.

```json
{
  "tags": [
    "first-contact",
    "war",
    "nemesis",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Nemesis",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Nemesis DLC."
    },
    {
      "condition_type": "required",
      "dimension": "first_contact_result",
      "operator": "equals",
      "value": "aggressive",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires entering a first contact war by destroying ships or conquering a planet before full communications."
    }
  ],
  "warnings": [
    "Achievement requires aggressive first-contact policy or being attacked by unknown hostiles."
  ],
  "planner_notes": "Use the Aggressive First Contact policy and engage unknown empire ships to trigger a first contact war.",
  "known_limitations": [
    "The current save parser does not yet classify first-contact outcomes or pre-contact war status."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `first_contact_result` dimension exists in DIMENSIONS.md with example values including `"shoot_to_kill"`. Using `"aggressive"` as a broader outcome value.

---

## big_red_button — Big Red Button

Source requirement: Win the game as a Galactic Nemesis by fully upgrading the Aetherophasic Engine.

```json
{
  "tags": [
    "crisis",
    "war",
    "nemesis",
    "endgame",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Nemesis",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Nemesis DLC."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_path_nemesis",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the Become the Crisis path and fully upgrading the Aetherophasic Engine."
    }
  ],
  "warnings": [
    "All other empires will declare war on you once you reach high crisis level."
  ],
  "planner_notes": "Use the zero-empire trick or lower difficulty; focus on menacing ship tech and menace generation.",
  "known_limitations": [
    "The current save parser does not yet detect crisis path stages or Aetherophasic Engine completion."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the condition is straightforward and well-defined, even if hard to execute.

---

## modern_cincinnatus — Modern Cincinnatus

Source requirement: As Galactic Custodian, defeat the endgame crisis and pass End Custodianship resolution.

```json
{
  "tags": [
    "crisis",
    "galactic-custodian",
    "diplomacy",
    "nemesis",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Nemesis",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Nemesis DLC."
    },
    {
      "condition_type": "required",
      "dimension": "galactic_custodian",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must be the Galactic Custodian."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_defeated",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Any endgame crisis must be defeated while the empire is Galactic Custodian."
    },
    {
      "condition_type": "required",
      "dimension": "galactic_custodian_action",
      "operator": "equals",
      "value": "end_custodianship",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must pass the End Custodianship resolution after the crisis is defeated."
    }
  ],
  "warnings": [
    "Losing custodian status or having the term expire naturally invalidates the achievement."
  ],
  "planner_notes": "Become Custodian before the endgame crisis, defeat it, then proactively end the custodianship via resolution.",
  "known_limitations": [
    "The current save parser does not yet detect custodian status, crisis defeat, or resolution outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `galactic_custodian_action` dimension exists in DIMENSIONS.md with `"end_custodianship"` as an example value.

---

## sic_semper_tyrannis — Sic Semper Tyrannis

Source requirement: Lead a successful rebellion that deposes the Galactic Emperor.

```json
{
  "tags": [
    "war",
    "galactic-emperor",
    "diplomacy",
    "nemesis",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Nemesis",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Nemesis DLC."
    },
    {
      "condition_type": "required",
      "dimension": "galactic_emperor",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Galactic Imperium must be proclaimed and an Emperor must exist."
    },
    {
      "condition_type": "required",
      "dimension": "galactic_emperor_rebellion",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Player must instigate and win a rebellion against the Galactic Emperor."
    }
  ],
  "warnings": [
    "The hardest part is getting an AI empire to become Custodian and then Emperor before the player rebels."
  ],
  "planner_notes": "Support an AI empire becoming Custodian first, allow them to proclaim Imperium, then lead the rebellion.",
  "known_limitations": [
    "The current save parser does not yet detect Galactic Imperium, Emperor status, or rebellion outcomes."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: `galactic_emperor_rebellion` extends the existing dimension `galactic_emperor_rebellion` from DIMENSIONS.md. However, the wiki hint says the hardest part is getting an AI Custodian — raising questions about whether the player can be the Emperor or must rebel against an AI Emperor.

---

## beastmaster — Beastmaster

Source requirement: Defeat an endgame crisis without building any artificial military ships. Only cloned space fauna allowed.

```json
{
  "tags": [
    "crisis",
    "space-fauna",
    "war",
    "grand-archive",
    "dlc-gated",
    "civic-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Grand Archive",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Grand Archive DLC."
    },
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_beastmasters",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Beastmasters civic to clone space fauna."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_defeated",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires defeating any endgame crisis using only cloned space fauna."
    },
    {
      "condition_type": "required",
      "dimension": "artificial_military_ships_built",
      "operator": "equals",
      "value": false,
      "timing": "terminal",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must not build any artificial military ships throughout the entire game."
    }
  ],
  "warnings": [
    "Only cloned space fauna are permitted; building any corvette, destroyer, cruiser, battleship, Titan, or Juggernaut invalidates the run."
  ],
  "planner_notes": "Start with the Beastmasters civic, rely entirely on captured and cloned space fauna for military power. Lower crisis strength to 0.25x.",
  "known_limitations": [
    "The current save parser does not yet detect ship build history or distinguish artificial vs. fauna ships."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `artificial_military_ships_built` is a new boolean dimension representing a constraint check on build history. The civic exact name is speculative; Stellaris uses `civic_beastmasters` or similar.

---

## king_of_monsters — King of Monsters

Source requirement: Win the game via the Behemoth Fury crisis path.

```json
{
  "tags": [
    "crisis",
    "war",
    "endgame",
    "biogenesis",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "BioGenesis",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the BioGenesis DLC."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_path_behemoth_fury",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the Behemoth Fury crisis path."
    }
  ],
  "warnings": [
    "The Behemoth Fury crisis path requires specific origin/empire setup from BioGenesis content."
  ],
  "planner_notes": "Use the zero-empire trick to complete the crisis path without interference.",
  "known_limitations": [
    "The current save parser does not yet detect crisis path stages or Behemoth Fury completion."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## galactic_firestorm — Galactic Firestorm

Source requirement: Win the game through the Hyperthermia Crisis Path.

```json
{
  "tags": [
    "crisis",
    "war",
    "endgame",
    "infernals",
    "dlc-gated"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Infernals",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Infernals DLC."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_path_hyperthermia",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the Galactic Hyperthermia crisis path."
    }
  ],
  "warnings": [],
  "planner_notes": "Use the zero-empire trick for an easier path to victory.",
  "known_limitations": [
    "The current save parser does not yet detect Hyperthermia crisis path stages."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## does_not_compute — Does Not Compute

Source requirement: Defeat the Contingency by destroying Nexus Zero-One.

```json
{
  "tags": [
    "crisis",
    "contingency",
    "war",
    "endgame"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "endgame_crisis",
      "operator": "equals",
      "value": "contingency",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Contingency must be the endgame crisis that spawns."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_defeated",
      "operator": "equals",
      "value": "contingency",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires destroying the Contingency's final hub (Nexus Zero-One) and defeating all Sterilization Hubs."
    }
  ],
  "warnings": [
    "Blocked if a different crisis appears. Force Contingency in galaxy settings if available."
  ],
  "planner_notes": "Lower crisis strength to 0.25x; focus on shield-damage weapons (Kinetic/Energy vs. Contingency shield-heavy fleets).",
  "known_limitations": [
    "The current save parser does not yet parse crisis state or final-hub destruction flags."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the condition is well-defined despite the challenge.

---

## rift_sealed — Rift Sealed

Source requirement: Defeat the Extradimensional Invaders (Unbidden) by destroying the final Dimensional Portal.

```json
{
  "tags": [
    "crisis",
    "unbidden",
    "war",
    "endgame"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "endgame_crisis",
      "operator": "equals",
      "value": "unbidden",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Extradimensional Invaders (Unbidden) must be the endgame crisis."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_defeated",
      "operator": "equals",
      "value": "unbidden",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires destroying the final Dimensional Portal."
    }
  ],
  "warnings": [
    "Blocked if a different crisis appears. Force Unbidden in galaxy settings if available."
  ],
  "planner_notes": "Lower crisis strength to 0.25x; Unbidden fleets are weak to Kinetic weapons.",
  "known_limitations": [
    "The current save parser does not yet parse crisis state or portal destruction flags."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## there_be_dragons — There Be Dragons

Source requirement: Have 13 owned space dragons of any kind in the empire's capital system.

```json
{
  "tags": ["war", "leviathans", "space-fauna", "aquatics", "dlc-gated", "origin-gated"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Aquatics",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Aquatics DLC (Here Be Dragons origin)."
    },
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_here_be_dragons",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Here Be Dragons origin to obtain and breed space dragons."
    },
    {
      "condition_type": "required",
      "dimension": "space_fauna_type_captured",
      "operator": "contains",
      "value": "space_dragon",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires 13 owned space dragons (any kind) in the empire's capital system."
    }
  ],
  "warnings": [
    "The Sky Dragon from the origin counts but must survive. At least 10 Fledgling Dragons must be grown to replace it in case of loss.",
    "Dragons can be killed in battle — protect them throughout the run."
  ],
  "planner_notes": "Complete the Here Be Dragons origin event chain, preserve the Sky Dragon, and grow at least 10 Fledgling Dragons (from the hatchery). Keep all dragons alive in the capital system until the count reaches 13.",
  "known_limitations": [
    "The current save parser does not yet detect space fauna ownership by type or count per system."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The origin spawns a Sky Dragon and allows hatching Fledgling Dragons from the hatchery building. The exact growth mechanics are documented in the Here Be Dragons origin event chain. A new `space_fauna_type_captured` or `owned_space_dragons` dimension would be needed for precise parser evaluation.

---

## unstoppable_force — Unstoppable Force

Source requirement: Build a Juggernaut ship type.

```json
{
  "tags": ["fleet", "war", "late-game", "federations", "dlc-gated"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Federations",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Juggernaut ship type requires the Federations DLC."
    },
    {
      "condition_type": "required",
      "dimension": "fleet_count",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building the Juggernaut mobile shipyard, a unique Titan-class vessel."
    }
  ],
  "warnings": [
    "The Juggernaut requires Mega-Engineering technology and significant alloy/influence investment.",
    "Only one Juggernaut can be owned at a time per empire."
  ],
  "planner_notes": "Research Mega-Engineering, then build the Juggernaut at any shipyard. It is a natural late-game fleet goal. The Juggernaut provides a mobile shipyard and repair aura.",
  "known_limitations": [
    "The current save parser does not yet detect Juggernaut construction or distinguish ship types."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The requirement is straightforward — build one Juggernaut. The `fleet_count` dimension is a proxy since there is no dedicated `juggernaut_built` dimension.

---

## synth_detector — Voight-Kampff

Source requirement: Complete the event chain to detect synthetic infiltrators during the Contingency crisis.

```json
{
  "tags": [
    "crisis",
    "contingency",
    "story",
    "event-limited"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "endgame_crisis",
      "operator": "equals",
      "value": "contingency",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Contingency must be the endgame crisis that spawns."
    },
    {
      "condition_type": "required",
      "dimension": "special_project_completed_type",
      "operator": "equals",
      "value": "synthetic_infiltrator_detection",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the unique event chain for detecting synthetic infiltrators spawned during the Contingency."
    }
  ],
  "warnings": [
    "This event chain only fires during the Contingency crisis; requires specific conditions to trigger."
  ],
  "planner_notes": "Force the Contingency crisis in galaxy settings. The detection event chain should start automatically.",
  "known_limitations": [
    "The current save parser does not yet parse crisis state or contingency-specific event flags."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: `special_project_completed_type` exists in DIMENSIONS.md but the value `synthetic_infiltrator_detection` is speculative — the exact event chain name is not confirmed from publicly available wiki data.
