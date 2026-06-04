# Batch E Curation Draft — Diplomacy, Politics, Federation, Vassals, Galactic Community, Espionage, Enclaves

**Review status**: Draft for human review. Each entry provides a replacement `curation` block only.  
**Scope**: 29 achievements covering diplomacy, politics, federations, subjects/vassals, Galactic Community/Imperium, espionage, and enclaves.  
**Date**: 2026-06-03  
**Style reference**: `catalog/CURATION_STYLE.md`  
**Dimensions reference**: `catalog/DIMENSIONS.md`  
**Tags reference**: `catalog/TAGS.md`

---

## empire — Battle Thralls

Source requirement: Have 3 other Empires as vassals.

```json
{
  "tags": ["vassal", "subject", "diplomacy", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "vassal_count",
      "operator": "greater_than",
      "value": 2,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires at least 3 subject empires (tributaries, vassals, protectorates, or specialized subjects)."
    }
  ],
  "warnings": [],
  "planner_notes": "Releasing sectors as vassals is faster than forceful vassalization through war. The Domination tradition tree helps.",
  "known_limitations": [
    "The current save parser does not yet count vassalized empires."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. The requirement is unambiguous.

---

## unification — Birth of a Federation

Source requirement: Be the leader of a federation.

```json
{
  "tags": ["federation", "diplomacy", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "federation_formed",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires forming or joining a federation and becoming the federation president."
    }
  ],
  "warnings": [],
  "planner_notes": "Improve relations with nearby empires and propose a federation. The Common Ground origin guarantees a federation at game start.",
  "known_limitations": [
    "The current save parser does not yet detect federation membership or leadership."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## league_of_nations — League of Nations

Source requirement: Choose to support the Galactic Community when it is first proposed.

```json
{
  "tags": ["galactic-community", "diplomacy", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "galactic_community_founding_member",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires voting to support the Galactic Community when it is first proposed. If passed without your support or rejected, this cannot be obtained."
    }
  ],
  "warnings": [
    "If the initial GC proposal passes without the player empire as a supporter, the achievement is permanently missed."
  ],
  "planner_notes": "Common Ground and Hegemon origins guarantee this achievement unless another empire with the same origin proposes it first.",
  "known_limitations": [
    "The current save parser does not yet detect Galactic Community founding membership."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## opposites_attract — Opposites Attract

Source requirement: Have empires representing every ethic in your Federation.

```json
{
  "tags": ["federation", "diplomacy", "mid-game", "long-duration"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "federation_member_ethics",
      "operator": "at_least",
      "value": 8,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires all eight ethics (fanatic variants count as their base) present across federation members."
    }
  ],
  "warnings": [
    "Requires finding or developing empires that collectively cover all eight ethics; the player may need to shift ethics late-game."
  ],
  "planner_notes": "Build a large, diverse federation. If short one ethic, the player's empire can shift ethics using faction influence or the Reform Government agenda.",
  "known_limitations": [
    "The current save parser does not yet track federation member ethics."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because the requirement depends on federation composition, which changes over time and involves RNG in which ethics AI empires adopt.

---

## old_friends — Old Friends

Source requirement: Accept a gift from a patronising Enigmatic Observer or Keepers of Knowledge Fallen Empire.

```json
{
  "tags": ["diplomacy", "fallen-empire", "event-limited", "story", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "fallen_empire_gift_accepted",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires completing requests or tasks for an Enigmatic Observer or Keepers of Knowledge Fallen Empire until a gift is bestowed."
    }
  ],
  "warnings": [
    "Only Enigmatic Observer (xenophile) and Keepers of Knowledge (materialist) Fallen Empires give gifts; other types (xenophobe/spiritualist) do not.",
    "Gift events are not guaranteed in every interaction chain."
  ],
  "planner_notes": "Increase Fallen Empire count in galaxy setup to improve the odds of a gift-giving type spawning. Complete their requests as they appear.",
  "known_limitations": [
    "The current save parser does not yet track Fallen Empire interaction outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because the gift event depends on which Fallen Empire types spawn and their specific task chains. The `fallen_empire_gift_accepted` dimension is a new addition not yet in the canonical DIMENSIONS.md list.

---

## omniculture — Omnicultural

Source requirement: The ruler of your empire is different from your founding species.

```json
{
  "tags": ["diplomacy", "species-management", "politics", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ruler_species_different_from_founding",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires a ruler whose species class differs from the empire's founding species. Oligarchic and Corporate authorities cycle rulers frequently."
    }
  ],
  "warnings": [
    "Dictatorial and Imperial authorities only change rulers on death, making this harder."
  ],
  "planner_notes": "Choose Oligarchic or Corporate authority for frequent ruler turnover. The manual ruler selection costs 2000 Unity.",
  "known_limitations": [
    "The current save parser can read the current ruler species but does not yet compare it against the founding species."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The `ruler_species_different_from_founding` dimension is a new addition not yet in the canonical DIMENSIONS.md list. An alternative framing would compare `ruler_species_class` against `species_class` with a not-equals operator, but the existing operator set (equals/contains/at_least/greater_than) does not support inequality directly.

---

## victorious — Victorious

Source requirement: Finish the game in Ironman mode via one of the victory conditions.

```json
{
  "tags": ["late-game", "endgame", "long-duration"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "victory_condition_reached",
      "operator": "equals",
      "value": true,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires reaching a victory year score victory (or another victory condition such as Crisis ascension) in an Ironman game."
    }
  ],
  "warnings": [
    "Ironman mode is required; non-Ironman games do not qualify."
  ],
  "planner_notes": "Set mid-game, end-game, and victory years to their minimum values for the fastest completion. The score victory is the most common path.",
  "known_limitations": [
    "The current save parser does not yet detect victory conditions or game-over state."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The `victory_condition_reached` dimension is a new addition. The requirement is unambiguous but may interact with other victory paths from DLCs (e.g., Become the Crisis, Cosmogenesis).

---

## unlimited_power — Unlimited Power!

Source requirement: Use the active effect of a Relic by spending Unity or other resources.

```json
{
  "tags": ["relic", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "relic_active_effect_used",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires activating the active ability of any relic (costs Unity or other resources)."
    }
  ],
  "warnings": [],
  "planner_notes": "Start with the Treasure Hunters origin for a guaranteed early relic, or obtain relics through normal gameplay.",
  "known_limitations": [
    "The current save parser does not yet track relic active-effect usage."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## to_the_next_level — Let Us Go Forward Together

Source requirement: Level up a Federation you are in.

```json
{
  "tags": ["federation", "diplomacy", "early-game", "federations", "dlc-gated"],
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
      "dimension": "federation_level",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires any federation level-up from level 1 to level 2 or higher."
    }
  ],
  "warnings": [],
  "planner_notes": "The Common Ground or Hegemon origin starts you in a federation, making the level-up much easier. Federation XP accumulates from member contributions and presidency actions.",
  "known_limitations": [
    "The current save parser does not yet detect federation level."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## throw_your_weight_around — Throw Your Weight Around

Source requirement: Have your Diplomatic Weight > 8999.

```json
{
  "tags": ["diplomacy", "galactic-community", "mid-game", "federations", "dlc-gated"],
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
      "dimension": "diplomatic_weight",
      "operator": "greater_than",
      "value": 8999,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires diplomatic weight exceeding 8999."
    }
  ],
  "warnings": [],
  "planner_notes": "Diplomatic weight scales with pops, fleet power, economy, and technology. This is typically achieved naturally by the mid-to-late game in a standard playthrough.",
  "known_limitations": [
    "The current save parser does not yet calculate diplomatic weight."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## we_are_number_one — We're Number One

Source requirement: Be the leader of a Level 5 Federation.

```json
{
  "tags": ["federation", "diplomacy", "late-game", "federations", "dlc-gated"],
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
      "dimension": "federation_level",
      "operator": "at_least",
      "value": 5,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires reaching federation level 5 while the player empire is federation president."
    }
  ],
  "warnings": [
    "If the player loses federation presidency before level 5, another empire could become president and claim the level-up."
  ],
  "planner_notes": "The Common Ground or Hegemon origin makes this easier. Focus on federation XP contributions through envoys, holding the presidency, and completing federation resolutions.",
  "known_limitations": [
    "The current save parser does not yet detect federation level."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## whatever_it_is_im_against_it — Whatever it is, I'm against it

Source requirement: Reject the creation of the Galactic Community and leave a Federation.

```json
{
  "tags": ["diplomacy", "galactic-community", "federation", "timing-window", "federations", "dlc-gated"],
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
      "dimension": "galactic_community_founding_member",
      "operator": "equals",
      "value": false,
      "timing": "event_limited",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Galactic Community to be created without the player empire as a supporting member (either vote against or abstain)."
    },
    {
      "condition_type": "required",
      "dimension": "federation_formed",
      "operator": "equals",
      "value": false,
      "timing": "event_limited",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires leaving any federation the player empire belongs to."
    }
  ],
  "warnings": [
    "Must be done early when the GC is first proposed. If the player supports the GC, the achievement is permanently missed."
  ],
  "planner_notes": "Start with the Common Ground or Hegemon origin to ensure a federation exists to leave. When the GC is first proposed, vote against it or abstain. Then leave the federation.",
  "known_limitations": [
    "The current save parser does not yet detect GC founding decisions or federation departure."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## see_you_in_court — You've Been Served

Source requirement: Pass any Denounce resolution on an Empire that isn't in breach of Galactic Law while all major sanctions are active.

```json
{
  "tags": ["galactic-community", "diplomacy", "political", "timing-window", "mid-game", "federations", "dlc-gated"],
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
      "dimension": "galactic_community_exists",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Galactic Community must exist."
    },
    {
      "condition_type": "required",
      "dimension": "denounce_resolution_passed_all_sanctions",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires passing a Denounce resolution against an empire that is not actually in breach of galactic law, while all four major sanctions resolutions are active in the GC."
    }
  ],
  "warnings": [
    "Requires significant diplomatic weight to pass specific GC resolutions.",
    "All four major sanctions must be passed before the denounce can be used on a non-breaching empire."
  ],
  "planner_notes": "Build enough diplomatic weight to control GC outcomes. Pass all major sanctions first, then target a neutral empire with a Denounce resolution.",
  "known_limitations": [
    "The current save parser does not yet track which GC resolutions have been passed."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: Low confidence because this achievement requires navigating complex GC resolution mechanics. The `denounce_resolution_passed_all_sanctions` dimension is a new addition and oversimplifies a multi-step GC process. The exact requirement — "all four major sanctions" (economic, trade, territorial, military?) — depends on game version and may need refinement.

---

## burn_notice — Burn Notice

Source requirement: When an operation fails, disavow an Asset instead of trying to re-establish contact.

```json
{
  "tags": ["espionage", "nemesis", "dlc-gated", "early-game"],
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
      "dimension": "espionage_asset_disavowed",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires an espionage operation to fail and the player to choose the 'disavow' option for the exposed asset."
    }
  ],
  "warnings": [
    "Can only occur when an espionage operation fails and the asset is exposed."
  ],
  "planner_notes": "Run espionage operations at low intel or with minimal asset upgrades to increase the chance of failure. Have infiltrated assets available.",
  "known_limitations": [
    "The current save parser does not yet track espionage operation outcomes or asset disavowal."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because the opportunity depends on RNG (operation failure + asset exposure), and the player must deliberately choose to disavow.

---

## with_thunderous_applause — With Thunderous Applause

Source requirement: Pass the Proclaim Galactic Imperium resolution with you as Galactic Custodian.

```json
{
  "tags": ["galactic-emperor", "galactic-community", "diplomacy", "late-game", "nemesis", "dlc-gated"],
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
      "timing": "event_limited",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must first become the Galactic Custodian."
    },
    {
      "condition_type": "required",
      "dimension": "galactic_emperor",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must pass the Proclaim Galactic Imperium resolution while Custodian, becoming Galactic Emperor."
    }
  ],
  "warnings": [
    "Requires sufficient diplomatic weight to pass the Custodian and Imperium resolutions.",
    "AI empires may block progression if the player does not have enough GC support."
  ],
  "planner_notes": "Become Custodian during or after a crisis for easier support. Build diplomatic weight through fleet power, economy, and envoys.",
  "known_limitations": [
    "The current save parser does not yet detect Custodian or Emperor status."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because the path from Custodian to Emperor requires GC resolution mechanics that are complex and version-sensitive.

---

## tinker_tailor_soldier_blorg — Tinker, Tailor, Soldier, Blorg

Source requirement: Successfully complete 10 different Espionage Operations during one playthrough.

```json
{
  "tags": ["espionage", "late-game", "nemesis", "dlc-gated"],
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
      "dimension": "espionage_operations_completed",
      "operator": "at_least",
      "value": 10,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires completing at least 10 distinct espionage operation types in a single playthrough."
    }
  ],
  "warnings": [
    "Only 9 operation types are available at game start. Additional operations must be unlocked through technology, civics, or circumstances."
  ],
  "planner_notes": "Focus on building intel networks and infiltrations early. Unlock additional operations through espionage-related technologies. Run operations across multiple empires simultaneously.",
  "known_limitations": [
    "The current save parser does not yet track completed espionage operations."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because it requires tracking distinct operation types, which depend on tech unlocks and target suitability.

---

## fine_print — Fine Print

Source requirement: Successfully modify the vassalization agreement of a subject empire.

```json
{
  "tags": ["vassal", "subject", "diplomacy", "overlord", "dlc-gated", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "subject_contract_modified",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires successfully modifying any term in a subject empire's vassal contract."
    }
  ],
  "warnings": [],
  "planner_notes": "Release a sector as a vassal, then immediately modify one term in their contract. No approval needed for basic modifications on released vassals.",
  "known_limitations": [
    "The current save parser does not yet detect vassal contract modifications."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## gotta_subject_them_all — Gotta Subjugate Them All

Source requirement: Be the overlord of a level 3 Bulwark, Prospectorium and Scholarium simultaneously.

```json
{
  "tags": ["vassal", "subject", "diplomacy", "overlord", "dlc-gated", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "subject_type",
      "operator": "contains",
      "value": "bulwark",
      "timing": "terminal",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have at least one Bulwark subject at specialization level 3."
    },
    {
      "condition_type": "required",
      "dimension": "subject_type",
      "operator": "contains",
      "value": "prospectorium",
      "timing": "terminal",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have at least one Prospectorium subject at specialization level 3."
    },
    {
      "condition_type": "required",
      "dimension": "subject_type",
      "operator": "contains",
      "value": "scholarium",
      "timing": "terminal",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Must have at least one Scholarium subject at specialization level 3."
    }
  ],
  "warnings": [
    "Subject specialization levels increase slowly over time based on contract terms and subject development."
  ],
  "planner_notes": "Release three vassals from sectors. Set each to a different specialization (Bulwark, Prospectorium, Scholarium) in their contract. Wait for them to reach tier 3.",
  "known_limitations": [
    "The current save parser does not yet detect subject types or specialization levels."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because the `subject_type` dimension uses `contains` with a string value, but the rule engine needs to distinguish between all three types simultaneously at level 3. The dimension scheme may need refinement if the level requirement cannot be expressed in a single `contains` check.

---

## into_the_unknown — Into the Unknown

Source requirement: Use a Quantum Catapult to send 10 crewed science ships into unsurveyed regions.

```json
{
  "tags": ["exploration", "overlord", "dlc-gated", "origin-gated", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_slingshot_to_the_stars",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Slingshot to the Stars origin."
    },
    {
      "condition_type": "required",
      "dimension": "quantum_catapult_used",
      "operator": "at_least",
      "value": 10,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires launching 10 crewed science ships through the Quantum Catapult into unsurveyed regions."
    }
  ],
  "warnings": [
    "Avoid exploring the areas around the Quantum Catapult destination before launching, or the target system may already be surveyed."
  ],
  "planner_notes": "Start with Slingshot to the Stars origin. Repair the Quantum Catapult early. Hold off on exploring nearby systems to keep target areas unsurveyed.",
  "known_limitations": [
    "The current save parser does not yet detect Quantum Catapult usage or distinguish crewed vs. uncrewed launches."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The `quantum_catapult_used` dimension in DIMENSIONS.md is a boolean but this achievement requires a count of 10 crewed launches. A boolean is insufficient; a count value of 10 or a separate `quantum_catapult_science_ship_launches` dimension may be needed.

---

## none_shall_pass — None Shall Pass

Source requirement: Have a Fortress World with >9,999 fleet power and all required buildings and upgrades.

```json
{
  "tags": ["defense", "starbase", "military", "overlord", "dlc-gated", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "planet_designation",
      "operator": "equals",
      "value": "fortress_world",
      "timing": "terminal",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires a planet with the Fortress World designation."
    },
    {
      "condition_type": "required",
      "dimension": "orbital_ring_present",
      "operator": "equals",
      "value": true,
      "timing": "terminal",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires a fully upgraded orbital ring around the planet."
    },
    {
      "condition_type": "required",
      "dimension": "planet_shield_present",
      "operator": "equals",
      "value": true,
      "timing": "terminal",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Planetary Defense Shield building on the planet."
    },
    {
      "condition_type": "required",
      "dimension": "fleet_power",
      "operator": "greater_than",
      "value": 9999,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "The planet's total defensive fleet power must exceed 9,999."
    }
  ],
  "warnings": [],
  "planner_notes": "Build a planet with the Fortress World designation, add a fully-upgraded capital, Planetary Defense Shield, and a fully upgraded orbital ring filled with defense modules. Fortress buildings with garrison armies boost fleet power.",
  "known_limitations": [
    "The current save parser does not yet detect planet designations, orbital rings, or planetary shield buildings."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because multiple sub-conditions (fortress world, upgraded capital, orbital ring with modules, planetary shield, fleet power threshold) must all be met on a single planet. The `planet_designation`, `orbital_ring_present`, and `planet_shield_present` dimensions are new additions.

---

## surfing_the_web — Surfing the Web

Source requirement: Have >29 controlled systems with built Hyper Relays which connect to the empire's capital.

```json
{
  "tags": ["hyper-relay", "expansion", "infrastructure", "overlord", "dlc-gated", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "hyper_relay_count",
      "operator": "greater_than",
      "value": 29,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires over 29 Hyper Relays in systems owned by the empire, all connected in a network reaching the capital."
    }
  ],
  "warnings": [
    "Hyper Relays must form a continuous chain connected to the capital; isolated relays do not count."
  ],
  "planner_notes": "Build Hyper Relays in a contiguous chain of owned systems from the capital outward. Aim to control at least 30 systems in a connected network.",
  "known_limitations": [
    "The current save parser does not yet count Hyper Relays or verify connectivity to the capital."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because the `hyper_relay_count` dimension captures the count but not the capital-connectivity requirement. The rule engine would either need to treat count as a proxy or add a separate connectivity dimension.

---

## maximally_effective — Maximally Effective

Source requirement: Be the patron empire of three maximum sized Mercenary Enclaves.

```json
{
  "tags": ["mercenary-enclave", "enclave", "economy", "overlord", "dlc-gated", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "mercenary_enclaves_patroned",
      "operator": "at_least",
      "value": 3,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires being the patron of three mercenary enclaves that have reached maximum fleet size."
    }
  ],
  "warnings": [
    "Each mercenary enclave requires significant fleet investment to reach maximum size."
  ],
  "planner_notes": "Build mercenary enclaves in sectors that will be released as vassals to bypass the enclave capacity limit. Feed fleets into them until they reach maximum size.",
  "known_limitations": [
    "The current save parser does not yet detect mercenary enclave patronage or fleet size."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because the dimension captures patron count but not the "maximum size" sub-condition. The rule engine may need to distinguish patroned enclaves that have reached max fleet size.

---

## yeet_the_fleet — Yeet the Fleet

Source requirement: Use a Quantum Catapult to launch a military fleet into a hostile empire's capital system.

```json
{
  "tags": ["war", "quantum-catapult", "overlord", "dlc-gated", "origin-gated", "timing-window"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "origin",
      "operator": "equals",
      "value": "origin_slingshot_to_the_stars",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the Slingshot to the Stars origin."
    },
    {
      "condition_type": "required",
      "dimension": "quantum_catapult_used",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires using a Quantum Catapult to launch a military fleet into an enemy capital system during a war."
    }
  ],
  "warnings": [
    "Requires being at war with the target empire and the Quantum Catapult to be in range of their capital."
  ],
  "planner_notes": "Start with Slingshot to the Stars. Repair the Quantum Catapult. Declare war on an empire whose capital is within range of the catapult, then launch a fleet at their capital system.",
  "known_limitations": [
    "The current save parser does not yet detect Quantum Catapult usage or distinguish fleet types launched."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Medium confidence because the `quantum_catapult_used` dimension is a boolean in DIMENSIONS.md, but this achievement requires distinguishing a military fleet launch from a science ship launch into a specific target (enemy capital during war).

---

## meet_the_new_boss — Meet the New Boss

Source requirement: Pledge secret fealty to another empire and have them win an Allegiance War.

```json
{
  "tags": ["vassal", "diplomacy", "war", "overlord", "dlc-gated", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "dlc_required",
      "operator": "equals",
      "value": "Overlord",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "This achievement requires the Overlord DLC."
    },
    {
      "condition_type": "required",
      "dimension": "secret_fealty_pledged",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires pledging secret fealty to another empire."
    },
    {
      "condition_type": "required",
      "dimension": "allegiance_war_won_by_overlord",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires the overlord to win an Allegiance War against the player's current overlord."
    }
  ],
  "warnings": [
    "Requires another empire to declare and win an Allegiance War on your behalf — very difficult to orchestrate in single-player without the multiplayer trick.",
    "The targeted empire must be willing and able to defeat your current overlord."
  ],
  "planner_notes": "Become a subject of a weak overlord, pledge secret fealty to a stronger empire, and hope the stronger empire declares and wins an Allegiance War. The multiplayer trick (switching to another empire to declare war) can force the outcome.",
  "known_limitations": [
    "The current save parser does not yet detect secret fealty pledges or Allegiance War outcomes."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: Low confidence because this achievement is extremely difficult to orchestrate in single-player. The `allegiance_war_won_by_overlord` dimension is a new addition. The multiplayer trick is a known workaround but outside normal achievement pathing.

---

## mad_genius — Mad Genius

Source requirement: Recruit a scientist from a curator enclave after having developed sufficient opinion with them to do so.

```json
{
  "tags": ["enclave", "curator-enclave", "diplomacy", "leviathans", "dlc-gated", "early-game"],
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
      "dimension": "enclave_interaction_type",
      "operator": "contains",
      "value": "recruit_curator_scientist",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires finding a Curator enclave and recruiting a scientist from them."
    }
  ],
  "warnings": [
    "Curator enclave spawn is determined at galaxy generation."
  ],
  "planner_notes": "Explore to find the Curator enclave. Pay the opinion cost and recruit a scientist. Enclave spawn rate can be influenced by galaxy size and random generation.",
  "known_limitations": [
    "The current save parser does not yet detect enclave interactions."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## the_good_stuff — The Good Stuff

Source requirement: Purchase a strategic resource from a trader enclave.

```json
{
  "tags": ["enclave", "trader-enclave", "trade", "leviathans", "dlc-gated", "early-game"],
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
      "dimension": "enclave_interaction_type",
      "operator": "contains",
      "value": "buy_trader_resource",
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires finding a Trader enclave and purchasing any strategic resource from them."
    }
  ],
  "warnings": [
    "Trader enclave spawn is determined at galaxy generation."
  ],
  "planner_notes": "Explore to find a Trader enclave and buy any strategic resource. Trader enclaves are one of the three basic enclave types.",
  "known_limitations": [
    "The current save parser does not yet detect enclave interactions."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## return_to_dust — Return to Dust

Source requirement: Destroy an enclave station.

```json
{
  "tags": ["enclave", "war", "leviathans", "dlc-gated", "mid-game"],
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
      "dimension": "enclave_interaction_type",
      "operator": "contains",
      "value": "destroy_enclave",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires destroying any enclave station (Curator, Trader, or Artisan)."
    }
  ],
  "warnings": [
    "Destroying an enclave is a permanent aggressive action that may have diplomatic consequences."
  ],
  "planner_notes": "Build a fleet strong enough to destroy an enclave station. Any enclave type qualifies — the achievement does not distinguish between Leviathans or other DLC enclaves, but the Leviathans DLC is required.",
  "known_limitations": [
    "The current save parser does not yet detect enclave destruction."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The `enclave_interaction_type` dimension in DIMENSIONS.md lists `"destroy_enclave"`, which matches well. The wiki note says the DLC requirement is Leviathans even if some enclaves appear with other DLCs.

---

## galatron — Inscrutable Power

Source requirement: Obtain The Galatron.

```json
{
  "tags": ["galatron", "relic", "mega-corp", "dlc-gated", "rng-locked"],
  "conditions": [
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
    },
    {
      "condition_type": "required",
      "dimension": "galatron_acquired",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires obtaining the Galatron relic from Caravaneers reliquaries."
    }
  ],
  "warnings": [
    "The Galatron is a very rare reward from Caravaneers reliquaries (low single-digit percentage chance)."
  ],
  "planner_notes": "Build a strong trade-based economy to afford many reliquary purchases. Save scumming is a common strategy. Activating the Galatron gives all empires a casus belli to take it.",
  "known_limitations": [
    "The current save parser does not yet detect relic ownership, including the Galatron."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. The requirement is unambiguous.

---

## relic_robbery — Raiders of the Lost Galatron

Source requirement: Capture The Galatron from an empire with it in war.

```json
{
  "tags": ["galatron", "relic", "war", "mega-corp", "dlc-gated", "rng-locked"],
  "conditions": [
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
    },
    {
      "condition_type": "required",
      "dimension": "galatron_captured",
      "operator": "equals",
      "value": true,
      "timing": "event_limited",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires another empire to first obtain the Galatron, then capture it from them in a war."
    }
  ],
  "warnings": [
    "Chains on the Galatron RNG: another empire must first obtain it (rare), then you must defeat them in war and capture it.",
    "Activating the Galatron grants a Take Galatron casus belli to all empires."
  ],
  "planner_notes": "This is a multi-step RNG chain. First, the Galatron must be obtained by anyone. If an AI gets it, declare war with the Take Galatron casus belli. If you get it first, you can grant the CB to others and have them take it, then recapture it.",
  "known_limitations": [
    "The current save parser does not yet detect relic ownership or war outcomes involving relic capture."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. The requirement is unambiguous though extremely situational.

---

## Summary

**File created**: `catalog/curation-drafts/batch-e-diplomacy-politics.md`

**Achievements curated**: 29

**Notably uncertain entries**:

| ID | Confidence | Reason |
|---|---|---|
| `see_you_in_court` | low | Multi-step GC resolution path (pass all major sanctions, then denounce a non-breaching empire). The `denounce_resolution_passed_all_sanctions` dimension oversimplifies a complex mechanic. |
| `meet_the_new_boss` | low | Near-impossible to orchestrate in single-player without multiplayer tricks. The `allegiance_war_won_by_overlord` dimension is new and hard to evaluate. |
| `old_friends` | medium | Depends on which Fallen Empire types spawn and their specific event chains. The `fallen_empire_gift_accepted` dimension is new. |
| `opposites_attract` | medium | Requires all 8 ethics across federation members — highly dependent on AI empire ethics composition. |
| `gotta_subject_them_all` | medium | Three simultaneous subject type + level requirements. The `contains` operator on `subject_type` may need refinement for level-gated checks. |
| `none_shall_pass` | medium | Multiple sub-conditions (fortress world, orbital ring, shield, fleet power) on a single planet — new dimensions proposed. |
| `surfing_the_web` | medium | `hyper_relay_count` captures the count but not the capital-connectivity requirement. |
| `maximally_effective` | medium | The patron count dimension exists but "maximum sized" sub-condition is not directly expressed. |
| `into_the_unknown` | high but note | `quantum_catapult_used` is a boolean in DIMENSIONS.md but 10 crewed launches need a count-based dimension. |
| `tinker_tailor_soldier_blorg` | medium | Requires tracking distinct operation types — some must be unlocked through tech. |

**New dimensions proposed** (not yet in canonical DIMENSIONS.md):
- `fallen_empire_gift_accepted` (bool)
- `ruler_species_different_from_founding` (bool)
- `victory_condition_reached` (bool)
- `denounce_resolution_passed_all_sanctions` (bool)
- `espionage_asset_disavowed` (bool)
- `planet_designation` (string, e.g., `"fortress_world"`)
- `orbital_ring_present` (bool)
- `planet_shield_present` (bool)
- `allegiance_war_won_by_overlord` (bool)
