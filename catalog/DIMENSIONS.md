# Achievement Curation Dimensions

This document defines the forward-looking `dimension` keys used in achievement curation conditions. Dimensions are catalog-authoring concepts: they describe structured requirements from the Stellaris wiki and do not imply that the local save parser can extract the value yet.

## Scope Check

The taxonomy below was checked against all 211 achievements in `catalog/latest.json`, including the 195 entries that still need curation. The audit added missing dimensions for species traits, first-contact outcomes, specific leviathan/event chains, relic actions, subject types, war types, psionic covenants, and DLC-specific mechanics. It also removes redundant dimensions such as `wormhole_present`, `amoeba_present`, and species booleans that are better represented by `species_class`.

## Naming Rules

- Use snake_case for every dimension.
- Use boolean past-tense dimensions for completed one-off events, such as `wormhole_travel_completed`.
- Use count or threshold dimensions for numeric values, such as `owned_planets` or `federation_level`.
- Use type dimensions for enum-like values, such as `megastructure_type`, `war_type`, or `covenant_type`.
- Keep existing curated dimensions stable unless there is a clear conflict.

## Setup Dimensions

Setup dimensions are usually determined at empire creation. They are commonly paired with `timing: "setup"`, `mutability: "immutable"`, and `severity: "hard"`.

| Dimension | Value Type | Examples | Notes |
| --- | --- | --- | --- |
| `species_class` | string | `"Humanoid"`, `"Lithoid"`, `"Machine"`, `"Aquatic"` | Existing dimension. Use for species-class requirements. |
| `species_trait` | string | `"trait_budding"`, `"trait_venerable"` | Use for founder-species biological traits only. Ruler/leader traits use `ruler_trait` instead. |
| `ruler_trait` | string | `"leader_trait_brainslug"` | Use for ruler/leader-specific traits — parsed from `country.ruler → leaders.{id}.traits`. Not for founder-species biological traits (use `species_trait` for those). |
| `origin` | string | `"origin_clone_army"`, `"origin_necrophage"`, `"origin_broken_shackles"` | Use for origin-gated achievements. |
| `civic` | string | `"civic_determined_exterminator"`, `"civic_masterful_crafters"` | Existing dimension. Usually evaluated with `contains`. |
| `ethic` | string | `"ethic_pacifist"`, `"ethic_xenophobe"` | Existing dimension. Usually evaluated with `contains`. |
| `authority` | string | `"auth_democratic"`, `"auth_imperial"`, `"auth_corporate"`, `"auth_hive_mind"`, `"auth_machine_intelligence"` | Use for authority or empire-type requirements. |

## Discovery Dimensions

Discovery dimensions depend on galaxy generation, random events, or finding a specific target. They are commonly paired with `timing: "discovery"` and `mutability: "rng_locked"`.

| Dimension | Value Type | Examples | Notes |
| --- | --- | --- | --- |
| `endgame_crisis` | string | `"prethoryn"`, `"unbidden"`, `"contingency"`, `"cetana"` | Existing dimension. Use for crisis type prerequisites. |
| `sol_system_era` | string | `"world_war_ii"`, `"machine_age"` | Existing dimension. Covers primitive Earth variants. |
| `primitive_earth_present` | bool | `true` | Use when Earth itself must exist as a target. |
| `pre_ftl_era_target` | string | `"stone_age"`, `"bronze_age"`, `"machine_age"` | Use for any pre-FTL target, not only Earth/Sol. |
| `target_species_class` | string | `"Reptilian"` | Existing dimension. Use when target species class matters. |
| `target_homeworld_class` | string | `"pc_desert"` | Existing dimension. Use when the target planet class matters. |
| `precursor_type` | string | `"zroni"`, `"baol"`, `"vultaum"` | Use when the spawned precursor matters. |
| `precursor_chain_completed` | string | `"zroni"`, `"baol"`, `"inetian_traders"` | Use for completing a specific precursor chain. |
| `l_cluster_unlocked` | bool | `true` | Use for L-Gate/L-Cluster discovery achievements. |
| `shielded_world_unlocked` | bool | `true` | Use for shielded-world event outcomes. |
| `ancient_leviathan` | string | `"ether_drake"`, `"stellarite_devourer"`, `"enigma_fortress"` | Use for finding or defeating specific leviathans. |
| `enclave_type_present` | string | `"artisan"`, `"curator"`, `"trader"`, `"mercenary"` | Use when an enclave must exist or be found. |
| `great_khan_spawned` | bool | `true` | Use for Great Khan-specific achievements. |

