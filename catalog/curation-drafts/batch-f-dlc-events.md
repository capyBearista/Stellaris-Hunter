# Batch F — DLC-Specific Story/Event-Chain, Ascension/Covenant, Machine Age, BioGenesis, Infernals, Toxoids, Shadows of the Shroud

**Review status**: Draft for human review. Replace only the `curation` block in each achievement entry in `catalog/latest.json`.

---

## otherside — ... To The Other Side

Source requirement: Research 15 rare technologies in a single game.

```json
{
  "tags": ["rare-tech", "research", "technology", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "rare_technologies_acquired",
      "operator": "at_least",
      "value": 15,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires acquiring at least 15 rare technologies in a single playthrough."
    }
  ],
  "warnings": [
    "Rare technology draw is subject to RNG; some rare techs require specific prerequisites."
  ],
  "planner_notes": "Void Dwellers and Teachers of the Shroud origins start with a permanent rare tech research option, making progression easier. Set Tech Cost to minimum and prioritise rare-tech-draw physics and society alternatives.",
  "known_limitations": [
    "The current save parser does not yet count acquired rare technologies."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. Straightforward numeric threshold.

---

## tradition_is_everything — Tradition is Everything

Source requirement: Unlock all 42 Traditions.

```json
{
  "tags": ["traditions", "ascension-perks", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "traditions_adopted",
      "operator": "at_least",
      "value": 42,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "All 42 tradition picks must be unlocked across the 7 tradition trees and 2 ascension perk slots per tree."
    }
  ],
  "warnings": [],
  "planner_notes": "Set Tech/Tradition Cost to 0.25x in galaxy setup to significantly speed up tradition adoption. The Harmony finisher reduces tradition cost. Moderate-difficulty empire sprawl management helps.",
  "known_limitations": [
    "The current save parser does not yet count adopted traditions."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. Clear numeric progression achievement.

---





## tend_the_garden — Tend the Garden

Source requirement: Wenkwort Artem has the Wenkwort Custodian modifier, Resort World designation, Ranger Lodge building and all original blockers present.

```json
{
  "tags": ["plantoids", "dlc-gated", "civic-gated", "story", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Plantoids",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Plantoids DLC."
    },
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_environmentalist",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Environmentalist civic is required to build the Ranger Lodge building."
    }
  ],
  "warnings": [
    "Wenkwort Artem must spawn as a unique system — this is subject to galaxy generation RNG.",
    "Do not clear any of the original blockers on Wenkwort Artem.",
    "Planet must be designated as a Resort World, which prevents other development."
  ],
  "planner_notes": "Start with the Environmentalist civic. When Wenkwort Artem is discovered, complete its custodian event chain. Keep all original blockers intact. Build a Ranger Lodge. Set the planet designation to Resort World. The planet cannot be your capital.",
  "known_limitations": [
    "The current save parser cannot detect the Wenkwort Custodian modifier, planetary blocker state, or planet designation."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The achievement group is "Plantoids" but the Environmentalist civic is from the Base game (added in 2.2). The unique planet Wenkwort Artem spawns as part of the Plantoids DLC content. The condition complexity (four simultaneous state requirements on one planet) makes this hard to parse precisely.

---

## diy — Fixer Upper

Source requirement: As an Idyllic Bloom empire, own every Junk Ratlings Tomb World and terraform them into Gaia Worlds.

```json
{
  "tags": ["plantoids", "dlc-gated", "origin-gated", "story", "terraforming"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Plantoids",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Plantoids DLC."
    },
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_idyllic_bloom",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must start with the Idyllic Bloom origin."
    }
  ],
  "warnings": [
    "The Ketling Star Pack (Junk Ratlings) has only an 80% spawn chance in a Huge galaxy; may not appear at all.",
    "All Junk Ratlings tomb worlds must be owned simultaneously before terraforming.",
    "The Idyllic Bloom origin can only terraform habitable worlds into Gaia Worlds via the Bloom Blight decision."
  ],
  "planner_notes": "Play on Huge galaxy for maximum Ketling spawn chance. Locate the Ketling Star Pack, conquer or vassalize them to claim their tomb worlds. Use the Idyllic Bloom's Bloom Blight decision to terraform each Junk Ratlings world into a Gaia World.",
  "known_limitations": [
    "The current save parser cannot detect the Idyllic Bloom special terraforming decision or Ketling-specific world ownership."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The Junk Ratlings homeworlds are specific preset systems. The achievement fires when all of them are terraformed to Gaia. Precise count of "all" may vary if Ketling spawns are split.

---

## could_be_worse — Could be Worse

Source requirement: Turn a formerly Toxic World into a Tomb World.

```json
{
  "tags": ["toxoids", "dlc-gated", "origin-gated", "story", "terraforming"],
  "conditions": [
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
    },
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_knights_of_the_toxic_god",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must start with the Knights of the Toxic God origin."
    },
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_relentless_industrialists",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Relentless Industrialists civic is required to convert worlds to Tomb Worlds."
    }
  ],
  "warnings": [
    "The Knights of the Toxic God origin's starting homeworld is the Toxic World target; it must be colonised first.",
    "The Relentless Industrialists civic decision to turn a world into a Tomb World may have cooldowns."
  ],
  "planner_notes": "Start with Knights of the Toxic God origin and Relentless Industrialists civic. Colonise the Toxic God world in your home system. Use the Relentless Industrialists decision to turn it into a Tomb World.",
  "known_limitations": [
    "The current save parser cannot detect the Relentless Industrialists terraforming outcome on specific planets."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The achievement seems to target the starter Toxic God world specifically, but the wiki phrase "a formerly Toxic World" implies any colonised toxic world may work. The safe route is to use the origin's homeworld.

---

## quest_complete — Quest Complete

Source requirement: Reach the true ending of the Knights of the Toxic God story.

```json
{
  "tags": ["toxoids", "dlc-gated", "origin-gated", "story", "event-chain"],
  "conditions": [
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
    },
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_knights_of_the_toxic_god",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must start with the Knights of the Toxic God origin."
    },
    {
      "condition_type": "required",
      "dimension": "special_project_completed_type",
      "operator": "equals",
      "value": "knights_toxic_god_final",
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must complete the Toxic God situation and defeat the Toxic Entity in combat for the true ending."
    }
  ],
  "warnings": [
    "The Toxic God story chain has multiple branches; the 'true ending' specifically requires defeating the Toxic Entity.",
    "Siding with the Toxic God in the situation may lock out the true ending."
  ],
  "planner_notes": "Play Knights of the Toxic God origin. Complete the Toxic God situation in full. When the Toxic Entity appears, defeat it in space combat. This is the true ending. Do not accept the Toxic God's offer during the situation if it would skip the Entity encounter.",
  "known_limitations": [
    "The current save parser cannot detect the Knights of the Toxic God story branch outcome or entity defeat flag."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The "true ending" distinction is important — the wiki describes this as distinct from other branches of the Toxic God situation. The precise trigger condition may need parse confirmation from game state flags.

---

## recent_history — Recent History

Source requirement: As a Memorialist empire, have a Galactic Memorial on five Tomb Worlds that were not Tomb Worlds at game start.

```json
{
  "tags": ["necroids", "dlc-gated", "civic-gated", "story", "terraforming", "event-limited"],
  "conditions": [
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
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_memorialist",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have the Memorialist civic to build Galactic Memorials."
    },
    {
      "condition_type": "required",
      "dimension": "galactic_memorials_on_tomb_worlds",
      "operator": "at_least",
      "value": 5,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires Galactic Memorials on five Tomb Worlds that did not start as Tomb Worlds."
    }
  ],
  "warnings": [
    "Tomb Worlds that existed at game start do not count — they must be created during the run (via World Cracker, the Horizon Signal, or terraforming events).",
    "Without other DLCs, the Horizon Signal event chain (20% spawn chance) is the most reliable way to create Tomb Worlds.",
    "Each Tomb World requires building a separate Galactic Memorial with the Memorialist civic."
  ],
  "planner_notes": "The most reliable path requires the Horizon Signal event chain (20% spawn chance per game). With other DLCs, Colossus World Cracker or terraforming events can also create Tomb Worlds. Build empire sprawl to support five memorialized planets.",
  "known_limitations": [
    "The current save parser does not yet detect Tomb World creation history or count Galactic Memorial buildings."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: Low confidence because the primary method (Horizon Signal) has only a 20% spawn chance. Without it, the player needs World Cracker (Apocalypse DLC) or other events to create Tomb Worlds. The `galactic_memorials_on_tomb_worlds` dimension from DIMENSIONS.md matches this requirement. The achievement requires distinguishing Tomb Worlds that existed at galaxy generation from those created during play, which is a parser challenge.

---

## mother_knows_best — Mother Knows Best

Source requirement: Declare war on Cetana after complying with all of her demands.

```json
{
  "tags": ["machine-age", "dlc-gated", "crisis", "story", "event-limited"],
  "conditions": [
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
      "notes": "Cetana must be the endgame crisis that spawns."
    }
  ],
  "warnings": [
    "Cetana spawning as the crisis is RNG-dependent unless forced via galaxy settings.",
    "Complying with all demands is time-sensitive; missed demands may lock the achievement.",
    "Raiding convoys and outposts does not count as disobedience.",
    "After complying, you must then declare war on Cetana."
  ],
  "planner_notes": "Force Cetana as the endgame crisis in galaxy settings if available. Follow all of Cetana's demands without exception (raiding is permitted). After all demands are met, declare war on Cetana to complete the achievement.",
  "known_limitations": [
    "The current save parser cannot detect Cetana demand compliance state or the specific war declaration trigger."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: This achievement depends on Cetana (a specific crisis from The Machine Age DLC) spawning, complying with scripted demands, and then triggering a war — all hard to model in current dimensions. The compliance state involves multiple event flags that would need dedicated dimension names. Marked as cross-cutting because Cetana-specific achievements may appear in crisis-related batches.

---

## past_the_expiration_date — Past the Expiration Date

Source requirement: Used the Subsume World decision on a Fallen Empire homeworld until it becomes a Nanite World.

```json
{
  "tags": ["machine-age", "dlc-gated", "authority-gated", "ascension-path", "war", "late-game"],
  "conditions": [
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
      "dimension": "authority",
      "operator": "equals",
      "value": "auth_machine_intelligence",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must be a Machine Intelligence empire to access the Nanotech ascension path."
    },
    {
      "condition_type": "required",
      "dimension": "ascension_path",
      "operator": "equals",
      "value": "nanite",
      "timing": "eventual",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must complete the Nanotech ascension path to unlock the Subsume World decision."
    }
  ],
  "warnings": [
    "Fallen Empire homeworlds are heavily fortified with FE-level fleets.",
    "Nanite swarm ships are weak against Fallen Empire ships; a conventional fleet is recommended.",
    "The Subsume World decision must be used repeatedly on the FE homeworld until fully converted."
  ],
  "planner_notes": "Start as a Machine Intelligence empire. Complete the Nanotech ascension path. Build a strong conventional fleet (nanite swarm ships are ineffective against FEs). Conquer or invade a Fallen Empire's homeworld, then use the Subsume World decision until the planet becomes a Nanite World.",
  "known_limitations": [
    "The current save parser cannot detect ascension path completion or the Subsume World decision outcome."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The Subsume World decision is a multi-stage planetary decision that progresses over time. The achievement requires following it through to completion (Nanite World state). The specific FE homeworld must be colonised/owned by the player.

---



## smugglers_of_hope — Smugglers of Hope

Source requirement: Use the Smuggle Population operation to smuggle pops of the founder species from an empire where they were enslaved or being purged.

```json
{
  "tags": ["biogenesis", "dlc-gated", "espionage", "species-management", "mid-game"],
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
      "dimension": "espionage_operations_completed",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Must successfully complete at least one Smuggle Population espionage operation targeting a specific founder-species pop."
    }
  ],
  "warnings": [
    "Requires another empire to have pops of your founder species in slavery or being purged.",
    "Broken Shackles and Payback origins have guaranteed enslaved founder pops at Minamar Specialized Industries.",
    "Requires building a Spy Network on the target empire to unlock the Smuggle Population operation.",
    "If the target empire fully purges the founder pops before the operation completes, the opportunity is lost."
  ],
  "planner_notes": "Play as Broken Shackles or Payback origin for a guaranteed target. Build a Spy Network on Minamar Specialized Industries. Use the Smuggle Population operation when available. The operation extracts founder species pops from slavery.",
  "known_limitations": [
    "The current save parser cannot detect specific espionage operation outcomes or pop-slavery status at target empires."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: This achievement targets specifically the Smuggle Population operation (added in BioGenesis). The wiki specifically recommends Broken Shackles/Payback origins for guaranteed enslaved founder pops, but any situation where another empire has enslaved/purged founder pops should work.

---

## made_a_friend_today — Made a Friend Today

Source requirement: Form a Covenant.

```json
{
  "tags": ["shadows-of-shroud", "dlc-gated", "shroud", "covenant", "story"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Shadows of the Shroud",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Shadows of the Shroud DLC."
    },
    {
      "condition_type": "required",
      "dimension": "covenant_type",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Must form a Covenant with any Patron (major or minor)."
    }
  ],
  "warnings": [],
  "planner_notes": "Very easy. The Endbringers origin guarantees a Covenant with the End of the Cycle. Alternatively, any empire with psionic access can encounter minor patrons in the Shroud. Simply accept the first Covenant offer.",
  "known_limitations": [
    "The current save parser cannot detect Covenant formation or patron type."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. Clear and simple requirement.

---

## mildly_possessed — Mildly Possessed

Source requirement: Establish communications with all 8 patrons available in a game.

```json
{
  "tags": ["shadows-of-shroud", "dlc-gated", "shroud", "covenant", "rng-locked"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Shadows of the Shroud",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Shadows of the Shroud DLC."
    }
  ],
  "warnings": [
    "Not all 8 patrons may be available in a single game depending on galaxy conditions.",
    "Some patrons require specific trigger conditions to appear.",
    "Highly RNG-dependent — may require multiple attempts."
  ],
  "planner_notes": "Shroud-Forged origin that supports the Shroud starts in contact with the Animator of Clay, reducing the total to 7 remaining. The 8 patrons include both major (Eater of Worlds, Instrument of Desire, End of the Cycle) and minor patrons. Deep Shroud diving and multiple covenant offers are needed to encounter all available patrons. Because available patrons vary by game conditions, this achievement may be easier with Shroud-Forged or Endbringers origin.",
  "known_limitations": [
    "The current save parser cannot detect which patrons have been encountered or are available."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: This is the most RNG-dependent achievement in this batch. The number of patrons that can appear varies by game conditions, and it is unclear whether all 8 can appear simultaneously in a single game. The wiki statement "all 8 patrons available in a game" implies a variable set. The Shroud-Forged origin is the recommended approach.

---

## master_of_puppets — Master of Puppets

Source requirement: Instigate a Proxy War involving 5+ empires as an empire with a Secret Societies civic.

```json
{
  "tags": ["shadows-of-shroud", "dlc-gated", "civic-gated", "espionage", "war"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Shadows of the Shroud",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Shadows of the Shroud DLC."
    },
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_secret_societies",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have the Secret Societies civic."
    },
    {
      "condition_type": "required",
      "dimension": "proxy_war_count",
      "operator": "at_least",
      "value": 5,
      "timing": "current",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "The Proxy War must involve 5 or more empires simultaneously."
    }
  ],
  "warnings": [
    "Requires a dense galaxy with enough empires to draw 5+ into one Proxy War.",
    "Building Spy Networks is essential; assets and infiltration level affect the operation."
  ],
  "planner_notes": "Take the Secret Societies civic. Build Spy Networks on all neighbours. Use the Instigate Proxy War operation, targeting a region with many empires to involve as many as possible. Higher intel and assets improve success chance.",
  "known_limitations": [
    "The current save parser cannot detect Proxy War involvement counts."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Cross-cutting candidate — Proxy War mechanic from the Shadows of the Shroud DLC may also appear in espionage-focused batches.

---

## master_of_the_shroud — Master of the Shroud

Source requirement: Have the "Psionic Mastery" modifier.

```json
{
  "tags": ["shadows-of-shroud", "dlc-gated", "shroud", "ascension-path", "covenant"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Shadows of the Shroud",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Shadows of the Shroud DLC."
    },
    {
      "condition_type": "required",
      "dimension": "covenant_type",
      "operator": "equals",
      "value": "forge_own_path",
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "When offered a Covenant in the Shroud, must choose to forge your own path instead. This grants the Psionic Mastery modifier."
    }
  ],
  "warnings": [
    "Once you forge your own path, you cannot accept any Covenant in that game.",
    "Requires psionic ascension to access the Shroud and receive Covenant offers."
  ],
  "planner_notes": "Achieve psionic ascension. Enter the Shroud until offered a Covenant. Instead of accepting, choose the option to forge your own path. This grants the Psionic Mastery modifier. The Endbringers origin forces an End of the Cycle offer — declining it to forge your own path still counts.",
  "known_limitations": [
    "The current save parser cannot detect the Psionic Mastery modifier or Shroud path choices."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `forge_own_path` value for `covenant_type` is a proposed dimension value — no existing dimension covers choosing to decline a covenant. The precise trigger is having the country flag/modifier `psionic_mastery`. A dedicated dimension like `shroud_path_chosen` may be more appropriate.

---

## mind_over_metal — Mind Over Metal

Source requirement: Unlock the following technologies: Precognition Interface, Psi Jump Drives, Psionic Shields, Zro Launchers, Psionic Bombers, Psionic Disruptors, Psionic Lightning and Psi-Phase Generators.

```json
{
  "tags": ["shadows-of-shroud", "dlc-gated", "technology", "research", "psionic", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Shadows of the Shroud",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Shadows of the Shroud DLC."
    },
    {
      "condition_type": "required",
      "dimension": "psionic_techs_unlocked",
      "operator": "at_least",
      "value": 8,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires unlocking all 8 psionic ship component technologies."
    }
  ],
  "warnings": [
    "Some psionic techs require Zro as a strategic resource.",
    "Tech draw RNG may delay specific component research options.",
    "The Shroudwalkers enclave can provide missing techs via special projects every 10 years."
  ],
  "planner_notes": "Pursue psionic ascension for access to psionic techs. The Shroudwalkers (available with Shadows of the Shroud DLC) can be paid every 10 years to give a special project that unlocks a missing psionic component tech. Secure a Zro source for the later component techs.",
  "known_limitations": [
    "The current save parser cannot count unlocked psionic component technologies."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The wiki lists exactly 8 technologies. The Shroudwalkers special project provides a failsafe against missing tech RNG. The dimension `psionic_techs_unlocked` already exists in DIMENSIONS.md.

---

## this_is_the_part_where_we_kill_you — This Is the Part Where We Kill You

Source requirement: Spend 1000+ pops at once in an Experimental Testing situation.

```json
{
  "tags": ["shadows-of-shroud", "dlc-gated", "civic-gated", "pop-management", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Shadows of the Shroud",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Shadows of the Shroud DLC."
    },
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_experimental_sentencing",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have the Experimental Sentencing civic (or Judicial R&D)."
    }
  ],
  "warnings": [
    "Requires 1000+ pops to be available to spend at once — this is a very large population even for a mature empire.",
    "The Experimental Testing situation will consume the pops permanently.",
    "Pop growth speed and immigration must be maximised well before triggering."
  ],
  "planner_notes": "Take Experimental Sentencing or Judicial R&D civic. Build a very large population (1000+ pops) through conquest, pop assembly, and migration. When ready, start the Experimental Testing situation and choose the option to spend 1000+ pops. The pops are consumed, so time this when you can afford the loss.",
  "known_limitations": [
    "The current save parser cannot detect Experimental Testing situation state or pop consumption."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The achievement fires when spending "1000+ pops at once" — meaning a single situation decision consumes them. The threshold is ">999 pops" per the requirement description. The civic `judicial_r&d` (alternative name) may also unlock this situation.

---

## the_end — The End

Source requirement: Form a Covenant with the End of the Cycle and destroy the galaxy.

```json
{
  "tags": ["shadows-of-shroud", "dlc-gated", "covenant", "crisis-path", "endgame"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Shadows of the Shroud",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Shadows of the Shroud DLC."
    },
    {
      "condition_type": "required",
      "dimension": "covenant_type",
      "operator": "equals",
      "value": "end_of_the_cycle",
      "timing": "discovery",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must form a Covenant with the End of the Cycle patron."
    }
  ],
  "warnings": [
    "The End of the Cycle is one of the major patrons and may not appear in every game without the Endbringers origin.",
    "The Aura of the End must cover every system in the galaxy before the Reckoning triggers.",
    "The Reckoning event triggers on a timer and cannot be stopped once started.",
    "If the Reckoning fires before the Aura covers the galaxy, the achievement fails."
  ],
  "planner_notes": "Extremely difficult. Use the Endbringers origin for a guaranteed End of the Cycle Covenant offer. After forming the Covenant, the Aura of the End spreads from your capital. Expand it aggressively to cover every system before the Reckoning timer expires. Small galaxy sizes reduce the number of systems to cover.",
  "known_limitations": [
    "The current save parser cannot detect covenant type or Aura of the End coverage state."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The achievement is mechanically complex (galaxy-wide aura spread vs. timer). The specific trigger is having the Aura of the End country modifier cover all systems when the Reckoning happens. No existing dimension models this. May be cross-cutting with crisis-path batches.

---

## cosmic_confetti — Cosmic Confetti

Source requirement: Red Giant Expansion situation progresses completely.

```json
{
  "tags": ["infernals", "dlc-gated", "origin-gated", "story", "early-game"],
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
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_red_giant",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must start with the Red Giant origin."
    }
  ],
  "warnings": [
    "The Red Giant Expansion situation progresses automatically but takes time.",
    "Your home system is destroyed when the situation completes."
  ],
  "planner_notes": "Very easy. Start as a Red Giant origin empire. The Red Giant Expansion situation will auto-progress. Wait for it to complete and watch your home star explode. Colonise other planets before completion since your homeworld is destroyed.",
  "known_limitations": [
    "The current save parser cannot detect the Red Giant Expansion situation progress or completion."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. Simple origin-gated waiting game with a clear trigger. Mark as cross-cutting if Infernals DLC achievements are also collected in another dedicated batch.
