# Batch D — Exploration, Discovery, First Contact, Precursors, Archaeology, Anomalies, L-Cluster, Astral Rifts, Special Projects, Space Fauna, Grand Archive Discovery

**Scope**: 40 achievements from Base game, Ancient Relics, Distant Stars, Astral Planes, First Contact, Galactic Paragons, Grand Archive, and Cosmic Storms.

**Style**: Each entry provides a replacement `curation` block only. Existing `source` fields are unchanged. All conditions use `source: "wiki-reviewed"`.

---

## trade — Mutual Understanding

Source requirement: Trade with another empire.

```json
{
  "tags": ["trade", "diplomacy", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "diplomatic_weight",
      "operator": "at_least",
      "value": 0,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Any trade deal with another empire, including gifts, triggers the achievement."
    }
  ],
  "warnings": [],
  "planner_notes": "Gifts count as trade — send a small gift of resources to the first empire you meet.",
  "known_limitations": [
    "The current save parser does not yet track trade deal history."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The most straightforward achievement in this batch. No DLC required, no RNG, no timing pressure.

---

## they_come_in_pieces — They Come In Pieces

Source requirement: Vivisect an alien during First Contact.

```json
{
  "tags": ["first-contact", "pre-ftl", "early-game", "xenophobe"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "first_contact_result",
      "operator": "equals",
      "value": "aggressive",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Aggressive First Contact Protocol policy and selecting the vivisection option."
    }
  ],
  "warnings": [
    "Not available to all ethics; some pacifist or xenophile ethics block the Aggressive First Contact Protocol."
  ],
  "planner_notes": "Set empire policy to Aggressive First Contact Protocol. The vivisection option appears when investigating a pre-FTL civilization with that policy active.",
  "known_limitations": [
    "The current save parser does not yet identify First Contact protocol policies or event outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The exact ethics/civics that block Aggressive First Contact Protocol are not fully enumerated here. Wiki indicates it is restricted but the specific blockers should be confirmed.

---

## we_come_in_peace — We Come In Peace

Source requirement: Have no negative events happen to you or another empire during First Contact.

```json
{
  "tags": ["first-contact", "diplomacy", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "first_contact_result",
      "operator": "equals",
      "value": "peaceful",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires Proactive or Cautious First Contact Protocol and no negative incidents during the First Contact phase."
    }
  ],
  "warnings": [
    "Negative First Contact events are partially RNG-dependent; Aggressive Protocol is incompatible."
  ],
  "planner_notes": "Use Proactive or Cautious First Contact Protocol. Avoid hostile actions that may trigger negative incidents before contact is fully established.",
  "known_limitations": [
    "The current save parser does not yet track First Contact incident history."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The exact list of "negative incidents" is broad (science ship destruction, border incidents, etc.). Best-effort avoidance is the practical strategy.

---

## beyond_the_veil — Beyond the Veil

Source requirement: Take the Breach the Shroud tradition and complete the resulting special project.

```json
{
  "tags": ["shroud", "psionic", "ascension-path", "story", "technology"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ascension_path",
      "operator": "equals",
      "value": "psionic",
      "timing": "eventual",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the Psionic tradition tree to unlock the Breach the Shroud finisher."
    },
    {
      "condition_type": "required",
      "dimension": "special_project_completed_type",
      "operator": "equals",
      "value": "breach_the_shroud",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires completing the special project that follows the Breach the Shroud tradition."
    }
  ],
  "warnings": [
    "Requires Psionic Theory technology and the Transcendence ascension perk; spiritualist ethics make this easier."
  ],
  "planner_notes": "Start with Teachers of the Shroud origin (Overlord DLC) for a guaranteed early path to psionic theory.",
  "known_limitations": [
    "The current save parser does not yet detect completed ascension paths or special projects."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: This achievement's group is "Base game" — the psionic tradition tree and Breach the Shroud are available without any DLC. Teachers of the Shroud origin (Overlord DLC) is not required but makes it much faster.

---

## uplift — Clever Girl

Source requirement: Uplift a species.

```json
{
  "tags": ["species-management", "pre-ftl", "early-game", "technology"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "species_uplifted",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires uplifting a pre-sapient species to full sapience."
    }
  ],
  "warnings": [
    "Requires pre-sapient species in the galaxy; high galaxy settings for pre-sapients improve availability."
  ],
  "planner_notes": "Increase Pre-Sapient Species galaxy setting. The Epigenetic Triggers technology enables uplifting.",
  "known_limitations": [
    "The current save parser does not yet detect species uplift completion."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: No significant ambiguity. The requirement is straightforward — find a pre-sapient, research the tech, complete the project.

---

## controlled_evolution — Controlled Evolution

Source requirement: Modify a species which has > 7 trait points.

```json
{
  "tags": ["species-management", "genetic", "ascension-path", "technology", "rare-tech"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "species_genetically_modified",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires modifying a species with more than 7 total trait point value."
    },
    {
      "condition_type": "required",
      "dimension": "ascension_perks_unlocked",
      "operator": "contains",
      "value": "engineered_evolution",
      "timing": "eventual",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Engineered Evolution ascension perk is required to exceed 7 trait points."
    }
  ],
  "warnings": [
    "Engineered Evolution ascension perk requires completing Genetics tradition tree."
  ],
  "planner_notes": "Prioritize Genetics tradition tree to unlock Engineered Evolution, then use gene-modding on a species with existing positive traits to push past 7 points.",
  "known_limitations": [
    "The current save parser does not yet detect species trait points or genetic modification outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The Engineered Evolution ascension perk was originally Utopia-gated but is now part of the base game's Genetic tradition tree. The wiki group confirms "Base game."

---

## enlightened_times — Enlightened Times

Source requirement: Enlighten a primitive civilization from the Bronze or Stone Age.

```json
{
  "tags": ["pre-ftl", "early-game", "observation"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "pre_ftl_era_target",
      "operator": "equals",
      "value": "stone_age",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "A Bronze Age or Stone Age primitive civilization must exist as a valid target."
    },
    {
      "condition_type": "required",
      "dimension": "pre_ftl_infiltration_completed",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Enlightenment is the peaceful path — raising the civilization to the space age."
    }
  ],
  "warnings": [
    "The required primitive era may not spawn; high Pre-FTL galaxy settings improve odds."
  ],
  "planner_notes": "Set Pre-FTL Civilizations to 5x and Habitable Worlds to 0.25x to find early-era primitives faster.",
  "known_limitations": [
    "The current save parser does not yet classify pre-FTL era or track enlightenment completion."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The dimension `pre_ftl_infiltration_completed` is used for enlightenment even though it's technically not "infiltration." A separate `pre_ftl_enlightenment_completed` dimension would be more precise in a future revision.

---

## genetailor — Faster, Stronger, Better

Source requirement: Genetically modify a species.

```json
{
  "tags": ["species-management", "genetic", "technology", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "species_genetically_modified",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires applying any genetic modification template to a species."
    }
  ],
  "warnings": [
    "Requires Gene Tailoring technology (or start with Overtuned origin from Toxoids DLC)."
  ],
  "planner_notes": "Research Gene Tailoring and modify any species with at least one trait change. The Overtuned origin (Toxoids DLC) starts with the ability to gene-mod immediately.",
  "known_limitations": [
    "The current save parser does not yet detect species genetic modification."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: No significant ambiguity. Note the Overtuned origin hint is DLC-gated (Toxoids) though the achievement itself is Base game.

---

## belongs_in_museum — It belongs in a museum!

Source requirement: Own at least 1 relic.

```json
{
  "tags": ["relic", "archaeology", "exploration", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "relic_owned",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires owning at least one relic from any source."
    }
  ],
  "warnings": [],
  "planner_notes": "Completing an archaeological site or starting with the Treasure Hunters origin (Grand Archive DLC) provides a reliable relic.",
  "known_limitations": [
    "The current save parser does not yet identify owned relic count."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: Relics can come from archaeology sites (base game), precursor chains, leviathans, or the Treasure Hunters origin. The wiki group labels this "Base game" and relics are obtainable without any DLC.

---

## what_came_before — What Came Before

Source requirement: Travel to the home system of a precursor empire.

```json
{
  "tags": ["precursor", "exploration", "story", "archaeology"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "precursor_chain_completed",
      "operator": "equals",
      "value": true,
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Completing any precursor chain reveals the precursor home system."
    }
  ],
  "warnings": [
    "Precursor assignment is RNG-based; forcing a specific precursor via galaxy settings is not possible in the base game."
  ],
  "planner_notes": "Send science ships to survey many systems to find precursor archaeology sites faster.",
  "known_limitations": [
    "The current save parser does not yet detect precursor chain completion or home system travel."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The requirement is for any precursor — Cybrex, First League, Irassians, etc. All precursors spawned by the base game work.

---

## then_virgil_now_beatrice — Then Virgil, Now Beatrice

Source requirement: Complete the In Limbo anomaly either after having researched synths, or prior to accepting an Enigmatic Observers Fallen Empire's relevant request.

```json
{
  "tags": ["story", "event-limited", "exploration", "anomaly", "technology"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "special_project_completed_type",
      "operator": "equals",
      "value": "limbo",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires finding and completing the In Limbo anomaly on a specific habitable world, then choosing to revive the species."
    }
  ],
  "warnings": [
    "The In Limbo anomaly is RNG-based — it may not spawn in a given game."
  ],
  "planner_notes": "Aggressive early exploration improves the chance of finding the anomaly. The preferred path is to complete it after researching Synthetics technology.",
  "known_limitations": [
    "The current save parser does not yet detect specific anomaly completions or the In Limbo event chain."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: This achievement depends on a specific anomaly and its branching event chain. The wiki describes two valid completion paths but the anomaly itself is random. Rule confidence is low due to the double RNG (anomaly spawn + event outcome choices).

---

## tourist_trap — Tourist Trap

Source requirement: Number of different species on a planet > 9.

```json
{
  "tags": ["species-management", "migration", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "species_on_planet_count",
      "operator": "greater_than",
      "value": 9,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires at least 10 different species present on a single owned planet."
    }
  ],
  "warnings": [
    "Requires migration treaties, refugees, conquest, or the Broken Shackles origin to gather enough species."
  ],
  "planner_notes": "The Broken Shackles origin (First Contact DLC) starts with multiple species. Alternatively, use migration treaties and a high-traffic habitat to collect diverse pops.",
  "known_limitations": [
    "The current save parser does not yet count species diversity per planet."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: No DLC requirement for the achievement itself (Base game group), but the Broken Shackles origin mentioned in the wiki hint requires First Contact DLC.

---

## unboxing — Unboxing

Source requirement: Complete the project to lower the barrier on a shielded world.

```json
{
  "tags": ["exploration", "event-limited", "story", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "shielded_world_unlocked",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the special project to lower the shield on a shielded world."
    }
  ],
  "warnings": [
    "Shielded worlds only appear within the borders of the Fanatic Xenophobe Fallen Empire."
  ],
  "planner_notes": "Wait for the Fanatic Xenophobe Fallen Empire to open borders or conquer their territory to access the shielded world.",
  "known_limitations": [
    "The current save parser does not yet detect shielded world status or the barrier-lowering project."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The shielded world is guaranteed (inside FE territory) but accessing it depends on FE behavior. No DLC required for the achievement itself.

---

## what_was_will_be — What Was Will Be

Source requirement: Complete the Horizon Signal event chain and either outcome.

```json
{
  "tags": ["horizon-signal", "story", "rng-locked", "event-limited"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "horizon_signal_completed",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the Horizon Signal event chain regardless of the final outcome choice."
    }
  ],
  "warnings": [
    "Horizon Signal has only a 20% chance to spawn per game; it cannot be forced."
  ],
  "planner_notes": "If the Horizon Signal triggers, follow the chain through to completion. Both the accepting and rejecting outcomes satisfy the requirement.",
  "known_limitations": [
    "The current save parser does not yet detect Horizon Signal event flags or completion."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The 20% spawn chance makes this highly RNG-dependent, but the requirement itself (complete chain + either outcome) is unambiguous.

---

## dust_off — Dust Off

Source requirement: Finish the Zroni Precursor chain.

```json
{
  "tags": ["precursor", "archaeology", "story", "ancient-relics", "dlc-gated"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Ancient Relics",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Ancient Relics DLC."
    },
    {
      "condition_type": "required",
      "dimension": "precursor_chain_completed",
      "operator": "equals",
      "value": "zroni",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the Zroni precursor archaeology chain."
    }
  ],
  "warnings": [
    "Precursor assignment is RNG-based unless other precursors are disabled in galaxy settings."
  ],
  "planner_notes": "To force Zroni in a targeted run, enable only Ancient Relics DLC precursors and disable others in galaxy settings.",
  "known_limitations": [
    "The current save parser does not yet detect precursor identity or chain completion."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: No significant ambiguity. The Zroni is one of the two precursors added by Ancient Relics (alongside Baol). Restricting galaxy settings to force it is well-documented.

---

## green_thumb — Green Thumb

Source requirement: Finish the Baol Precursor chain.

```json
{
  "tags": ["precursor", "archaeology", "story", "ancient-relics", "dlc-gated"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Ancient Relics",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Ancient Relics DLC."
    },
    {
      "condition_type": "required",
      "dimension": "precursor_chain_completed",
      "operator": "equals",
      "value": "baol",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the Baol precursor archaeology chain."
    }
  ],
  "warnings": [
    "Precursor assignment is RNG-based unless other precursors are disabled in galaxy settings."
  ],
  "planner_notes": "To force Baol in a targeted run, enable only Ancient Relics DLC precursors and disable others in galaxy settings.",
  "known_limitations": [
    "The current save parser does not yet detect precursor identity or chain completion."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: Same pattern as Dust Off but for the Baol precursor. No significant ambiguity.

---

## l-cluster — ...and Hope?

Source requirement: Be the first to open an L-Gate.

```json
{
  "tags": ["l-cluster", "exploration", "distant-stars", "dlc-gated", "rng-locked"],
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
      "dimension": "l_cluster_unlocked",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires being the first empire in the galaxy to open an L-Gate."
    }
  ],
  "warnings": [
    "AI empires may beat the player to the first L-Gate opening — the zero empires trick avoids this."
  ],
  "planner_notes": "The zero-empires galaxy setting guarantees the player is first. Otherwise, rush L-Gate Insight technologies and survey aggressively.",
  "known_limitations": [
    "The current save parser does not yet detect L-Gate opening history."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The "first to open" condition is unambiguous but can be preempted by AI empires in a normal galaxy.

---

## amoeba — It Followed Me Home

Source requirement: Make friends with a Space Amoeba.

```json
{
  "tags": ["space-fauna", "bubbles", "distant-stars", "dlc-gated", "story", "rng-locked"],
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
      "dimension": "amoeba_companion_found",
      "operator": "equals",
      "value": true,
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Lost Amoeba event chain where an orphaned space amoeba can be befriended."
    }
  ],
  "warnings": [
    "The Lost Amoeba event is RNG-dependent — it may not spawn in every game."
  ],
  "planner_notes": "Survey many systems to increase the chance of triggering the Lost Amoeba anomaly. Choose the friendly option when the event fires.",
  "known_limitations": [
    "The current save parser does not yet detect the Lost Amoeba event or amoeba companion flag."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The event relies on an RNG-based anomaly. The friendly outcome choice is straightforward once the event appears.

---

## breaching_the_planes — Breaching the Planes

Source requirement: Explore an astral rift.

```json
{
  "tags": ["exploration", "astral-planes", "dlc-gated", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Astral Planes",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Astral Planes DLC."
    },
    {
      "condition_type": "required",
      "dimension": "astral_rifts_explored",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires entering and exploring at least one astral rift."
    }
  ],
  "warnings": [],
  "planner_notes": "The Riftworld origin (Astral Planes DLC) starts with an astral rift in the home system for immediate access.",
  "known_limitations": [
    "The current save parser does not yet track astral rift exploration counts."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: No significant ambiguity. The Riftworld origin provides a guaranteed rift, but any explored astral rift counts.

---

## growing_planes — Growing Planes

Source requirement: Explore 5 Astral Rifts.

```json
{
  "tags": ["exploration", "astral-planes", "dlc-gated", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Astral Planes",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Astral Planes DLC."
    },
    {
      "condition_type": "required",
      "dimension": "astral_rifts_explored",
      "operator": "at_least",
      "value": 5,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires exploring five distinct astral rifts in a single game."
    }
  ],
  "warnings": [
    "Astral rifts are finite; galaxy generation may limit availability."
  ],
  "planner_notes": "The Riftworld origin (Astral Planes DLC) ensures at least one rift in the home system. Active exploration with multiple science ships speeds up finding the remaining rifts.",
  "known_limitations": [
    "The current save parser does not yet track astral rift exploration counts."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: Same pattern as Breaching the Planes but with a count of 5. No significant ambiguity.

---

## put_a_cork_in_it — Put a Cork in It

Source requirement: Use the Dimensional Lock action on an owned wormhole, Gateway or L-Gate.

```json
{
  "tags": ["astral-planes", "dlc-gated", "technology", "wormhole", "gateway"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Astral Planes",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Astral Planes DLC."
    },
    {
      "condition_type": "required",
      "dimension": "wormhole_travel_completed",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires access to at least one wormhole, Gateway, or L-Gate to target with the Dimensional Lock."
    }
  ],
  "warnings": [
    "Dimensional Lock requires the Dimensional Locks astral action to be researched."
  ],
  "planner_notes": "Increase Wormhole Pairs galaxy setting for more wormholes. The Riftworld origin provides early astral rift access to help research astral actions faster.",
  "known_limitations": [
    "The current save parser does not yet detect astral actions or Dimensional Lock usage."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The exact condition dimension for "use Dimensional Lock on X" does not exist yet. The wormhole/gateway/L-Gate access condition is a reasonable proxy but not an exact match.

---

## my_name_is_ozymandias — My Name is Ozymandias

Source requirement: Obtain the Eternal Throne relic.

```json
{
  "tags": ["astral-planes", "dlc-gated", "relic", "story"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Astral Planes",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Astral Planes DLC."
    },
    {
      "condition_type": "required",
      "dimension": "relic_owned",
      "operator": "equals",
      "value": "eternal_throne",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the astral rift chain that grants the Eternal Throne relic."
    }
  ],
  "warnings": [
    "The Eternal Throne comes from a specific astral rift chain; exploration coverage is important."
  ],
  "planner_notes": "The Riftworld origin helps. Prioritize astral rift exploration to find the chain leading to the Eternal Throne.",
  "known_limitations": [
    "The current save parser does not yet detect specific relic ownership or astral rift relic sources."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The Eternal Throne is obtained through astral rifts (Astral Planes DLC mechanic), but the exact rift chain is not fully documented. Medium confidence due to RNG in finding the correct rift.

---

## returned_to_form — Returned to Form

Source requirement: Obtain the legendary paragon Zadigal.

```json
{
  "tags": ["astral-planes", "dlc-gated", "paragon", "story"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Astral Planes",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Astral Planes DLC."
    },
    {
      "condition_type": "required",
      "dimension": "legendary_paragon_recruited",
      "operator": "equals",
      "value": "zadigal",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires recruiting the legendary paragon Zadigal through astral rift exploration."
    }
  ],
  "warnings": [
    "Zadigal's availability depends on astral rift exploration outcomes."
  ],
  "planner_notes": "The Riftworld origin helps by providing immediate astral rift access. Explore all available rifts to find Zadigal.",
  "known_limitations": [
    "The current save parser does not yet detect specific legendary paragon recruitment."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Similar to other Astral Planes achievements — the paragon recruitment path through rifts has RNG elements that are hard to model precisely.

---

## insightful — Insightful

Source requirement: Research 12 insight technologies.

```json
{
  "tags": ["pre-ftl", "observation", "technology", "first-contact", "dlc-gated", "mid-game"],
  "conditions": [
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
      "dimension": "observation_station_count",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Insight technologies are earned through Observation Post study of pre-FTL civilizations."
    },
    {
      "condition_type": "required",
      "dimension": "rare_technologies_acquired",
      "operator": "at_least",
      "value": 12,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires researching 12 insight technologies over the course of a game."
    }
  ],
  "warnings": [
    "Insight techs require pre-FTL civilizations to study; low Pre-FTL density slows progress."
  ],
  "planner_notes": "Set Pre-FTL Civilizations to 5x. Build Observation Posts and use the Passive Study option to accumulate insight over time.",
  "known_limitations": [
    "The current save parser does not yet distinguish insight technologies from other rare techs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Insight technologies are a First Contact DLC mechanic. They are distinct from regular rare technologies but the parser doesn't yet distinguish them. The `rare_technologies_acquired` dimension is used as a proxy.

---

## path_not_taken — The Path Not Taken

Source requirement: Have 10 colonies before researching the Hyperspace Travel technology.

```json
{
  "tags": ["first-contact", "dlc-gated", "expansion", "colony", "timing-window"],
  "conditions": [
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
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_eager_explorers",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Eager Explorers civic (First Contact DLC) to colonize without hyperdrives."
    },
    {
      "condition_type": "required",
      "dimension": "colony_count_with_hyperspace_not_researched",
      "operator": "at_least",
      "value": 10,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must reach 10 colonies without researching Hyperspace Travel. Researching it invalidates the achievement."
    }
  ],
  "warnings": [
    "Researching Hyperspace Travel before reaching 10 colonies permanently locks the achievement."
  ],
  "planner_notes": "Pick the Eager Explorers civic. Colonize aggressively using the Slipspace Drive. Avoid researching the Hyperspace Travel technology until 10 colonies are established.",
  "known_limitations": [
    "The current save parser does not yet track colony counts or technology research states relative to achievement conditions."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The Eager Explorers civic is the only way to colonize without hyperdrives. The timing window is narrow — once Hyperspace Travel is researched, the condition is permanently locked.

---

## back_with_your_x — Back with Your X

Source requirement: With Sol as the capital system, accept the Habinte Unified Worlds' request to leave their system.

```json
{
  "tags": ["first-contact", "dlc-gated", "sol-system", "story", "event-limited"],
  "conditions": [
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
      "dimension": "sol_system_era",
      "operator": "equals",
      "value": "unified_world_government",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Pre-FTL Earth must have developed a unified world government and achieved spaceflight to trigger the Habinte chain."
    },
    {
      "condition_type": "required",
      "dimension": "primitive_earth_present",
      "operator": "equals",
      "value": true,
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Sol must be the player's capital system."
    }
  ],
  "warnings": [
    "Requires Sol as the starting system and a specific pre-FTL development path for Earth."
  ],
  "planner_notes": "Create an empire with Sol as the starting system. Set pre-FTL civilizations to high density. Follow the Habinte Unified Worlds event chain and accept their request.",
  "known_limitations": [
    "The current save parser does not yet track the Habinte Unified Worlds event chain or Sol pre-FTL development state."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: This achievement depends on a specific event chain with many branching conditions (Sol's development path, first contact outcome, player choices). Rule confidence is low because the complete set of preconditions is complex and partially undocumented.

---

## no_good_deed — No Good Deed

Source requirement: Intervene in the Propulsion Proponent Proclamation event while the Non-Interference Act is active.

```json
{
  "tags": ["first-contact", "dlc-gated", "pre-ftl", "event-limited", "galactic-community"],
  "conditions": [
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
      "dimension": "galactic_community_exists",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Non-Interference Act is a Galactic Community resolution that must be active."
    }
  ],
  "warnings": [
    "The Propulsion Proponent Proclamation event must fire, and the Non-Interference Act must be active simultaneously."
  ],
  "planner_notes": "Join the Galactic Community and support the Non-Interference Act. Maintain observation posts on pre-FTL civilizations to trigger the Propulsion Proponent event.",
  "known_limitations": [
    "The current save parser does not yet detect the Propulsion Proponent Proclamation event or pre-FTL intervention state."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: This is a multi-precondition event chain requiring Galactic Community resolutions and specific pre-FTL events to align. Low confidence due to event chain complexity and RNG.

---

## nothing_to_see_here — Nothing to See Here

Source requirement: Have a cloaked fleet in the target empire's capital system when declaring war.

```json
{
  "tags": ["first-contact", "dlc-gated", "war", "espionage", "timing-window"],
  "conditions": [
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
    "Requires cloaking technology and sufficient cloaking strength to avoid detection in the target capital system."
  ],
  "planner_notes": "Develop cloaking tech. Position a cloaked fleet in the target empire's capital system, then declare war while the fleet is still undetected inside the system.",
  "known_limitations": [
    "The current save parser does not yet detect fleet cloaking state or war declaration fleet positions."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The requirement is player-action-based rather than event-driven. The primary limitation is detection — if the enemy has sufficient detection capability, the fleet may be revealed before war is declared.

---

## keides — A Tad Too Late

Source requirement: Finish the storyline of Legendary Paragon Keides.

```json
{
  "tags": ["paragon", "story", "galactic-paragons", "dlc-gated", "exploration"],
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
      "dimension": "legendary_paragon_recruited",
      "operator": "equals",
      "value": "keides",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires finding and recruiting the legendary paragon Keides through exploration."
    }
  ],
  "warnings": [
    "Keides is found through exploration of the Dugar system — early and thorough system survey is recommended."
  ],
  "planner_notes": "Prioritize exploring all systems early to find the Dugar system where Keides starts. Keep influence available for any needed decisions.",
  "known_limitations": [
    "The current save parser does not yet detect specific legendary paragon recruitment status."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The Dugar system location is RNG-based. Once found, the Keides quest chain itself is well-defined. Medium confidence due to exploration RNG.

---

## million_souls — Dawn Of A Million Souls

Source requirement: Use the terraforming ability of Astrocreator Azaryn 3 times.

```json
{
  "tags": ["paragon", "story", "galactic-paragons", "dlc-gated"],
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
      "dimension": "legendary_paragon_recruited",
      "operator": "equals",
      "value": "azaryn",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Azaryn is a legendary paragon who will not contact Militarist or Gestalt Consciousness empires."
    }
  ],
  "warnings": [
    "Azaryn will not spawn if the player empire is Militarist or Gestalt Consciousness."
  ],
  "planner_notes": "Avoid Militarist ethics or Gestalt Consciousness authority to allow Azaryn to appear. Once recruited, use the terraforming ability on three eligible planets.",
  "known_limitations": [
    "The current save parser does not yet detect legendary paragon abilities or terraforming action counts."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The recruitment precondition (non-Militarist, non-Gestalt) is clearly documented. However, Azaryn's appearance still depends on paragon spawn RNG.

---

## animal_farm — Animal Farm

Source requirement: Reach 50 capacity on the Vivarium.

```json
{
  "tags": ["space-fauna", "vivarium", "grand-archive", "dlc-gated", "mid-game"],
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
      "dimension": "vivarium_capacity",
      "operator": "at_least",
      "value": 50,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires capturing and breeding enough space fauna to reach 50 Vivarium capacity."
    }
  ],
  "warnings": [],
  "planner_notes": "Capture space fauna using Gravitas Snares or Voidlures and allow them to breed in the Vivarium. Capacity grows with captured fauna population.",
  "known_limitations": [
    "The current save parser does not yet detect Vivarium capacity levels."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: No significant ambiguity. The Vivarium capacity count is straightforward.

---

## belongs_in_museum_oh_right — It Belongs in a... oh right

Source requirement: Fill any Collection category with active Exhibits.

```json
{
  "tags": ["grand-archive", "dlc-gated", "archaeology", "exploration"],
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
    }
  ],
  "warnings": [
    "Requires completing enough archaeological projects or events to fill a full collection category."
  ],
  "planner_notes": "The Habitable Worlds Survey event chain provides 5 Xeno Geology artifacts. Focus on completing archaeology sites and anomalies that yield collection exhibits.",
  "known_limitations": [
    "The current save parser does not yet track Grand Archive collection exhibit counts."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The exact number of exhibits needed to "fill a category" varies by category. The Xeno Geology shortcut (5 artifacts from Habitable Worlds Survey) fills one category. Medium confidence because exhibit sources are partially RNG-dependent.

---

## thats_no_asteroid — That's No Asteroid

Source requirement: Capture a Cutholoid.

```json
{
  "tags": ["space-fauna", "grand-archive", "dlc-gated", "exploration"],
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
      "dimension": "space_fauna_type_captured",
      "operator": "contains",
      "value": "cutholoid",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Cutholoids are hidden on celestial bodies and revealed when a science ship surveys them."
    }
  ],
  "warnings": [
    "Cutholoids are not guaranteed to exist in a given galaxy."
  ],
  "planner_notes": "Survey all celestial bodies with science ships to reveal hidden Cutholoids. Once found, use a Gravity Snare to capture them.",
  "known_limitations": [
    "The current save parser does not yet detect space fauna capture events."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Cutholoid spawn is RNG-based. The capture mechanic (Gravity Snare) is a Grand Archive feature. Medium confidence due to spawn RNG.

---

## void_charmer — Void Charmer

Source requirement: Build each type of Voidlure and attract each type of Space Fauna.

```json
{
  "tags": ["space-fauna", "vivarium", "grand-archive", "dlc-gated", "mid-game"],
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
    }
  ],
  "warnings": [
    "Space fauna types present in the galaxy are RNG-dependent; not all types may be available."
  ],
  "planner_notes": "Complete the Domestication tradition tree to unlock all Voidlure types. Build each Voidlure type to attract the corresponding space fauna. Large galaxies with more systems increase fauna diversity.",
  "known_limitations": [
    "The current save parser does not yet track Voidlure types built or space fauna attracted by type."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: This is one of the more complex Grand Archive achievements. The exact number of Voidlure types and corresponding fauna types varies, and fauna availability depends on galaxy generation. Low confidence because of the multi-dimensional RNG (fauna spawn × lure availability × galaxy size).

---

## infinite_creation — Infinite Creation

Source requirement: Peacefully complete the Infinity Sphere event, resulting in Gargantua transforming into Pantagruel.

```json
{
  "tags": ["leviathans", "dlc-gated", "exploration", "story", "event-limited"],
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
      "value": "infinity_sphere",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Infinity Sphere (Gargantua) leviathan must spawn in the galaxy."
    },
    {
      "condition_type": "required",
      "dimension": "special_project_completed_type",
      "operator": "equals",
      "value": "infinity_sphere_peaceful",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must agree to assist the Infinity Sphere and peacefully complete the 1800-day special project."
    }
  ],
  "warnings": [
    "The Infinity Sphere is a random leviathan spawn and is not guaranteed to appear.",
    "The peaceful completion path requires agreeing to assist the sphere, not attacking it.",
    "The 1800-day special project must be allowed to complete without interruption."
  ],
  "planner_notes": "When the Infinity Sphere is found, choose the option to assist it. Protect the science ship completing the 1800-day special project from interference.",
  "known_limitations": [
    "The current save parser does not yet detect leviathan spawns or track Infinity Sphere event completion."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The peaceful outcome (Gargantua → Pantagruel) requires the player to agree to help, not to attack. The aggressive option (attacking) does not trigger the achievement. The dimension `ancient_leviathan` is used with a proposed value `"infinity_sphere"` since this leviathan is not listed in the DIMENSIONS.md examples.

---

## wormageddon — Wormageddon

Source requirement: Destroy a Voidworm nest.

```json
{
  "tags": ["space-fauna", "grand-archive", "dlc-gated", "exploration"],
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
    }
  ],
  "warnings": [
    "Voidworm nests are found in certain black hole systems — black hole density affects availability."
  ],
  "planner_notes": "Explore black hole systems to find Voidworm nests. Destroy the nest with a fleet.",
  "known_limitations": [
    "The current save parser does not yet detect Voidworm nest destruction events."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Voidworm nests spawn in black hole systems. Their exact spawn behavior (guaranteed or RNG) is not fully documented — medium confidence pending confirmation.

---

## x_marks_the_spot — X Marks the Spot

Source requirement: Complete the 'Follow the Mysterious Chart' storyline.

```json
{
  "tags": ["grand-archive", "dlc-gated", "story", "exploration"],
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
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_treasure_hunters",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Treasure Hunters origin to trigger the 'Follow the Mysterious Chart' event chain."
    },
    {
      "condition_type": "required",
      "dimension": "special_project_completed_type",
      "operator": "equals",
      "value": "mysterious_chart",
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires completing the special project chain from the Mysterious Chart."
    }
  ],
  "warnings": [
    "Requires the Treasure Hunters origin (Grand Archive DLC)."
  ],
  "planner_notes": "Start with the Treasure Hunters origin and follow the Mysterious Chart event chain to completion. The origin provides the chart in the home system.",
  "known_limitations": [
    "The current save parser does not yet detect origins or specific special project completions."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The origin requirement is clear. The event chain length and specific choices are documented in the wiki. Medium confidence because the chain has multiple steps but they are deterministic once started.

---

## mediocre — Mediocre

Source requirement: Complete the Succumb to Tempest objective of the Storm Fever situation.

```json
{
  "tags": ["cosmic-storms", "dlc-gated", "story", "event-limited"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Cosmic Storms",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Cosmic Storms DLC."
    },
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_storm_chasers",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Storm Fever situation is triggered by the Storm Chasers origin."
    }
  ],
  "warnings": [
    "The Storm Fever situation must be allowed to run and the 'Succumb to Tempest' objective must be selected."
  ],
  "planner_notes": "Play as the Storm Chasers origin and follow the Storm Fever situation to its completion, choosing the Succumb to Tempest path.",
  "known_limitations": [
    "The current save parser does not yet detect Cosmic Storms situations or their objectives."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The achievement requires specific situational choices during the Storm Fever situation. Medium confidence because the situation is origin-triggered but the branching outcomes depend on player choices.

---

## but_at_what_cost — But at what cost

Source requirement: Kill Gurr by getting rid of it after studying it.

```json
{
  "tags": ["cosmic-storms", "dlc-gated", "exploration", "event-limited", "anomaly"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Cosmic Storms",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Cosmic Storms DLC."
    }
  ],
  "warnings": [
    "Requires finding the 'But They're Cute!' anomaly, which can only spawn on habitable planets."
  ],
  "planner_notes": "Survey habitable planets extensively to find the anomaly. Complete the study chain and choose to eliminate Gurr afterwards.",
  "known_limitations": [
    "The current save parser does not yet detect specific Cosmic Storms anomalies or Gurr-related events."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: The anomaly spawn is RNG-dependent and restricted to habitable planets. Low confidence due to the anomaly being both rare and location-restricted.

---

## humility_before_the_fall — Humility before the Fall

Source requirement: Complete the Inetian Traders' Precursor chain.

```json
{
  "tags": ["cosmic-storms", "dlc-gated", "precursor", "story"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Cosmic Storms",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Cosmic Storms DLC."
    },
    {
      "condition_type": "required",
      "dimension": "precursor_chain_completed",
      "operator": "equals",
      "value": "inetian_traders",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing the Inetian Traders precursor chain introduced in Cosmic Storms."
    }
  ],
  "warnings": [
    "Precursor assignment is RNG-based unless other precursors are disabled."
  ],
  "planner_notes": "To force Inetian Traders, disable other precursors in galaxy settings. Then survey systems to find precursor archaeology sites.",
  "known_limitations": [
    "The current save parser does not yet detect precursor identity or chain completion."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: Same pattern as Dust Off and Green Thumb. No significant ambiguity beyond the standard precursor RNG that can be mitigated with galaxy settings.

---

## unpopularity_contest — Unpopularity Contest

Source requirement: Use the The Tempest Invocator relic.

```json
{
  "tags": ["cosmic-storms", "dlc-gated", "relic", "story"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Cosmic Storms",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Cosmic Storms DLC."
    },
    {
      "condition_type": "required",
      "dimension": "precursor_chain_completed",
      "operator": "equals",
      "value": "adakaria_convention",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Tempest Invocator relic is obtained by completing the adAkkaria Convention precursor story and choosing against freeing the storm."
    },
    {
      "condition_type": "required",
      "dimension": "relic_owned",
      "operator": "equals",
      "value": "tempest_invocator",
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires owning and activating the Tempest Invocator relic."
    }
  ],
  "warnings": [
    "The adAkkaria Convention precursor must spawn. Choosing against freeing the storm is required to get the relic."
  ],
  "planner_notes": "Force the adAkkaria Convention precursor in galaxy settings. When completing the chain, choose the option that denies freeing the storm to obtain the Tempest Invocator.",
  "known_limitations": [
    "The current save parser does not yet detect specific relic ownership or the adAkkaria Convention event chain outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The relic requires both the correct precursor spawn AND the specific branching choice against freeing the storm. Medium confidence due to the branching requirement.

---

## Summary

- **Output file**: `catalog/curation-drafts/batch-d-exploration-discovery.md`
- **Total achievements curated**: 40
- **Covered DLCs**: Base game (14), Ancient Relics (2), Distant Stars (2), Astral Planes (5), First Contact (6), Galactic Paragons (2), Grand Archive (5), Cosmic Storms (4)
- **Format**: Replacement `curation` JSON blocks only, with achievement ID, name, source requirement, JSON block, and uncertainty notes

### Notably uncertain entries

| Achievement | Reason |
|---|---|
| `then_virgil_now_beatrice` | Double RNG — specific anomaly spawn + branching event outcome |
| `back_with_your_x` | Complex event chain with Sol development path, First Contact mechanics, and player choices |
| `no_good_deed` | Requires simultaneous alignment of Galactic Community resolution and a specific pre-FTL event |
| `but_at_what_cost` | Anomaly-only spawn on habitable planets, significant RNG |
| `void_charmer` | Multi-dimensional RNG across fauna types, lure types, and galaxy generation |
| `what_was_will_be` | 20% spawn chance per game with no forcing mechanism |