## Progression Dimensions

Progression dimensions describe thresholds that can change through normal play. They are commonly paired with `timing: "eventual"` or `"terminal"` and `mutability: "normal_change"`.

| Dimension | Value Type | Examples | Notes |
| --- | --- | --- | --- |
| `owned_planets` | integer | `1`, `10`, `25` | Existing dimension. |
| `colonized_planets` | integer | `1`, `5`, `10` | Use when the requirement specifically says colonize/colonies. |
| `surveyed_planets` | integer | `1`, `100` | Existing dimension. |
| `total_pops` | integer | `100`, `500`, `1000` | Use for population thresholds. |
| `enslaved_pops_count` | integer | `20000` | Use for enslaved-pop thresholds. |
| `robot_pop_ratio` | number | `0.75` | Use for percentage robotic/synthetic population requirements. |
| `energy_stored` | integer | `1000`, `5000` | Use for stored energy credit thresholds. |
| `energy_monthly` | integer | `1000` | Use when monthly income is specified. |
| `minerals_monthly` | integer | `250`, `1000` | Use for monthly mineral income thresholds. |
| `alloys_monthly` | integer | `100`, `500` | Use for monthly alloy income thresholds. |
| `trade_value_monthly` | integer | `1000` | Use for trade-value thresholds. |
| `strategic_resources_types` | integer | `10` | Use for owning multiple strategic resource types. |
| `organic_empires_remaining` | integer | `0` | Use when an achievement requires eliminating all biological/non-machine empires. |
| `fleet_power` | integer | `10000`, `40000`, `100000` | Use for fleet or starbase military-power thresholds. |
| `fleet_count` | integer | `100`, `200` | Use for fleet-size requirements. |
| `starbase_count` | integer | `10`, `50` | Use for starbase count requirements. |
| `gateway_count` | integer | `4` | Use for active gateway network requirements. |
| `hyper_relay_count` | integer | `30` | Use for hyper relay network requirements. |
| `rare_technologies_acquired` | integer | `1`, `15` | Existing dimension. Prefer this over `rare_tech_count`. |
| `traditions_adopted` | integer | `42` | Use for tradition completion thresholds. |
| `ascension_perks_unlocked` | integer | `1`, `8` | Existing dimension. |
| `ascension_path` | string | `"genetic"`, `"synthetic"`, `"psionic"`, `"cybernetic"`, `"virtual"`, `"modularity"`, `"nanotech"` | Use for committed/completed ascension paths. When a path is detected in the save, mismatched path requirements are treated as `Impossible` (mutually exclusive). When no path is detected, path requirements evaluate as `Unknown`. |
| `years_played` | integer | `100`, `200` | Use for elapsed-time requirements. |
| `years_at_peace` | integer | `200` | Existing dimension. |
| `diplomatic_weight` | integer | `9000` | Use for diplomatic-weight thresholds. |
| `intel_level_count` | integer | `5` | Use for intel thresholds across multiple empires. |
| `observation_station_count` | integer | `10` | Use for observation-post count requirements. |
| `capital_building_level` | string | `"imperial_complex"` | Use for capital building upgrade requirements. |
| `living_standard` | string | `"utopian_abundance"` | Use for living-standard requirements. |
| `mercenary_enclaves_patroned` | integer | `3` | Use for mercenary enclave patronage thresholds. |
| `vivarium_capacity` | integer | `50` | Use for Grand Archive vivarium capacity requirements. |

## Action And Event Dimensions

Action and event dimensions represent player-driven milestones, discrete event-chain outcomes, or one-off actions.

