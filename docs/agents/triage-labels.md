# Triage Label Configuration

## Label Mapping

The following labels are used by the `triage` skill to manage issue workflow:

| Canonical Role | Label String | Description |
|----------------|--------------|-------------|
| `needs-triage` | `needs-triage` | Maintainer needs to evaluate the issue |
| `needs-info` | `needs-info` | Waiting on reporter for clarification or additional information |
| `ready-for-agent` | `ready-for-agent` | Fully specified, AFK-ready (an agent can pick it up with no human context) |
| `ready-for-human` | `ready-for-human` | Needs human implementation or review |
| `wontfix` | `wontfix` | Will not be actioned |

## Triage Workflow

1. **New Issue** → `needs-triage`
2. **Evaluation** → `needs-info` (if clarification needed) OR `ready-for-agent` (if complete) OR `ready-for-human` (if human-only) OR `wontfix` (if rejected)
3. **Info Request** → `needs-info` → back to `needs-triage` when info provided
4. **Agent Work** → `ready-for-agent` → `ready-for-human` (if human handoff needed) OR completed
5. **Human Work** → `ready-for-human` → completed

## Usage in Skills

- `triage`: Automatically applies these labels based on issue analysis
- `to-issues`: Can set initial labels when creating issues
- Other skills can read these labels to determine issue state

## Customization

To modify label names, update the "Label String" column in the table above. Skills will use the exact strings specified here when applying labels to issues.
