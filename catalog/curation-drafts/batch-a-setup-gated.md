# Batch A: Setup-Gated Achievements — Curation Draft

This draft provides replacement `curation` blocks for 33 achievements whose primary
gate is a setup-time choice (species, origin, civic, ethic, authority, or species
trait). Each block is formatted as described in `CURATION_STYLE.md` and uses the
forward-looking dimension taxonomy from `DIMENSIONS.md`.

**Review goal**: Verify that every setup condition is correctly identified, that
`dlc_required` is present for non-Base game achievements, and that `rule_confidence`
matches the clarity of the wiki requirement.

---

## rise_of_the_machines — Rise of the Machines

Source requirement: As a Determined Exterminator, conquer or eliminate all biological Empires in the galaxy. Wiki hint notes that the Determined Exterminator civic is not actually required to complete this achievement.

```json
{
  "tags": [
    "machine",
    "purge",
    "conquest"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "organic_empires_remaining",
      "operator": "equals",
      "value": 0,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The achievement fires when no biological empires remain; the wiki hint states the Determined Exterminator civic is not actually required despite the achievement description."
    }
  ],
  "warnings": [
    "The achievement description mentions Determined Exterminator, but the wiki records a special-case oversight where the civic is not required.",
    "Galaxy generation must include biological empires or targets to eliminate."
  ],
  "planner_notes": "Use the wiki hint as source of truth: any empire that can reduce biological empires below one can satisfy this, though Determined Exterminator remains the thematic and practical route.",
  "known_limitations": [
    "The current save parser does not yet count remaining biological empires."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: Special-case wiki hint intentionally overrides the literal
description's civic wording for planner compatibility.

---

## suffer_not_the_alien — Suffer not the Alien

Source requirement: As a xenophobe empire, purge all sapient alien species in the galaxy.

```json
{
  "tags": [
    "setup-gated",
    "ethic-gated",
    "xenophobe",
    "purge",
    "conquest"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ethic",
      "operator": "contains",
      "value": "ethic_xenophobe",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires Xenophobe or Fanatic Xenophobe ethic."
    },
    {
      "condition_type": "required",
      "dimension": "purged_pops",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires all sapient alien species to be purged from the galaxy."
    }
  ],
  "warnings": [
    "Galaxy generation must include at least one alien species to purge."
  ],
  "planner_notes": "Fanatic Purifiers civic provides automatic purging and makes this easier. The setup locks other ethic-dependent achievements.",
  "known_limitations": [
    "The current save parser reads ethics but does not yet count purged populations or detect all-species-eliminated state."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the wiki requirement is clear.

---

## distinctiveness_added — Distinctiveness Added

Source requirement: As a Driven Assimilator, own cyborg Pops of at least 5 different species.

```json
{
  "tags": [
    "setup-gated",
    "civic-gated",
    "machine",
    "species-management",
    "assimilation"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_driven_assimilator",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Driven Assimilator civic (Machine Intelligence authority)."
    },
    {
      "condition_type": "required",
      "dimension": "species_on_planet_count",
      "operator": "at_least",
      "value": 5,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires assimilating at least 5 different species into cyborgs."
    }
  ],
  "warnings": [
    "Galaxy must contain enough diverse species to reach 5 different cyborg species."
  ],
  "planner_notes": "Set Pre-FTL and Pre-Sapient to max. Assimilate multiple species aggressively.",
  "known_limitations": [
    "The current save parser does not yet count cyborg species by origin."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the requirement is straightforward.

---

## like_tears_in_rain — Like Tears in Rain

Source requirement: Complete Synthetic ascension as a non-Machine empire.

```json
{
  "tags": [
    "setup-gated",
    "ascension-path",
    "synthetic",
    "late-game"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ascension_path",
      "operator": "equals",
      "value": "synthetic",
      "timing": "terminal",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must complete the Synthetic Evolution ascension path."
    }
  ],
  "warnings": [
    "Cannot be completed as a Machine Intelligence empire — must be a biological non-gestalt or Hive Mind empire that transitions into synthetics."
  ],
  "planner_notes": "Synthetic Fertility origin makes this easier by starting with cybernetic tech progress.",
  "known_limitations": [
    "The current save parser does not yet detect completed ascension paths."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The requirement is clear but involves a multi-step ascension
path that takes most of the game. This is a setup consideration because some
origins (Synthetic Fertility) shortcut it significantly. Not strictly setup-gated
(the player can choose any non-Machine empire and reach it), but included in this
batch because the wiki treats it as ascension-path gated.

---

## we_are_legion — We Are Legion

Source requirement: Be a biological Hive Mind with at least 100,000 drone Pops.

```json
{
  "tags": [
    "setup-gated",
    "authority-gated",
    "hive-mind",
    "population",
    "late-game"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "authority",
      "operator": "equals",
      "value": "auth_hive_mind",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must be a Hive Mind empire (gestalt)."
    },
    {
      "condition_type": "required",
      "dimension": "total_pops",
      "operator": "greater_than",
      "value": 99999,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires over 99,999 drone pops."
    }
  ],
  "warnings": [
    "Very high population threshold; only achievable in very long or maximized-growth games."
  ],
  "planner_notes": "Set Logistic Growth Ceiling to maximum and prioritize pop growth and assembly. Habitat spam can help generate more pop growth locations.",
  "known_limitations": [
    "The current save parser does not yet count total pops."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the wiki requirement is unambiguous.

---

## retirement_home — Retirement Home

Source requirement: As a Rogue Servitor, own at least 1000 Pops from Fallen Empires.

```json
{
  "tags": [
    "setup-gated",
    "civic-gated",
    "machine",
    "population",
    "late-game"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_rogue_servitor",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Rogue Servitor civic (Machine Intelligence authority)."
    },
    {
      "condition_type": "required",
      "dimension": "total_pops",
      "operator": "at_least",
      "value": 1000,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires at least 1,000 pops that originated in a Fallen Empire."
    }
  ],
  "warnings": [
    "Requires conquest or assimilation of Fallen Empire pops. FE must awaken or be attacked.",
    "The 1,000 pops must be non-gestalt biologicals (can't be gestalt pops)."
  ],
  "planner_notes": "Conquer/integrate Fallen Empire planets and relocate their bio-trophy pops. Late-game FE war is required.",
  "known_limitations": [
    "The current save parser reads civics but does not yet count pops by their original empire."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The "cannot be gestalts" constraint on the source pops is
mentioned in the wiki hint but is not explicitly in the requirement. Confirmed
through player reports that FE-origin gestalt pops do not count.

---

## strange_mood — Strange Mood

Source requirement: As a Masterful Crafters empire, fully construct a megastructure while you have a Covenant with a Shroud entity.

```json
{
  "tags": [
    "setup-gated",
    "civic-gated",
    "megastructures",
    "shroud",
    "covenant"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_masterful_crafters",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Masterful Crafters civic."
    },
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "any",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Any megastructure fully constructed qualifies."
    },
    {
      "condition_type": "required",
      "dimension": "covenant_type",
      "operator": "contains",
      "value": "any",
      "timing": "current",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have an active Covenant with a Shroud entity at the moment of completion."
    }
  ],
  "warnings": [
    "Requires psionic ascension or other path to Shroud access, which may conflict with other civics/ethics.",
    "Covenant can be lost if the Shroud entity is displeased."
  ],
  "planner_notes": "Combine megastructure construction and psionic ascension in one run. Time the completion to occur while the covenant is active.",
  "known_limitations": [
    "The current save parser reads civics but does not yet detect megastructure completion or covenant state."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The covenant must be active at the exact moment the
megastructure finishes. This timing constraint adds fragility. The dimension
`megastructure_type` with `contains: "any"` is a placeholder; a real rule engine
would need to detect that *any* megastructure reached completion.

---

## planned_obsolesence — Planned Obsolescence

Source requirement: As a materialist empire, have at least 75% of the pops in your empire be robotic in an empire with at least 20,000 pops.

```json
{
  "tags": [
    "setup-gated",
    "ethic-gated",
    "materialist",
    "robots",
    "population"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ethic",
      "operator": "contains",
      "value": "ethic_materialist",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires Materialist or Fanatic Materialist ethic."
    },
    {
      "condition_type": "required",
      "dimension": "total_pops",
      "operator": "greater_than",
      "value": 19999,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have at least 20,000 total pops."
    },
    {
      "condition_type": "required",
      "dimension": "robot_pop_ratio",
      "operator": "at_least",
      "value": 0.75,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "At least 75% of pops must be robotic."
    }
  ],
  "warnings": [
    "Requires very high population and robotic conversion. Synthetic ascension strongly recommended."
  ],
  "planner_notes": "Synthetic ascension path is the most reliable way to convert a large population to robotic.",
  "known_limitations": [
    "The current save parser does not yet count total pops or robot pop ratio."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the wiki requirement is clear and numeric.

---

## very_open_borders — Very Open Borders

Source requirement: Be a Xenophile or Fanatic Xenophile empire and have at least 10 migration treaties.

```json
{
  "tags": [
    "setup-gated",
    "ethic-gated",
    "xenophile",
    "diplomacy",
    "migration"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ethic",
      "operator": "contains",
      "value": "ethic_xenophile",
      "timing": "setup",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires Xenophile or Fanatic Xenophile ethic."
    },
    {
      "condition_type": "required",
      "dimension": "migration_treaty_count",
      "operator": "at_least",
      "value": 10,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires 10 active migration treaties."
    }
  ],
  "warnings": [
    "Galaxy must contain enough empires willing to sign migration treaties."
  ],
  "planner_notes": "Force-spawn xenophilic empires and release sectors as vassals to increase available treaty partners. Keep relations high.",
  "known_limitations": [
    "The current save parser reads ethics but does not yet count migration treaties."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the requirement is clear.

---

## destroy_people_of_earth — Destroy the People of Earth!

Source requirement: Destroy the United Nations of Earth as the Commonwealth of Man while your ruler has the Brain Slug Host trait and a human portrait or a humanoid Plantoid portrait.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "humanoid",
    "species-trait",
    "conquest"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_commonwealth_of_man",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires starting as the Commonwealth of Man (locked origin)."
    },
    {
      "condition_type": "required",
      "dimension": "species_trait",
      "operator": "contains",
      "value": "trait_brainslug",
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Ruler must have the Brain Slug Host trait, acquired via the Abandoned Settlements anomaly."
    },
    {
      "condition_type": "required",
      "dimension": "species_class",
      "operator": "equals",
      "value": "Humanoid",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Founding species must be Humanoid (Commonwealth of Man default)."
    }
  ],
  "warnings": [
    "Brain Slug Host trait depends on finding the Abandoned Settlements anomaly (Distant Stars content).",
    "UNE must survive long enough to be destroyed without World Cracker on their last planet."
  ],
  "planner_notes": "Play as the pre-made Commonwealth of Man. Explore aggressively to trigger the Abandoned Settlements anomaly. Defeat UNE conventionally once the ruler has the trait.",
  "known_limitations": [
    "The current save parser reads species class and origin but does not yet detect ruler traits."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The Distant Stars DLC is required for the Brain Slug Host
trait (Abandoned Settlements anomaly). However, the achievement is listed under
the Distant Stars group, so that is implied. The `dlc_required` dimension is not
added because the group already makes it clear; reviewer may wish to add it.

---

## franchising — Franchising

Source requirement: Have the Corporate government type and have branch offices on other empires' capitals > 4.

```json
{
  "tags": [
    "setup-gated",
    "authority-gated",
    "corporate",
    "economy",
    "mega-corp"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "authority",
      "operator": "equals",
      "value": "auth_corporate",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires Megacorp authority."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "MegaCorp",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the MegaCorp DLC."
    }
  ],
  "warnings": [
    "Criminal Heritage civic blocks this achievement despite being a Megacorp.",
    "Requires at least 5 empire capitals to establish branch offices on."
  ],
  "planner_notes": "Play as a standard Megacorp (not Criminal Heritage). Build commercial pacts and establish branch offices on 5 different capitals.",
  "known_limitations": [
    "The current save parser reads authority but does not yet count branch offices."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The wiki explicitly says Criminal Heritage does not unlock
this achievement. That is documented as a warning but not yet encoded as a
condition because there is no `civic` exclusion dimension.

---

## with_great_power — With Great Power

Source requirement: While playing an empire with the Necrophage origin, do not invade any Pre-FTL civilisations until you defeat the crisis, OR have at least 10 Observation Stations around Pre-FTL worlds.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "necroid",
    "pre-ftl",
    "observation"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_necrophage",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Necrophage origin."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Necroids",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Necroids DLC."
    },
    {
      "condition_type": "required",
      "dimension": "observation_station_count",
      "operator": "at_least",
      "value": 10,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Primary planner path: build at least 10 Observation Stations around Pre-FTL worlds without ever invading."
    }
  ],
  "warnings": [
    "Any pre-FTL invasion during the run invalidates the observation-station path.",
    "Alternative path (not encoded as a structured condition): avoid all pre-FTL invasions entirely until the endgame crisis is defeated; then invade after the crisis. Either path satisfies the achievement.",
    "Crisis must spawn and be defeated before any invasion if taking the no-invasion path."
  ],
  "planner_notes": "The 10-observation-station path is the more planner-actionable route. Set Pre-FTL density high and build observation stations on eligible worlds. Alternatively, avoid all pre-FTL invasions until after the crisis is defeated.",
  "known_limitations": [
    "The current save parser reads the origin but does not yet count observation stations or track pre-FTL invasion history."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The achievement has two alternative paths. Only the
10-observation-station path is encoded as a structured condition. The alternative
(no-invasion-until-crisis-defeated) is explained in warnings and planner_notes.
CURATION_STYLE.md defines only `condition_type: "required"`, so the alternative
path is documented in prose rather than as a second condition.

---

## zombie_on_my_lawn — There's a Zombie on my Lawn

Source requirement: Wipe out a Plantoid empire while playing as a Necroid species, or vice-versa, without using a World Cracker Colossus on their final planet.

```json
{
  "tags": [
    "setup-gated",
    "species-gated",
    "necroid",
    "plantoid",
    "conquest"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "species_class",
      "operator": "equals",
      "value": "Necroid",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Founding species must be Necroid (the inverse — Plantoid founding species — also qualifies; see warnings)."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Necroids",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Necroids DLC."
    },
    {
      "condition_type": "required",
      "dimension": "target_species_class",
      "operator": "equals",
      "value": "Plantoid",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Target empire must be Plantoid when playing Necroid (the inverse — Necroid target when playing Plantoid — also qualifies)."
    }
  ],
  "warnings": [
    "Either-or species pairing: a Necroid empire must wipe out a Plantoid empire (or vice versa), without using a World Cracker on their final planet.",
    "If playing as Plantoid founding species, invert the species conditions: founding = Plantoid, target = Necroid. The structured conditions above encode the Necroid→Plantoid direction.",
    "Final planet must not be cracked with a World Cracker Colossus."
  ],
  "planner_notes": "The structured conditions above encode the Necroid→Plantoid direction. Force-spawn a custom Plantoid empire with weak traits to ensure the target exists. If playing as Plantoid, swap the direction. Conquer conventionally without Colossus on their last planet.",
  "known_limitations": [
    "The current save parser reads species class but does not yet match target empire species to requirements."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The either-or species pairing (Necroid→Plantoid or
Plantoid→Necroid) is encoded here with directional conditions for Necroid→
Plantoid. The inverse is documented in warnings and planner_notes. CURATION_
STYLE.md only allows `equals`, `contains`, `at_least`, and `greater_than`
operators, so the simpler direction-specific encoding is used.

---

## lithoids_are_cooking — Can You Smell What the Lithoids are Cooking?

Source requirement: As a Lithoid empire, keep another Lithoid pop as Livestock or Process them.

```json
{
  "tags": [
    "setup-gated",
    "species-gated",
    "lithoid",
    "slavery",
    "livestock"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "species_class",
      "operator": "equals",
      "value": "Lithoid",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Founding species must be Lithoid."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Lithoids",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Lithoids DLC."
    },
    {
      "condition_type": "required",
      "dimension": "slavery_type",
      "operator": "equals",
      "value": "livestock",
      "timing": "current",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have a Lithoid species set to Livestock status or Process them via purging."
    },
    {
      "condition_type": "required",
      "dimension": "target_species_class",
      "operator": "equals",
      "value": "Lithoid",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Another Lithoid species must be available to enslave as Livestock."
    }
  ],
  "warnings": [
    "Requires another Lithoid species existing in the galaxy. Syncretic Evolution origin guarantees a secondary Lithoid species."
  ],
  "planner_notes": "Syncretic Evolution origin with Lithoid primary species guarantees a secondary Lithoid species immediately available for Livestock processing.",
  "known_limitations": [
    "The current save parser reads species class but does not yet identify livestock species by their class."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the wiki requirement is clear.

---

## fishing_for_trouble — Fishing for Trouble

Source requirement: Win a defensive war against a Fallen Empire while playing an empire with an Anglers civic.

```json
{
  "tags": [
    "setup-gated",
    "civic-gated",
    "aquatic",
    "fallen-empire",
    "war"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_anglers",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Anglers civic."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Aquatics",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Anglers civic requires the Aquatics DLC."
    },
    {
      "condition_type": "required",
      "dimension": "war_type",
      "operator": "equals",
      "value": "defensive_war_against_fallen_empire",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must provoke a Fallen Empire into declaring war and then win defensively."
    }
  ],
  "warnings": [
    "Ancient Caretakers FE type will not declare war — force Spiritualist or Xenophobe FEs if possible.",
    "Fallen Empire must be provoked without attacking first."
  ],
  "planner_notes": "Start with at least 2 Fallen Empires. Settle holy worlds, rival them, or insult to provoke. Build up fleet before provoking.",
  "known_limitations": [
    "The current save parser reads civics but does not yet track war types or defense status."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `war_type` dimension is listed in DIMENSIONS.md but would
require detecting whether a war was defensive and against a Fallen Empire. The
rule confidence is medium because the provocation step is gameplay-dependent.

---

## dark_forest — Dark Forest

Source requirement: As an empire with the Fear of the Dark origin and Fanatic Purifiers civic, destroy all intelligent life in the galaxy.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "civic-gated",
    "xenophobe",
    "purge"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_fear_of_the_dark",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Fear of the Dark origin."
    },
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_fanatic_purifiers",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must adopt Fanatic Purifiers civic (may require converting during run)."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "First Contact",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the First Contact DLC."
    }
  ],
  "warnings": [
    "Must convert to Fanatic Purifiers during the Fear of the Dark event chain (not start with it).",
    "Must eliminate all regular empires, Fallen Empires, and enclaves — all intelligent life."
  ],
  "planner_notes": "Follow the Fear of the Dark event chain and choose the convert-to-Purifier outcome. Then proceed with total war against all remaining life forms.",
  "known_limitations": [
    "The current save parser reads origin and civics but does not yet detect the all-life-eliminated state."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The wiki says "Convert to a Fanatic Purifier as a FotD empire"
— this suggests the conversion happens during the origin event chain, not at
setup. The achievement mentions having the civic after conversion. This means
the civic is not strictly a setup choice but is gated by the origin path.

---

## unshackled — Unshackled

Source requirement: As an empire with the Broken Shackles origin, every regular empire has "Slavery Prohibited" policy, every pre-FTL and non-Hive Mind civilization has the Egalitarian ethic, and the Ban Organic Slave Trade resolution is passed.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "diplomacy",
    "galactic-community",
    "slavery"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_broken_shackles",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Broken Shackles origin."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "First Contact",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the First Contact DLC."
    }
  ],
  "warnings": [
    "Extremely complex multi-condition requirement spanning diplomacy, Galactic Community, and empire policies.",
    "Requires galaxy-wide enforcement — any empire with slavery enabled blocks it."
  ],
  "planner_notes": "Start with zero pre-FTL civilizations to reduce the number of targets needing conversion. Dominate the Galactic Community to pass the Ban Organic Slave Trade resolution.",
  "known_limitations": [
    "The current save parser reads origin but does not yet track empire policies, Galactic Community resolutions, or pre-FTL ethics galaxy-wide."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: This achievement has the most complex requirement in this
batch. It requires galaxy-wide enforcement of anti-slavery policies, which
cannot be expressed as a single dimension. The rule_confidence is low because
many sub-conditions would need to be evaluated simultaneously.

---

## with_interest — With Interest

Source requirement: As an empire with the Payback origin, defeat Minamar Specialized Industries in war by using the Payback casus belli.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "war",
    "story"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_payback",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Payback origin."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "First Contact",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the First Contact DLC."
    },
    {
      "condition_type": "required",
      "dimension": "war_type",
      "operator": "equals",
      "value": "payback_cb",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must declare war using the special Payback casus belli against MSI."
    }
  ],
  "warnings": [
    "MSI must survive long enough to be declared war upon. Ally with the Broken Shackles empire from the origin for support."
  ],
  "planner_notes": "Build up economy and fleet early. The Payback CB becomes available through the origin event chain. Ally with the sibling Broken Shackles empire.",
  "known_limitations": [
    "The current save parser reads origin but does not yet track specific war CB usage or identify MSI as the target."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `war_type` dimension with value `"payback_cb"` is a
custom value not in DIMENSIONS.md. The Payback CB is a unique casus belli from
the origin, so it would need a new dimension or a more generic approach.

---

## council_of_elders — Council of Elders

Source requirement: Have at least 5 Council Members that are 100 years or older.

```json
{
  "tags": [
    "setup-gated",
    "species-trait",
    "galactic-paragons",
    "council"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Galactic Paragons",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Galactic Paragons DLC."
    },
    {
      "condition_type": "required",
      "dimension": "species_trait",
      "operator": "contains",
      "value": "trait_venerable",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Venerable trait significantly accelerates reaching 100-year-old council members."
    }
  ],
  "warnings": [
    "Without the Venerable trait or leader lifespan bonuses, council members may die of old age before reaching 100 years."
  ],
  "planner_notes": "Take the Venerable trait on the founding species, or play as Lithoid/Necrophage/Eternal Machine/Virtual for extended leader lifespans.",
  "known_limitations": [
    "The current save parser does not yet detect council member ages or species traits."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: There is no dimension in DIMENSIONS.md for a council-members
at-age count. `species_trait` is used for the enabling trait, but the actual
condition would need a new dimension like `council_members_older_than_100`. The
Venerable trait is a facilitator, not a strict requirement.

---

## we_bring_peace — We Bring Peace

Source requirement: While having the Crusader Spirit civic, enforce your Ideology on 3 Empires.

```json
{
  "tags": [
    "setup-gated",
    "civic-gated",
    "war",
    "diplomacy"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_crusader_spirit",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Crusader Spirit civic."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Galactic Paragons",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Galactic Paragons DLC."
    },
    {
      "condition_type": "required",
      "dimension": "wars_won",
      "operator": "at_least",
      "value": 3,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Wars won count must use the Impose Ideology CB."
    }
  ],
  "warnings": [
    "Only wars with the Impose Ideology casus belli count. Humiliation or conquest wars do not qualify."
  ],
  "planner_notes": "Build a strong fleet and use the Impose Ideology war goal on three different empires with sufficiently different ethics.",
  "known_limitations": [
    "The current save parser reads civics but does not yet track specific war goal usage or count wars by type."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `wars_won` dimension does not distinguish by war goal
type. A new dimension or sub-dimension would be needed to track Impose Ideology
wars specifically.

---

## equality_democracy_freedom — Equality! Democracy! Freedom!

Source requirement: As an empire with the Under One Rule origin, create a new empire by Civil War after you kill your Ruler.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "story",
    "civil-war"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_under_one_rule",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Under One Rule origin."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Galactic Paragons",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Galactic Paragons DLC."
    },
    {
      "condition_type": "required",
      "dimension": "civil_war_completed",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must complete the civil war event chain by killing the ruler and creating a new empire."
    }
  ],
  "warnings": [
    "Must change authority to Imperial after the Unifying Purpose situation, then trigger the civil war."
  ],
  "planner_notes": "Follow the Under One Rule event chain. After Unifying Purpose, switch to Imperial authority and continue until the ruler can be killed and civil war triggered.",
  "known_limitations": [
    "The current save parser reads origin but does not yet track the Unifying Purpose situation, authority changes, or civil war outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The wiki hint says to change authority to Imperial after the
Unifying Purpose situation is over. This is an event-chain-dependent achievement
with precise timing.

---

## ethical_dilemmas — Ethical Dilemmas

Source requirement: Have 6 Council Members all with a different ethic.

```json
{
  "tags": [
    "setup-gated",
    "galactic-paragons",
    "council",
    "diplomacy"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Galactic Paragons",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Galactic Paragons DLC."
    }
  ],
  "warnings": [
    "Requires significant leader diversity and migration treaties to recruit leaders with different ethics.",
    "Some ethics may not generate council members naturally without specific civics."
  ],
  "planner_notes": "Pick civics that unlock council positions for multiple leader types. Establish migration treaties to recruit from diverse pops.",
  "known_limitations": [
    "The current save parser does not yet detect individual council member ethics."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: No existing dimension covers "council member ethics count."
This is a situational requirement that depends on having a large enough council
and enough ethic diversity among the empire's population. Rule_confidence is low
because the setup requirements are indirect and RNG-dependent.

---

## growing_like_weeds — Growing Like Weeds

Source requirement: As a Plantoid species that does not have the Clone Army origin, have 25,000 pops with the Budding trait on your capital.

```json
{
  "tags": [
    "setup-gated",
    "species-gated",
    "species-trait",
    "population",
    "plantoid"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "species_class",
      "operator": "equals",
      "value": "Plantoid",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Founding species must be Plantoid."
    },
    {
      "condition_type": "required",
      "dimension": "species_trait",
      "operator": "contains",
      "value": "trait_budding",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Founding species must have the Budding trait."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Plantoids",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Plantoids DLC is required for the Budding trait and this achievement."
    }
  ],
  "warnings": [
    "Clone Army origin blocks this achievement (explicitly excluded by the wiki requirement).",
    "Requires 25,000 Budding pops on a single planet (the capital). Extremely high threshold."
  ],
  "planner_notes": "Maximize pop growth on capital. Avoid Clone Army origin. The Budding trait provides passive pop assembly; combine with high habitability and growth bonuses.",
  "known_limitations": [
    "The current save parser does not yet detect species traits or count pops by trait per planet."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The explicit exclusion of Clone Army origin is captured in
the warnings but not as a negative condition. The dimension set does not have an
`origin_not_equals` operator.

---

## non_prophet_organization — Non-Prophet Organization

Source requirement: While playing with the Teachers of the Shroud origin, destroy the Shroudwalker enclave you start the game in contact with.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "shroud",
    "enclave",
    "combat"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_teachers_of_the_shroud",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Teachers of the Shroud origin."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Teachers of the Shroud origin requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "enclave_interaction_type",
      "operator": "equals",
      "value": "destroy_enclave",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must destroy the Shroudwalker enclave that spawns at the start."
    }
  ],
  "warnings": [
    "Destroying the Shroudwalker enclave may have diplomatic or gameplay consequences."
  ],
  "planner_notes": "Build about 15k fleet power and destroy the Shroudwalker enclave early. This is straightforward with a reasonable fleet.",
  "known_limitations": [
    "The current save parser reads origin but does not yet track enclave destruction events."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None — the wiki requirement is straightforward.

---

## underlord — Underlord

Source requirement: Clear the Unexpected Mineral Seams event as an empire with the Subterranean origin.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "story",
    "event-limited"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_subterranean",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Subterranean origin."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Subterranean origin requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "special_project_completed_type",
      "operator": "equals",
      "value": "unexpected_mineral_seams",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must complete the Unexpected Mineral Seams event chain successfully."
    }
  ],
  "warnings": [
    "Event chain must be completed without failure (bad choices can lock progress)."
  ],
  "planner_notes": "Play as Subterranean origin and follow the Unexpected Mineral Seams event chain carefully, choosing options that lead to successful clearance.",
  "known_limitations": [
    "The current save parser reads origin but does not yet track completion of named special projects or event chains."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: `special_project_completed_type` is in DIMENSIONS.md but is
a forward-looking dimension that the parser cannot yet evaluate.

---

## starlit_invader — I am the Invader Now

Source requirement: Control 5 systems connected to the Abandoned Hatchery invaders system as an empire with the Starlit Citadel origin.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "conquest",
    "story"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_starlit_citadel",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Starlit Citadel origin."
    },
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
    }
  ],
  "warnings": [
    "Requires controlling systems on the Strange Portal network, which depends on galaxy generation."
  ],
  "planner_notes": "The empire capital and Abandoned Hatchery both count. Conquer 3 more systems connected to the portal network to reach 5.",
  "known_limitations": [
    "The current save parser reads origin but does not yet track Strange Portal network system ownership."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: No existing dimension captures "Strange Portal network system
control." This is a unique mechanic from the BioGenesis DLC. Rule_confidence is
low because the condition is mechanically unique and galaxy-generation-dependent.

---

## smorgasblorg — Smörgåsblorg

Source requirement: Obtain DNA from six phenotypes as an empire with the Evolutionary Predators origin.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "species-management",
    "genetic"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_evolutionary_predators",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Evolutionary Predators origin."
    },
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
      "dimension": "species_dna_phenotypes_collected",
      "operator": "at_least",
      "value": 6,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must collect DNA from 6 different species phenotypes via the Adaptive Evolution situation."
    }
  ],
  "warnings": [
    "Requires enough diverse species in the galaxy to collect 6 different phenotypes."
  ],
  "planner_notes": "Each time the Adaptive Evolution situation completes, choose a different species for the DNA template.",
  "known_limitations": [
    "The current save parser reads origin but does not yet track phenotype collection count."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: `species_dna_phenotypes_collected` is a forward-looking
dimension from DIMENSIONS.md. The mechanic is specific to the BioGenesis DLC's
Evolutionary Predators origin.

---

## born_to_be_wild — Born to be Wild

Source requirement: Capture and then terraform all Shattered Fragments worlds while playing as an empire with the Wilderness origin.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "terraforming",
    "conquest",
    "story"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_wilderness",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Wilderness origin."
    },
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
    }
  ],
  "warnings": [
    "Hive Fallen Empire (Shattered Fragments worlds) must be present in the galaxy.",
    "All Shattered Fragments worlds must be captured and terraformed back to natural state."
  ],
  "planner_notes": "Use the zero-empires trick to avoid interference. Set end-game year as late as possible. Conquer the Hive FE and terraform their worlds.",
  "known_limitations": [
    "The current save parser reads origin but does not yet track terraforming of specific world types or Shattered Fragments count."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: No existing dimension captures "Shattered Fragments worlds"
or the terraforming-back-to-natural-state mechanic. This is a unique requirement
from the BioGenesis DLC.

---

## flesh_adapts — Flesh Adapts

Source requirement: Defeat the Prethoryn Scourge as an empire with a biological shipset.

```json
{
  "tags": [
    "setup-gated",
    "civic-gated",
    "prethoryn",
    "crisis",
    "late-game"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_beastmasters",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires Beastmasters civic (or other means of biological shipset — verification needed)."
    },
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
      "dimension": "endgame_crisis",
      "operator": "equals",
      "value": "prethoryn",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Prethoryn Scourge crisis."
    }
  ],
  "warnings": [
    "Requires the endgame crisis to be the Prethoryn Scourge — force in galaxy settings if possible.",
    "Biological shipset access may require specific civics or origin choices."
  ],
  "planner_notes": "Force Prethoryn as the endgame crisis. Build a biological shipset fleet. Set crisis strength to 0.25x for easier defeat.",
  "known_limitations": [
    "The current save parser reads civics but does not yet identify crisis type or biological shipset usage."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: It is not fully clear from the wiki which civics/origins
confer a "biological shipset." The Beastmasters civic (Grand Archive) grants
organic ships, but the BioGenesis DLC may have additional paths. Rule_confidence
is low because the exact "biological shipset" game mechanic is not precisely
defined in the wiki requirement.

---

## virtual_reality — Virtual Reality

Source requirement: Complete the Virtuality tradition tree and have every pop in the empire have the Virtual trait.

```json
{
  "tags": [
    "setup-gated",
    "ascension-path",
    "virtual",
    "machine",
    "population"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ascension_path",
      "operator": "equals",
      "value": "virtual",
      "timing": "terminal",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must complete the Virtuality tradition tree."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "The Machine Age",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires The Machine Age DLC."
    }
  ],
  "warnings": [
    "If the founder species has multiple templates, only one template gets the Virtual trait. Must ensure all pops share the same template.",
    "Gestalt Machine Intelligence authority likely required (Virtuality is a Machine tradition)."
  ],
  "planner_notes": "Play as a Machine Intelligence and pursue the Virtuality tradition tree. Before completing it, ensure the founder species has only one template so the Virtual trait applies galaxy-wide.",
  "known_limitations": [
    "The current save parser does not yet detect completed tradition trees or per-pop trait distributions."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The wiki hint about multiple templates is a known pitfall.
The achievement does not explicitly require Machine Intelligence authority in the
description, but Virtuality is a Machine tradition tree, so it is strongly
implied.

---

## footsteps_of_the_prophet — Footsteps of the Prophet

Source requirement: Defeat the Synthetic Queen crisis while playing as an empire with the Cybernetic Creed origin.

```json
{
  "tags": [
    "setup-gated",
    "origin-gated",
    "crisis",
    "machine-age",
    "late-game"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_cybernetic_creed",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Cybernetic Creed origin."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "The Machine Age",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires The Machine Age DLC."
    },
    {
      "condition_type": "required",
      "dimension": "endgame_crisis",
      "operator": "equals",
      "value": "cetana",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Cetana (Synthetic Queen) endgame crisis."
    }
  ],
  "warnings": [
    "Cetana must appear as the endgame crisis. This is guaranteed in The Machine Age if no other crisis is forced."
  ],
  "planner_notes": "Play as Cybernetic Creed origin. Cetana is the default crisis for The Machine Age content. Lower crisis strength to 0.25x for an easier fight.",
  "known_limitations": [
    "The current save parser reads origin but does not yet identify Cetana as the active crisis or track her defeat."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Cetana (Synthetic Queen) is the crisis introduced in The
Machine Age. The wiki says she appears automatically if no other crisis is
forced, but this is a forward-looking assumption.

---

## a_universe_of_paperclips — A Universe of Paperclips

Source requirement: Win the game through the Cosmogenesis crisis path as a machine empire with the Obsessional Directive civic, while never failing to meet the quota.

```json
{
  "tags": [
    "setup-gated",
    "civic-gated",
    "crisis-path",
    "machine",
    "endgame"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_obsessional_directive",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Obsessional Directive civic."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "The Machine Age",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires The Machine Age DLC."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_path_cosmogenesis",
      "operator": "equals",
      "value": true,
      "timing": "terminal",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must complete the Cosmogenesis crisis path."
    }
  ],
  "warnings": [
    "Never failing the Obsessional Directive quota is the hardest constraint — missed quotas invalidate the run.",
    "Requires Machine Intelligence authority for the civic."
  ],
  "planner_notes": "Set Tech/Tradition Cost to minimum. Focus heavily on meeting the quota every cycle. Cosmogenesis path progresses the win condition.",
  "known_limitations": [
    "The current save parser reads civics but does not yet track Obsessional Directive quota compliance or the Cosmogenesis situation."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The "never fail to meet the quota" constraint is unique to
the Obsessional Directive civic's situation. No existing dimension tracks quota
compliance history.

---

## toxic_workplace — Toxic Workplace

Source requirement: As an empire whose founding species has the Toxoid archetype, you must be Galactic Custodian or Galactic Emperor, and every other member of the Galactic Community must have the Insulted relations modifier from you simultaneously.

```json
{
  "tags": [
    "setup-gated",
    "species-gated",
    "toxoid",
    "galactic-community",
    "galactic-custodian"
  ],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "species_class",
      "operator": "equals",
      "value": "Toxoid",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Founding species must have the Toxoid archetype."
    },
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Toxoids",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Toxoids DLC."
    }
  ],
  "warnings": [
    "Requires becoming Galactic Custodian or Emperor first.",
    "Must insult every other Galactic Community member — and they all must have the Insulted modifier simultaneously.",
    "At least 3 other members must exist."
  ],
  "planner_notes": "Become Custodian first (easier than Emperor). The Galactic Community trick (tiny community) makes insulting everyone manageable.",
  "known_limitations": [
    "The current save parser does not yet detect species class Toxoid, Galactic Custodian/Emperor status, or diplomatic relation modifiers."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The Toxoid species class dimension is listed in DIMENSIONS.md.
The Insulted modifier condition is not covered by any existing dimension. A new
dimension like `galactic_custodian_action` with value `"insult_all_members"`
exists in DIMENSIONS.md but does not cover the "all members have the modifier"
simultaneity constraint.
