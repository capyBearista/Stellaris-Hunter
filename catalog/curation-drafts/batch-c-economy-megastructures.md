# Batch C — Economy, Resource Thresholds, Megastructures, Terraforming, Infrastructure, Population Thresholds

> **Review draft.** Replace `curation` blocks in `catalog/latest.json` for the listed achievement IDs.
> Preserve all existing fields (`id`, `steam_app_id`, `steam_api_name`, `source`, etc.). Only the `curation` object should change.
> Every condition uses `source: "wiki-reviewed"` per CURATION_STYLE.md.

---

## economy_1 — Energetic

Source requirement: Have at least 1000 energy credits in storage.

```json
{
  "tags": ["economy", "resources", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "energy_stored",
      "operator": "at_least",
      "value": 1000,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires 1000 stored energy credits."
    }
  ],
  "warnings": [],
  "planner_notes": "Naturally earned during normal expansion; no special setup needed.",
  "known_limitations": [
    "The current save parser does not yet read stored energy credit amounts."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. Straightforward threshold.

---

## economy_2 — Power Overwhelming

Source requirement: Accumulate more than 4999 energy credits.

```json
{
  "tags": ["economy", "resources", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "energy_stored",
      "operator": "greater_than",
      "value": 4999,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires more than 5000 stored energy credits."
    }
  ],
  "warnings": [],
  "planner_notes": "Naturally earned as the economy scales; Dyson Sphere or trade value strategies accelerate this.",
  "known_limitations": [
    "The current save parser does not yet read stored energy credit amounts."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## resourceful — Resourceful

Source requirement: Have at least 1 income from 10 different strategic resources. Wiki hint: need >=1 income for all 11 tradable resources (achievement tooltip is out of date and says 10).

```json
{
  "tags": ["resources", "economy", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "strategic_resources_types",
      "operator": "at_least",
      "value": 10,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires monthly income >= 1 from at least 10 different strategic resource types."
    }
  ],
  "warnings": [
    "The actual in-game tooltip requirement may be out of date; the wiki states >=1 income for all 11 tradable strategic resources."
  ],
  "planner_notes": "Trade enclaves, vassal taxation, and galactic market purchases can supplement strategic resource income if deposits are scarce.",
  "known_limitations": [
    "The current save parser does not yet read individual strategic resource income values."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The wiki hint suggests the achievement may actually require all 11 tradable resources (not 10), but the tooltip says 10. The condition is set to 10 to match the requirement field; if the wiki is correct about 11, this will need adjustment.

---

## minerals_1 — Digging Deep

Source requirement: Monthly income of minerals greater than 249.

```json
{
  "tags": ["economy", "resources", "early-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "minerals_monthly",
      "operator": "greater_than",
      "value": 249,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires monthly mineral income > 249."
    }
  ],
  "warnings": [],
  "planner_notes": "Naturally reached as colonies develop mining districts; arc furnace megastructures are an alternative source in the Machine Age DLC.",
  "known_limitations": [
    "The current save parser does not yet read monthly resource income values."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## minerals_2 — The Industrial Re-Revolution

Source requirement: Monthly income of minerals greater than 999.

```json
{
  "tags": ["economy", "resources", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "minerals_monthly",
      "operator": "greater_than",
      "value": 999,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires monthly mineral income > 999."
    }
  ],
  "warnings": [],
  "planner_notes": "Heavy mineral world specialization, orbital rings with mineral processing, or an Arc Furnace megastructure can push income past 1000.",
  "known_limitations": [
    "The current save parser does not yet read monthly resource income values."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## center_of_trade — Center of Trade

Source requirement: Earn more than 999 energy credits per month from trade policy conversion. Hint: Wealth Creation needs 1000 Trade Value, all other policies need 2000.

```json
{
  "tags": ["trade", "economy", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "trade_value_monthly",
      "operator": "greater_than",
      "value": 999,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires > 999 energy credits per month from trade policy conversion. Effective trade value needed depends on the chosen trade policy: ~1000 for Wealth Creation, ~2000 for other policies."
    }
  ],
  "warnings": [
    "The trade policy in use affects the raw trade value needed; Wealth Creation (1:1 conversion) is the most efficient for this achievement."
  ],
  "planner_notes": "Build a trade-focused empire with a Commercial Pact or a Trading Federation; branch offices on the Mercantile tradition or Thrifty species trait help.",
  "known_limitations": [
    "The current save parser does not yet read trade policy or monthly energy-from-trade values."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The wiki distinguishes between Trade Value (the raw value) and energy credits from trade (the converted amount). The dimension `trade_value_monthly` is the closest match, but the achievement checks the converted energy output, not raw trade value. If the parser later reads both, separate dimensions may be needed.

---

## megapolis — Megapolis

Source requirement: Have a planet with more than 9999 pops.

```json
{
  "tags": ["colony", "expansion", "population", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "planet_pop_count",
      "operator": "greater_than",
      "value": 9999,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires a single planet with > 9999 pops."
    }
  ],
  "warnings": [
    "Requires extremely high population on a single planet; habitat or ring world segments may overflow more easily."
  ],
  "planner_notes": "A Void Dweller habitat or a ring world segment with multiple housing districts, high habitability, and pop growth buffs is the most reliable approach.",
  "known_limitations": [
    "The current save parser does not yet read per-planet population counts."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The `planet_pop_count` dimension does not exist in DIMENSIONS.md. The closest existing dimension is `total_pops` which is empire-wide. A new dimension should be added when the parser supports per-planet thresholds. The condition above uses the future dimension name `planet_pop_count`.

---

## capital — New Shining Star

Source requirement: Upgrade the homeworld's capital building to System-Capital Complex (Imperial Complex).

```json
{
  "tags": ["colony", "economy", "expansion", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "capital_building_level",
      "operator": "equals",
      "value": "imperial_complex",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires upgrading the capital building to Imperial Complex / System-Capital Complex."
    }
  ],
  "warnings": [],
  "planner_notes": "Naturally earned as the capital planet develops; no special setup required.",
  "known_limitations": [
    "The current save parser does not yet read building levels on the capital."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The dimension `capital_building_level` from DIMENSIONS.md uses the example value `"imperial_complex"` which matches the System-Capital Complex requirement.

---

## slave_to_the_system — Slave to the Systems

Source requirement: As an authoritarian empire with more than 49,999 total pops, have more than 19,999 enslaved pops.

```json
{
  "tags": ["slavery", "authoritarian", "population", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ethic",
      "operator": "contains",
      "value": "ethic_authoritarian",
      "timing": "setup",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires authoritarian ethic."
    },
    {
      "condition_type": "required",
      "dimension": "total_pops",
      "operator": "greater_than",
      "value": 49999,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires total empire population > 49,999."
    },
    {
      "condition_type": "required",
      "dimension": "enslaved_pops_count",
      "operator": "greater_than",
      "value": 19999,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires > 19,999 enslaved pops within the empire."
    }
  ],
  "warnings": [
    "Ethic drift away from authoritarian could eventually invalidate the setup requirement."
  ],
  "planner_notes": "Slaver Guilds or Indentured Assets civics help maintain a high enslaved ratio as population grows.",
  "known_limitations": [
    "The current save parser reads ethics but does not yet count enslaved pops."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. Conditions map cleanly to existing dimensions.

---

## star_struck — Star Struck

Source requirement: Have 200 starbases, including outposts.

```json
{
  "tags": ["expansion", "starbase", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "starbase_count",
      "operator": "at_least",
      "value": 200,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires owning at least 200 starbases (outposts count toward this total)."
    }
  ],
  "warnings": [
    "Each colonized system contributes one outpost; war and claims are needed to reach 200 in a standard-size galaxy."
  ],
  "planner_notes": "Expand aggressively and claim systems; habitats and gateway systems also increase the starbase count.",
  "known_limitations": [
    "The current save parser does not yet count owned starbases or outposts."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None.

---

## captive_star — Captive Star

Source requirement: Construct a complete Dyson Sphere from scratch (repairing a destroyed one does not count).

```json
{
  "tags": ["megastructures", "economy", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "dyson_sphere",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Dyson Sphere to completion. Repairing a pre-existing Dyson Sphere does not trigger this achievement."
    }
  ],
  "warnings": [
    "The achievement specifically requires new construction; repairing a damaged Dyson Sphere does not count."
  ],
  "planner_notes": "Requires Mega-Engineering technology and significant alloy/influence investment. Galactic Doorstep or a ruined Dyson Sphere discovery can help locate a suitable star.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures or distinguish new construction from repairs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The wiki strongly warns that only newly constructed Dyson Spheres qualify; repairs do not. The current `megastructure_type` dimension does not distinguish between built and repaired megastructures. A `megastructure_origin` qualifier may be needed later.

---

## i_can_see_forever — I Can See Forever

Source requirement: Construct a complete Sentry Array from scratch (repairing a destroyed one does not count).

```json
{
  "tags": ["megastructures", "exploration", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "sentry_array",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Sentry Array to completion. Repairing a pre-existing Sentry Array does not trigger the achievement."
    }
  ],
  "warnings": [
    "New construction is required; repairing a ruined Sentry Array does not count."
  ],
  "planner_notes": "Requires Mega-Engineering. The Sentry Array provides full-galaxy sensor coverage and is the cheapest of the original megastructures to build.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures or distinguish new construction from repairs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Same new-construction caveat as captive_star. The wiki is explicit that repairs do not count.

---

## imperial_highway — Imperial Highway

Source requirement: Own 4 gateways that have been activated.

```json
{
  "tags": ["gateway", "expansion", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "gateway_count",
      "operator": "at_least",
      "value": 4,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires owning 4 activated gateways."
    }
  ],
  "warnings": [
    "Gateway activation requires the Gateway Restoration technology."
  ],
  "planner_notes": "Set Abandoned Gateways to 5x and consider the Galactic Doorstep origin to maximize gateway availability. Alternatively, build new gateways once the technology is researched.",
  "known_limitations": [
    "The current save parser does not yet count owned gateways."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: None. The `gateway_count` dimension is a direct match.

---

## ringineering — Ringworld Engineers

Source requirement: Construct a complete Ring World megastructure from scratch (repairing a destroyed one does not count).

```json
{
  "tags": ["megastructures", "colony", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "ring_world",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Ring World to completion. Repairing a ruined Ring World does not trigger the achievement."
    }
  ],
  "warnings": [
    "New construction is required; repairing a ruined Ring World does not count."
  ],
  "planner_notes": "Extremely expensive in alloys and influence. The Cybrex precursor chain provides a ruined Ring World, but repairing it does not qualify — new construction is needed.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures or distinguish new construction from repairs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Same new-construction caveat. The wiki hint explicitly states "Repairing a destroyed Ringworld does not count."

---

## think_tank — Think Tank

Source requirement: Construct a complete Science Nexus from scratch (repairing a destroyed one does not count).

```json
{
  "tags": ["megastructures", "research", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "science_nexus",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Science Nexus to completion. Repairing a ruined Science Nexus does not trigger the achievement."
    }
  ],
  "warnings": [
    "New construction is required; repairing a ruined Science Nexus does not count."
  ],
  "planner_notes": "The cheapest of the original megastructures to build; still requires Mega-Engineering and substantial alloy stockpiles.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures or distinguish new construction from repairs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Same new-construction caveat.

---

## giga_engineering — Giga-Engineering

Source requirement: Have at least 4 fully operational megastructures within your borders (habitats, ring worlds, and gateways do not count). Hint: repairing destroyed megastructures also counts.

```json
{
  "tags": ["megastructures", "late-game", "endgame"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "owned_megastructures_count_excluding_habitats_ringworlds_gateways",
      "operator": "at_least",
      "value": 4,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires 4 completed megastructures excluding habitats, ring worlds, and gateways. Unlike other megastructure achievements, repairs do count toward this total."
    }
  ],
  "warnings": [
    "Habitats, ring worlds, and gateways are excluded. The count includes Dyson Spheres, Science Nexi, Sentry Arrays, and any MegaCorp/Machine Age megastructures."
  ],
  "planner_notes": "This is a late-game goal that typically requires multiple megastructure builds across the game. Prioritize cheaper megastructures (Science Nexus, Sentry Array) to reach 4 faster.",
  "known_limitations": [
    "The current save parser does not yet count completed megastructures or track per-type exclusions."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The exclusion of habitats, ring worlds, and gateways makes this one of the more complex megastructure achievements. The wiki also notes that unlike other achievements in this category, repairing destroyed megastructures *does* count. The dimension name above is a placeholder — a cleaner composite dimension like `megastructure_count_excluding` should be formalized in DIMENSIONS.md.

---

## put_a_ring_on_it — Put a Ring On It

Source requirement: Designate a Ring World segment as your empire's capital.

```json
{
  "tags": ["megastructures", "colony", "late-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "capital_planet_class",
      "operator": "equals",
      "value": "pc_ringworld_habitable",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires moving the capital to a Ring World segment. The capital designation decision on a ring world segment is what triggers the achievement."
    }
  ],
  "warnings": [
    "Relocating the capital costs influence and requires the target planet to be a fully colonised ring world segment."
  ],
  "planner_notes": "Build a new Ring World, repair a ruined one (the Cybrex chain), or capture one from a Fallen Empire, then designate a segment as the capital via the planet decisions menu.",
  "known_limitations": [
    "The current save parser does not yet detect the capital planet's class or capital designation status."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `capital_planet_class` dimension does not exist in DIMENSIONS.md. A similar pattern to `capital_building_level` should be added. The planet class for ring world segments is `pc_ringworld_habitable` in Clausewitz.

---

## terraform — Building Better Worlds

Source requirement: Terraform a planet (requires the Terrestrial Sculpting technology).

```json
{
  "tags": ["colony", "expansion", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "planet_terraformed",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires terraforming at least one planet."
    }
  ],
  "warnings": [
    "Requires the Terrestrial Sculpting technology."
  ],
  "planner_notes": "Research Terrestrial Sculpting (Society tech, appears mid-game) and terraform any colonizable planet.",
  "known_limitations": [
    "The current save parser does not yet detect terraforming completion."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The `planet_terraformed` dimension is not in DIMENSIONS.md. A dimension like `terraforming_completed` (integer) may be more appropriate. This entry uses the simplest interpretation.

---

## paradise_found — Paradise Found

Source requirement: Terraform a planet into a Gaia World. Hint: Gaia Worlds created by events also count.

```json
{
  "tags": ["colony", "ascension-perks", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "planet_type_gaia",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires creating (via terraforming or event) a Gaia world that is owned by the player."
    }
  ],
  "warnings": [
    "The World Shaper ascension perk is required for deliberate Gaia terraforming. Event-based Gaia worlds (e.g., the Baol precursor chain) also count."
  ],
  "planner_notes": "Take the World Shaper ascension perk and terraform any planet, or rely on the Baol precursor chain to create Gaia worlds via the Active Relic action.",
  "known_limitations": [
    "The current save parser does not yet count owned planets by class or detect terraforming outcomes."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `planet_type_gaia` dimension does not exist in DIMENSIONS.md. A more general `owned_planet_class_gaia_count` dimension or simply `owned_gaia_worlds` should be added. The event-based method (Baol relic) makes this feasible without the ascension perk.

---

## planet_of_the_mechs — Planet of the Mechs

Source requirement: Terraform a planet into a Machine World. Requires the Machine Worlds ascension perk (two prior ascension perks needed).

```json
{
  "tags": ["colony", "ascension-perks", "machine", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "planet_type_machine_world",
      "operator": "at_least",
      "value": 1,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires terraforming an owned planet into a Machine World."
    },
    {
      "condition_type": "required",
      "dimension": "ascension_perks_unlocked",
      "operator": "at_least",
      "value": 3,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "The Machine Worlds ascension perk requires two prior perks and is itself the third."
    }
  ],
  "warnings": [
    "Only machine empires or empires that have completed Synthetic ascension can use Machine Worlds; organic empires do not benefit from habitability on Machine Worlds."
  ],
  "planner_notes": "A Machine Intelligence empire can take the Machine Worlds perk as the third ascension perk and terraform any non-Machine World planet.",
  "known_limitations": [
    "The current save parser does not yet detect planet class changes after terraforming."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `planet_type_machine_world` dimension does not exist in DIMENSIONS.md. The Machine Worlds ascension perk requires World Shaper as a prerequisite (or another perk before it). The `ascension_perks_unlocked >= 3` condition is a proxy — the actual game check is "has the Machine Worlds perk," not a raw count.

---

## habitat_at_the_end_of_the_universe — View from the End of the World

Source requirement: Own a Habitat in a system that contains a naturally occurring black hole. Stars turned into black holes by events do not count.

```json
{
  "tags": ["habitat", "colony", "megastructures", "mid-game"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "habitat",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Habitat."
    },
    {
      "condition_type": "required",
      "dimension": "black_hole_system_habitat",
      "operator": "equals",
      "value": true,
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Habitat must be built in a system with a naturally occurring black hole. Black holes created by event chains do not count."
    }
  ],
  "warnings": [
    "The black hole must be a naturally spawned one, not created by events such as the Horizon Signal chain."
  ],
  "planner_notes": "Survey systems near your starting area to find a natural black hole, then build a Habitat there once the Habitat technology (Utopia DLC content) is available.",
  "known_limitations": [
    "The current save parser does not yet detect Habitat ownership or distinguish black hole types."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `black_hole_system_habitat` boolean dimension does not exist in DIMENSIONS.md. A combined dimension for "habitat in qualifying black hole system" should be added. The achievement's source group says "Base game" but the Habitat feature itself requires the Utopia DLC. If the group classification is revised later, a `dlc_required` condition may be appropriate.

---

## black_hole_mining — Black Hole Mining

Source requirement: Construct a complete Matter Decompressor from scratch (repairing a destroyed one does not count).

```json
{
  "tags": ["megastructures", "economy", "late-game", "mega-corp"],
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
      "notes": "This achievement requires the MegaCorp expansion."
    },
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "matter_decompressor",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Matter Decompressor to completion in a black hole system. Repairs do not count."
    }
  ],
  "warnings": [
    "Requires a black hole system within owned territory. New construction only — repairing a ruined Matter Decompressor does not count."
  ],
  "planner_notes": "Requires Mega-Engineering and the MegaCorp DLC. Build in any owned black hole system; the Matter Decompressor provides massive mineral income.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures or distinguish new construction from repairs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Same new-construction caveat as other megastructure achievements. The DLC requirement is from the MegaCorp expansion, per the source group.

---

## obscure_tastes — Obscure Tastes

Source requirement: Construct a complete Mega-Art Installation (including the Perfection stage) in a system within a nebula. Repairs do not count.

```json
{
  "tags": ["megastructures", "economy", "late-game", "mega-corp"],
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
      "notes": "This achievement requires the MegaCorp expansion."
    },
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "mega_art_installation",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Mega-Art Installation to the Perfection stage. Repairs do not count."
    },
    {
      "condition_type": "required",
      "dimension": "nebula_system_megastructure",
      "operator": "equals",
      "value": true,
      "timing": "discovery",
      "mutability": "rng_locked",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The completed Mega-Art Installation must be in a nebula system."
    }
  ],
  "warnings": [
    "Requires a nebula system within owned territory. Nebula spawn locations are determined at galaxy generation."
  ],
  "planner_notes": "Set Nebula density to maximum in galaxy settings to increase the chance of suitable systems. Build the Mega-Art Installation in an owned nebula system and upgrade it to Perfection stage.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures, distinguish stages, or identify nebula system locations."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `nebula_system_megastructure` boolean does not exist in DIMENSIONS.md. The achievement also requires the Perfection stage specifically, not just any completion. A stage-qualifier for megastructure completion may be needed later. Like other megastructure achievements, new construction is required.

---

## strategic_initiative — Strategic Initiative

Source requirement: Construct a complete Strategic Coordination Center from scratch (repairs do not count).

```json
{
  "tags": ["megastructures", "fleet", "late-game", "mega-corp"],
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
      "notes": "This achievement requires the MegaCorp expansion."
    },
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "strategic_coordination_center",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Strategic Coordination Center to completion. Repairs do not count."
    }
  ],
  "warnings": [
    "New construction only — repairing a ruined Strategic Coordination Center does not count."
  ],
  "planner_notes": "Provides naval capacity and sublight speed bonuses. Build in any owned system once Mega-Engineering is researched.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures or distinguish new construction from repairs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Same new-construction caveat.

---

## united_space — United Space

Source requirement: Construct a complete Interstellar Assembly from scratch (repairs do not count).

```json
{
  "tags": ["megastructures", "diplomacy", "late-game", "mega-corp"],
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
      "notes": "This achievement requires the MegaCorp expansion."
    },
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "interstellar_assembly",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building an Interstellar Assembly to completion. Repairs do not count."
    }
  ],
  "warnings": [
    "New construction only — repairing a ruined Interstellar Assembly does not count."
  ],
  "planner_notes": "Boosts diplomatic weight and migration attraction. Build once Mega-Engineering is available.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures or distinguish new construction from repairs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Same new-construction caveat.

---

## forge_among_the_stars — Our Fleets Will Blot Out the Stars

Source requirement: Construct a complete Mega-Shipyard from scratch (repairs do not count).

```json
{
  "tags": ["megastructures", "fleet", "late-game", "federations"],
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
      "notes": "This achievement requires the Federations expansion."
    },
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "mega_shipyard",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires building a Mega-Shipyard to completion. Repairs do not count."
    }
  ],
  "warnings": [
    "New construction only — repairing a ruined Mega-Shipyard does not count."
  ],
  "planner_notes": "The Mega-Shipyard dramatically accelerates fleet construction. Prioritize Mega-Engineering and save alloys for the build.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures or distinguish new construction from repairs."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: Same new-construction caveat.

---

## burning_brightly — Burning Brightly

Source requirement: Have a fully-upgraded Arc Furnace and Dyson Swarm in the same system.

```json
{
  "tags": ["megastructures", "economy", "late-game", "machine-age"],
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
      "notes": "This achievement requires the Machine Age expansion."
    },
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "arc_furnace",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires a completed Arc Furnace in a system."
    },
    {
      "condition_type": "required",
      "dimension": "megastructure_type",
      "operator": "contains",
      "value": "dyson_swarm",
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires a completed Dyson Swarm in the same system as the Arc Furnace."
    }
  ],
  "warnings": [
    "Both megastructures must be fully upgraded and in the exact same star system."
  ],
  "planner_notes": "The Arc Welders origin starts with an incomplete Arc Furnace, making this achievement significantly easier. Build the Dyson Swarm in the same system.",
  "known_limitations": [
    "The current save parser does not yet detect completed megastructures, their upgrade stages, or co-location in the same system."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The co-location constraint (both in the same system) is the primary complexity. The `megastructure_type` conditions above cannot express "both in the same system" — that requires a compound dimension or a `megastructure_co_location` boolean. The conditions here are a best-effort representation.

---

## volcanic_empire — Volcanic Empire

Source requirement: Terraform 10 owned planets to Volcanic World before a Galactic Firestorm victory.

```json
{
  "tags": ["colony", "infernals", "mid-game"],
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
      "notes": "This achievement requires the Infernals expansion."
    },
    {
      "condition_type": "required",
      "dimension": "planets_terraform_to_volcanic",
      "operator": "at_least",
      "value": 10,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires terraforming at least 10 owned planets to Volcanic before a Galactic Firestorm victory."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_path_hyperthermia",
      "operator": "equals",
      "value": false,
      "timing": "terminal",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The terraforming must be completed before the Galactic Firestorm victory (the Infernals crisis path end)."
    }
  ],
  "warnings": [
    "Time-limited by the Galactic Firestorm victory condition; delaying victory allows more terraforming."
  ],
  "planner_notes": "Start with Planet Forgers or Planetary Architects civic and set habitable planets to 5x. Prioritize the Volcanic Terraforming technologies and terraform planets systematically before completing the crisis path.",
  "known_limitations": [
    "The current save parser does not yet detect terraforming outcomes by planet class or track crisis completion state for Infernals."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `planets_terraform_to_volcanic` dimension from DIMENSIONS.md matches this achievement directly. The "before Firestorm victory" constraint is a timing restriction that is hard to evaluate from save state alone. The `crisis_path_hyperthermia = false` condition is a proxy that assumes the evaluation checks whether the victory has already occurred.

---

## blazing_domain — Blazing Domain

Source requirement: Terraform 100 stars to Red Giants before a Galactic Firestorm victory.

```json
{
  "tags": ["infernals", "late-game"],
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
      "notes": "This achievement requires the Infernals expansion."
    },
    {
      "condition_type": "required",
      "dimension": "stars_terraform_to_red_giant",
      "operator": "at_least",
      "value": 100,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires terraforming at least 100 stars into Red Giants before a Galactic Firestorm victory."
    },
    {
      "condition_type": "required",
      "dimension": "crisis_path_hyperthermia",
      "operator": "equals",
      "value": false,
      "timing": "terminal",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The star terraforming must be completed before the Galactic Firestorm victory."
    }
  ],
  "warnings": [
    "Requires 100 star terraforming operations before the crisis victory; time management is critical."
  ],
  "planner_notes": "Use the zero-empires trick to avoid interference. Focus on star terraforming technologies and allocate resources to continually convert stars as soon as cooldowns permit.",
  "known_limitations": [
    "The current save parser does not yet track star terraforming operations or the Infernals crisis path state."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `stars_terraform_to_red_giant` dimension from DIMENSIONS.md matches directly. The crisis-timing constraint is the same pattern as volcanic_empire.

---

## burning_heaven — Burning Heaven

Source requirement: Terraform an owned Holy World to a Volcanic World.

```json
{
  "tags": ["colony", "infernals", "event-limited"],
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
      "notes": "This achievement requires the Infernals expansion."
    },
    {
      "condition_type": "required",
      "dimension": "volcanic_holy_world_created",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires terraforming an owned Holy World into a Volcanic World."
    }
  ],
  "warnings": [
    "Holy Worlds are rare and guarded by Fallen Empires. Owning one requires conquering or inheriting it from a Fallen Empire that spawns with one."
  ],
  "planner_notes": "Start with Planet Forgers or Planetary Architects civic. Find a Fallen Empire with a Holy World, conquer it, and terraform it to Volcanic.",
  "known_limitations": [
    "The current save parser does not yet detect planet class changes or interaction with Fallen Empire-owned worlds."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `volcanic_holy_world_created` dimension from DIMENSIONS.md matches directly. The main challenge is finding and acquiring a Holy World, which depends on Fallen Empire spawns.

---

## summer_vacation — Summer Vacation

Source requirement: Have a Resort World on a Volcanic World.

```json
{
  "tags": ["colony", "economy", "infernals", "mid-game"],
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
      "notes": "This achievement requires the Infernals expansion."
    },
    {
      "condition_type": "required",
      "dimension": "planet_class",
      "operator": "equals",
      "value": "pc_volcanic",
      "timing": "setup",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires a Volcanic World planet."
    },
    {
      "condition_type": "required",
      "dimension": "planet_designation",
      "operator": "equals",
      "value": "colony_resort",
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires designating the planet as a Resort World."
    }
  ],
  "warnings": [
    "The planet must both be Volcanic class and have the Resort World designation."
  ],
  "planner_notes": "Start on a Volcanic World with guaranteed habitable worlds turned on. Colonize the Volcanic world and use the planet decision to make it a Resort World.",
  "known_limitations": [
    "The current save parser does not yet read planet class or designation."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `planet_class` and `planet_designation` dimensions do not exist in DIMENSIONS.md. The Resort World designation is a planetary decision (costs energy and requires the Paradise Dome building tech). The condition above is a best-effort composite of class + designation.

---

## from_bad_to_worse — From Bad to Worse

Source requirement: Use 5 Blazing Scourge decisions on a Tomb World. Requires the Fire Cult civic.

```json
{
  "tags": ["infernals", "event-limited", "resources"],
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
      "notes": "This achievement requires the Infernals expansion."
    },
    {
      "condition_type": "required",
      "dimension": "civic",
      "operator": "contains",
      "value": "civic_fire_cult",
      "timing": "setup",
      "mutability": "immutable",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "The Blazing Scourge decision requires the Fire Cult civic."
    },
    {
      "condition_type": "required",
      "dimension": "blazing_scourge_decisions",
      "operator": "at_least",
      "value": 5,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires using the Blazing Scourge decision at least 5 times on a Tomb World."
    }
  ],
  "warnings": [
    "Requires the Fire Cult civic, which locks the empire into specific ethics and gameplay."
  ],
  "planner_notes": "Start with the Fire Cult civic. Find a Tomb World (common from the Sol system or event outcomes) and use the Blazing Scourge planetary decision 5 times.",
  "known_limitations": [
    "The current save parser reads civics but does not yet track planetary decision usage."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `blazing_scourge_decisions` dimension from DIMENSIONS.md matches directly. The Tomb World location qualifier is not separately dimensioned but is implied by the decision's availability (Blazing Scourge can only be used on Tomb Worlds).

---

## arcana — Arcana

Source requirement: Use the Reverse-Engineer Arcane Technology relic option to discover a random technology. Requires the Arcane Deciphering technology.

```json
{
  "tags": ["relic", "technology", "ancient-relics"],
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
      "notes": "This achievement requires the Ancient Relics Story Pack."
    },
    {
      "condition_type": "required",
      "dimension": "relic_active_effect_used",
      "operator": "equals",
      "value": true,
      "timing": "eventual",
      "mutability": "event_limited",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires using the Reverse-Engineer Arcane Technology relic action (available after researching Arcane Deciphering)."
    }
  ],
  "warnings": [
    "Requires the Arcane Deciphering technology (Society, rare) to unlock the relic action."
  ],
  "planner_notes": "Accumulate Minor Artifacts from archaeological sites, minor artifact deposits, or the Rubricator chain, then activate the Reverse-Engineer Arcane Technology option once Arcane Deciphering is researched.",
  "known_limitations": [
    "The current save parser does not yet track relic action usage or Minor Artifact counts."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `relic_active_effect_used` dimension is a boolean for any relic activation but the achievement specifically requires one particular relic action. A more specific dimension like `relic_action_reverse_engineer_arcane_used` would be more precise.

---

## artificer — Artificer

Source requirement: Have more than 999 Minor Artifacts (despite the description saying 100).

```json
{
  "tags": ["resources", "economy", "ancient-relics", "mid-game"],
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
      "notes": "This achievement requires the Ancient Relics Story Pack."
    },
    {
      "condition_type": "required",
      "dimension": "minor_artifacts_count",
      "operator": "greater_than",
      "value": 999,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires > 999 Minor Artifacts in storage. Note that the achievement description says 100 but the actual requirement is 1000."
    }
  ],
  "warnings": [
    "The in-game description states 100 but the actual requirement is 1000 minor artifacts."
  ],
  "planner_notes": "Archaeological sites, the Rubricator chain, and strategic resource deals with the Artisan Troupe can generate Minor Artifacts. Avoid spending them unnecessarily until the threshold is reached.",
  "known_limitations": [
    "The current save parser does not yet count Minor Artifact storage."
  ],
  "rule_confidence": "high"
}
```

Uncertainty notes: The wiki explicitly documents the description-vs-requirement mismatch. The `minor_artifacts_count` dimension does not exist in DIMENSIONS.md; a new integer dimension for Minor Artifact count should be added.

---

## relic_hunter — Relic Hunter

Source requirement: Own at least 5 relics in one game. The wiki hint says any 5 relics work despite the description mentioning "Ancient Relics Story Pack."

```json
{
  "tags": ["relic", "exploration", "ancient-relics", "mid-game"],
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
      "notes": "This achievement requires the Ancient Relics Story Pack."
    },
    {
      "condition_type": "required",
      "dimension": "relic_count_owned",
      "operator": "at_least",
      "value": 5,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires owning at least 5 relics simultaneously in one game. Any relics count, not just Ancient Relics Story Pack ones."
    }
  ],
  "warnings": [
    "Relics are obtained through archaeological sites, leviathan victories, precursor chains, and events. RNG can delay progress."
  ],
  "planner_notes": "Play a wide-exploration game with high archaeological site spawns (Treasure Hunters origin helps). Prioritize precursor chains and leviathan hunting to collect relics.",
  "known_limitations": [
    "The current save parser does not yet count owned relics or track which relics are in the inventory."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The `relic_count_owned` dimension does not exist in DIMENSIONS.md. The existing `relic_owned` dimension is for a specific named relic, not a count. `relic_hunter` needs a count. The wiki specifically notes that the achievement's description is misleading — any 5 relics work, not just Ancient Relics ones.

---

## make_great_pets — We'll Make Great Pets

Source requirement: Have more than 999 pops of the primary species in Alien Zoos within the empire. The pops must be non-sapient (pre-sapient or slave strata) and of the founding species. Requires the Pre-Sapients policy be set to Protected.

```json
{
  "tags": ["species-management", "slavery", "ancient-relics", "late-game"],
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
      "notes": "This achievement requires the Ancient Relics Story Pack."
    },
    {
      "condition_type": "required",
      "dimension": "pops_in_alien_zoos",
      "operator": "greater_than",
      "value": 999,
      "timing": "terminal",
      "mutability": "normal_change",
      "severity": "soft",
      "source": "wiki-reviewed",
      "notes": "Requires > 999 pops of the founding species displayed in Alien Zoo buildings. These pops must be non-sapient or in the slave stratum."
    }
  ],
  "warnings": [
    "The Pre-Sapients policy must be set to Protected. The Alien Zoo building requires the Xeno-Zoology technology. Founding species pops must be non-sapient."
  ],
  "planner_notes": "This is a challenging, niche achievement. Find a pre-sapient version of your own species (rare), or use genetic modification to create one. Alternatively, exploit pre-sapient species that match your founder species. Build many Alien Zoo buildings to house them.",
  "known_limitations": [
    "The current save parser does not yet track pop locations in specific buildings, alien zoo counts, or founding species pop strata."
  ],
  "rule_confidence": "low"
}
```

Uncertainty notes: This is one of the most complex achievements in this batch. The requirement involves: (1) founding species pops, (2) non-sapient or slave status, (3) located in Alien Zoo buildings. The `pops_in_alien_zoos` dimension does not exist in DIMENSIONS.md and would require tracking pops at the building level. The wiki states the requirement differently in different places — the source requirement field used here comes from the wiki. Rule confidence is low due to the multi-dimensional specificity.

---

## deus_vult — Deus Vult

Source requirement: Colonize 4 worlds with the Holy World modifier. Requires a spiritualist ethic.

```json
{
  "tags": ["colony", "economy", "spiritualist", "mid-game", "fallen-empire"],
  "conditions": [
    {
      "condition_type": "required",
      "dimension": "ethic",
      "operator": "contains",
      "value": "ethic_spiritualist",
      "timing": "setup",
      "mutability": "slow_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires a spiritualist or fanatic spiritualist ethic to safely colonize holy worlds without immediate war."
    },
    {
      "condition_type": "required",
      "dimension": "colonized_planets",
      "operator": "at_least",
      "value": 4,
      "timing": "eventual",
      "mutability": "normal_change",
      "severity": "hard",
      "source": "wiki-reviewed",
      "notes": "Requires colonizing 4 planets with the Holy World modifier."
    }
  ],
  "warnings": [
    "Holy Worlds are owned by the Holy Guardians or Doctrinal Enforcers Fallen Empire — colonizing them without a spiritualist ethic or before the FE is defeated triggers immediate war.",
    "The holy world modifier applies to specific preset planets (e.g., Prophet's Retreat, Walled Garden) that are part of the FE's territory."
  ],
  "planner_notes": "Play as a spiritualist empire to avoid early FE retaliation. If playing non-spiritualist, you must first defeat or cripple the spiritualist FE before colonizing their holy worlds. Set Fallen Empires to at least 2 to ensure a spiritualist FE spawns.",
  "known_limitations": [
    "The current save parser reads ethics but does not yet identify the Holy World modifier on planets or distinguish holy worlds from other colonizable worlds."
  ],
  "rule_confidence": "medium"
}
```

Uncertainty notes: The wiki requirement says "As a spiritualistic empire, own 4 holy worlds" but the achievement's in-game requirement field says just "Colonize 4 worlds with the Holy World modifier." The spiritualist requirement is from the wiki description. Non-spiritualist empires can still colonize holy worlds, but doing so will anger the FE. The condition above uses the safer spiritualist approach.