| Dimension | Value Type | Examples | Notes |
| --- | --- | --- | --- |
| `wars_won` | integer | `1`, `5` | Use for generic war-win counts. |
| `war_type` | string | `"humiliation"`, `"war_in_heaven"`, `"defensive_war_against_fallen_empire"` | Use when the type of war matters. |
| `subjects_acquired` | integer | `1`, `5` | Use for subject acquisition thresholds. |
| `vassal_count` | integer | `1`, `5` | Use for number of subjects/vassals. |
| `subject_type` | string | `"bulwark"`, `"prospectorium"`, `"scholarium"` | Use for specialist subject requirements. |
| `subject_contract_modified` | bool | `true` | Use for vassal contract achievements. |
| `secret_fealty_pledged` | bool | `true` | Use for allegiance-war achievements. |
| `federation_formed` | bool | `true` | Use for forming any federation. |
| `federation_type` | string | `"hegemony"`, `"research"`, `"trade"` | Use for specific federation types. |
| `federation_level` | integer | `5` | Use for federation level requirements. |
| `federation_member_ethics` | integer | `8` | Use for achievements requiring all ethics represented. |
| `galactic_community_exists` | bool | `true` | Existing dimension. Prefer over `galactic_community_formed`. |
| `galactic_community_founding_member` | bool | `true` | Use for founding-member requirements. |
| `galactic_custodian` | bool | `true` | Use for becoming custodian. |
| `galactic_custodian_action` | string | `"end_custodianship"`, `"insult_all_members"` | Use when the custodian must perform a specific action. |
| `galactic_emperor` | bool | `true` | Use for becoming emperor. |
| `galactic_emperor_rebellion` | bool | `true` | Use for rebellion against the emperor. |
| `colossus_built` | bool | `true` | Use for owning/building any Colossus. |
| `colossus_weapon_type` | string | `"world_cracker"`, `"global_pacifier"`, `"deluge_machine"` | Use for specific Colossus weapon outcomes. |
| `colossus_destroyed_while_firing` | bool | `true` | Use for Stay on Target-style timing windows. |
| `megastructure_type` | string | `"dyson_sphere"`, `"science_nexus"`, `"ring_world"`, `"arc_furnace"`, `"dyson_swarm"` | Use with `contains` for built/owned megastructure types. |
| `archaeological_site_completed` | integer | `1`, `5` | Existing dimension. |
| `robot_pop_built` | bool | `true` | Existing dimension. |
| `wormhole_travel_completed` | bool | `true` | Existing dimension. |
| `pre_ftl_infiltration_completed` | bool | `true` | Existing dimension. |
| `first_contact_result` | string | `"peaceful"`, `"aggressive"`, `"shoot_to_kill"` | Use for First Contact outcome achievements. |
| `espionage_operations_completed` | integer | `1`, `5` | Use for operation count requirements. |
| `migration_treaty_count` | integer | `10` | Use for migration-treaty thresholds. |
| `species_genetically_modified` | bool | `true` | Use for genetic modification achievements. |
| `species_uplifted` | bool | `true` | Use for uplifting achievements. |
| `species_on_planet_count` | integer | `10` | Use for multi-species planet requirements. |
| `species_dna_phenotypes_collected` | integer | `6` | Use for BioGenesis phenotype collection. |
| `slavery_type` | string | `"livestock"` | Existing dimension. |
| `livestock_species_count` | integer | `5` | Existing dimension. |
| `purged_pops` | integer | `100` | Use for purge/genocide achievements. |
| `species_enslaved` | integer | `1`, `5` | Use for number of enslaved species. |
| `relic_owned` | string | `"galatron"`, `"rubricator"` | Use for owning a specific relic. |
| `relic_active_effect_used` | bool | `true` | Use for achievements requiring activation of any relic effect. |
| `galatron_acquired` | bool | `true` | Use for receiving the Galatron. |
| `galatron_captured` | bool | `true` | Use for capturing the Galatron in war. |
| `artisan_enclave_patron` | bool | `true` | Existing dimension. |
| `enclave_interaction_type` | string | `"patron"`, `"recruit_curator_scientist"`, `"buy_trader_resource"`, `"destroy_enclave"` | Use for outcome-specific enclave interactions. |
| `amoeba_companion_found` | bool | `true` | Existing dimension. |
| `amoeba_companion_killed` | bool | `true` | Existing dimension. |
| `horizon_signal_completed` | bool | `true` | Use for Horizon Signal outcomes. |
| `machine_uprising_victory` | bool | `true` | Use for winning as a machine uprising. |
| `civil_war_completed` | bool | `true` | Use for origin-specific civil war outcomes. |
| `special_project_completed_type` | string | `"limbo"`, `"mysterious_chart"` | Use for named special project outcomes. |
| `colony_count_with_hyperspace_not_researched` | integer | `10` | Use for The Path Not Taken. |
| `crisis_defeated` | string or bool | `"prethoryn"`, `"unbidden"`, `"contingency"`, `"cetana"`, `true` | Use for defeating a crisis. Use a specific string value when the achievement requires a named crisis. Use `true` when any crisis defeat qualifies or when the requirement covers multiple crises without singling one out. **Save-game semantics**: The game tracks crisis state primarily through per-crisis `happened` blocks in the save — each crisis type has its own continuous block that fires at specific progression milestones throughout a playthrough. For the legacy endgame crises verified so far, defeating the crisis sets a persistent `crisis_defeated` country flag. This flag is **monotonic/eventual**: once set, it remains set for the remainder of the playthrough. Verification of newer crisis variants such as Cetana is still pending. The runtime currently reads `crisis_defeated` as a boolean via `has_flag(country_value, "crisis_defeated")`; it does **not** parse per-crisis string identities from `happened` blocks. Catalog entries using a non-boolean string value for this dimension cannot be evaluated by the current rule engine — they serve as structured documentation until per-crisis extraction is implemented. |
| `captured_prethoryn_scourge_queen` | bool | `true` | Existing dimension. Use this exact name. |
| `crisis_path_nemesis` | bool | `true` | Use for Become the Crisis / Aetherophasic Engine achievements. |
| `crisis_path_cosmogenesis` | bool | `true` | Use for Machine Age Cosmogenesis achievements. |
| `crisis_path_hyperthermia` | bool | `true` | Use for Infernals Hyperthermia achievements. |
| `crisis_path_behemoth_fury` | bool | `true` | Use for BioGenesis Behemoth Fury achievements. |
| `covenant_type` | string | `"eater_of_worlds"`, `"instrument_of_desire"`, `"end_of_the_cycle"` | Use for Shroud covenant achievements. |
| `psionic_techs_unlocked` | integer | `1`, `5` | Use for psionic ship-component collection. |
| `proxy_war_count` | integer | `5` | Use for proxy-war achievements. |
| `quantum_catapult_used` | bool | `true` | Use for quantum catapult achievements. |
| `astral_rifts_explored` | integer | `1`, `5` | Use for Astral Planes rift achievements. |
| `space_fauna_type_captured` | string | `"amoeba"`, `"tiyanki"`, `"crystalline_entity"`, `"asteroid"` | Use for Grand Archive capture achievements. |
| `legendary_paragon_recruited` | string | `"azaryn"`, `"keides"` | Use for Galactic Paragons story achievements. |
| `galactic_memorials_on_tomb_worlds` | integer | `5` | Use for Necroids memorial achievements. |
| `blazing_scourge_decisions` | integer | `5` | Use for Infernals Blazing Scourge achievements. |
| `stars_terraform_to_red_giant` | integer | `100` | Use for Infernals red-giant transformation achievements. |
| `planets_terraform_to_volcanic` | integer | `10` | Use for Infernals volcanic planet transformation achievements. |
| `volcanic_holy_world_created` | bool | `true` | Use for Infernals holy-world transformation achievements. |

