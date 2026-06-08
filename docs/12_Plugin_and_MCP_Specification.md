# 12 — SilicaRAW Plugin & MCP Specification

Status: GO WITH CONDITIONS

## Principle

Extensions can assist. Extensions cannot bypass user trust.

## Plugin Layer

v1 starts conservatively:

- Declarative preset plugins
- Local export presets later
- AI model registration later
- No arbitrary executable plugins in v1
- No plugin marketplace in v1

## Plugin Manifest

Required fields:

- plugin_id
- name
- version
- license
- author
- type
- minimum_silica_version
- permissions

## Permission Categories

- preset:provide
- export:local
- metadata:read/write
- catalog:read/write_flags
- edit:read/apply
- ai:model_register/run
- workflow:run
- network:access, future
- filesystem:limited, future

Default: no permissions until approved.

## MCP Layer

Default: OFF.

Modes:

- Off
- Read-only
- Review
- Edit
- Export
- No dangerous mode in v1

## MCP Tools

Read-only:

- silica.photos.list
- silica.photos.get
- silica.photos.get_metadata
- silica.collections.list
- silica.selection.get
- silica.presets.list
- silica.exports.list

Edit:

- set_rating
- set_rejected
- set_pick
- apply_preset
- apply_values
- create_collection

AI:

- run_blur_review
- run_quality_score
- create_subject_mask
- create_sky_mask
- suggest_auto_tone

Export:

- create_plan
- run
- status
- cancel

Forbidden:

- delete_original
- overwrite_original
- raw_sql
- unrestricted filesystem
- change MCP permissions
- install/enable plugins

## Logging

All mutation/export/permission/plugin actions are logged in action_log.

## Final Verdict

GO WITH CONDITIONS.

Need exact MCP schemas, permission enum, action log implementation, transport/session design, plugin manifest validation.
