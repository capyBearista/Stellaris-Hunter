# Achievement Curation Style Guide

Use this guide when filling `curation` blocks in `catalog/latest.json`.

## Purpose

Curation translates Stellaris wiki achievement requirements into structured planner metadata. It is not limited by the current save parser. If a condition is known from the wiki but not locally parseable yet, still include the condition and add a `known_limitations` note.

## Required Curation Shape

Every non-deprecated achievement should have:

- `tags`: 2-5 useful tags
- `conditions`: at least one structured condition
- `warnings`: an array, empty only when there are no meaningful RNG/timing/setup warnings
- `planner_notes`: one or two practical sentences
- `known_limitations`: parser or rule-engine limitations, if applicable
- `rule_confidence`: `"high"`, `"medium"`, or `"low"`

## Condition Shape

Use this shape for every condition:

```json
{
  "condition_type": "required",
  "dimension": "species_class",
  "operator": "equals",
  "value": "Lithoid",
  "timing": "setup",
  "mutability": "immutable",
  "severity": "hard",
  "source": "wiki-reviewed",
  "notes": "Player founding species must be Lithoid."
}
```

## Operators

- `equals`: exact scalar equality for string, boolean, or numeric values
- `contains`: membership in an array or collection-like value
- `at_least`: numeric threshold where equality satisfies the condition
- `greater_than`: numeric threshold where equality does not satisfy the condition

Avoid inventing new operators unless the requirement cannot be expressed with these four.

## Timing

- `setup`: determined at empire creation or before the run meaningfully starts
- `discovery`: determined by galaxy generation, RNG, or finding a target
- `current`: must be true in the current state but can change
- `eventual`: can be achieved through normal play over the run
- `terminal`: must be true at or near the relevant end state
- `event_limited`: depends on a specific event chain or narrow opportunity

## Mutability

- `immutable`: cannot realistically change after setup
- `slow_change`: can change, but only with major investment or limited mechanics
- `normal_change`: changes through normal gameplay
- `rng_locked`: depends substantially on random generation or event availability
- `event_limited`: depends on a specific event outcome or time window
- `unknown`: use only when the wiki requirement is too ambiguous to classify

## Severity

- `hard`: if unmet, this is a likely blocker for the run
- `soft`: if unmet, the achievement may still be possible through normal play

Use `hard` for setup gates, mutually exclusive conditions, specific crisis/precursor requirements, and missed event windows. Use `soft` for ordinary progression thresholds.

## Rule Confidence

- `high`: straightforward wiki requirement with unambiguous structured conditions
- `medium`: clear requirement but involves several moving parts, event chains, or interpretation
- `low`: highly situational, RNG-heavy, version-sensitive, or not precisely expressible yet

Difficulty does not automatically determine confidence. A very hard achievement can still have high rule confidence if the requirements are unambiguous.

## Known Limitations

Known limitations should describe parser or evaluation gaps, not gameplay strategy. Examples:

- `"The current save parser does not yet detect completed megastructures."`
- `"The current save parser does not yet track crisis outcome flags."`
- `"The current save parser does not yet count monthly resource production."`

Do not omit a condition merely because it is not currently parseable.

## Planner Notes

Planner notes should be concise and useful. Good notes explain setup choices, helpful game settings, or strategic constraints from the wiki hint.

Examples:

- `"Start with the required origin and avoid ethics/civic changes that would invalidate the setup requirement."`
- `"Lower crisis strength and force the matching crisis if the galaxy settings allow it."`
- `"This is usually easiest in a dedicated run because the timing window is narrow."`

## Warnings

Use warnings for blockers or risks that the planner should surface prominently:

- RNG target may not spawn
- Achievement requires a specific crisis, precursor, or leviathan
- Timing window can be missed
- Setup condition is mutually exclusive with other achievements
- DLC ownership is required
- War or federation state may lock out the condition

## DLC Conditions

For non-Base game achievements, add a `dlc_required` condition unless the achievement's source group is misleading or the requirement is clearly available without that DLC.

Example:

```json
{
  "condition_type": "required",
  "dimension": "dlc_required",
  "operator": "equals",
  "value": "Utopia",
  "timing": "setup",
  "mutability": "immutable",
  "severity": "hard",
  "source": "wiki-reviewed",
  "notes": "This achievement requires the Utopia DLC."
}
```

## Consistency Rules

- Use `rare_technologies_acquired`, not `rare_tech_count`.
- Use `captured_prethoryn_scourge_queen`, not `captured_prethoryn_queen`.
- Use `galactic_community_exists`, not `galactic_community_formed`.
- Use `megastructure_type`, not `megastructure_built`.
- Use `gateway_count`, not `gateway_built`.
- Use `species_class`, not species booleans such as `aquatic_species`.
- Use specific event dimensions instead of vague dimensions such as `shroud_event` or `biogenesis_event`.

## Output Expectations For Batch Drafts

Batch drafts should preserve each achievement's existing fields and provide only replacement `curation` blocks unless explicitly asked to merge into `catalog/latest.json`.

Each batch should include:

- Achievement ID
- Achievement name
- Replacement `curation` JSON
- Any uncertainty notes for human review