## Metadata Dimensions

Metadata dimensions describe requirements that come from achievement availability rather than save state.

| Dimension | Value Type | Examples | Notes |
| --- | --- | --- | --- |
| `dlc_required` | string | `"Utopia"`, `"Leviathans"`, `"Federations"` | Use for achievements that require DLC ownership. Do not add for Base game achievements unless the wiki requirement clearly depends on DLC content. |

## Existing Dimensions To Preserve

The following dimensions already appear in curated catalog entries and should remain valid: `archaeological_site_completed`, `owned_planets`, `rare_technologies_acquired`, `robot_pop_built`, `wormhole_travel_completed`, `surveyed_planets`, `species_class`, `target_species_class`, `pre_ftl_infiltration_completed`, `ethic`, `years_at_peace`, `ascension_perks_unlocked`, `livestock_species_count`, `slavery_type`, `sol_system_era`, `invaded_primitive_earth`, `endgame_crisis`, `captured_prethoryn_scourge_queen`, `artisan_enclave_patron`, `galactic_community_exists`, `civic`, `target_homeworld_class`, `amoeba_companion_found`, and `amoeba_companion_killed`.

## Dimensions To Avoid

- Avoid `rare_tech_count`; use `rare_technologies_acquired`.
- Avoid `captured_prethoryn_queen`; use `captured_prethoryn_scourge_queen`.
- Avoid `galactic_community_formed`; use `galactic_community_exists` or `galactic_community_founding_member`.
- Avoid `megastructure_built`; use `megastructure_type`.
- Avoid `gateway_built`; use `gateway_count`.
- Avoid `wormhole_present`; use `wormhole_travel_completed`.
- Avoid `amoeba_present`; use `amoeba_companion_found`.
- Avoid species booleans such as `aquatic_species`, `toxoid_species`, or `machine_species`; use `species_class`.
- Avoid vague DLC event dimensions such as `shroud_event` or `biogenesis_event`; use specific event/action dimensions.
