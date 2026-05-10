# Domain Documentation Configuration

## Layout: Single-Context

This repository uses a single-context layout for domain documentation.

## Structure

```
/
├── CONTEXT.md           # Project domain language and concepts (to be created)
├── docs/
│   ├── adr/            # Architectural Decision Records
│   └── agents/         # Agent configuration (this directory)
```

## Usage by Skills

### Skills that read `CONTEXT.md`:
- `improve-codebase-architecture`: Learns domain language to suggest meaningful refactoring
- `diagnose`: Uses domain context to understand bug impact and relationships
- `tdd`: Incorporates domain terminology into test descriptions and scenarios

### Skills that read `docs/adr/`:
- `improve-codebase-architecture`: Reviews past architectural decisions for context
- `diagnose`: Checks ADRs for relevant technical decisions affecting current issues
- `tdd`: Ensures tests align with established architectural patterns

## Consumer Rules

1. **Primary Context**: Always read `CONTEXT.md` from repository root
2. **ADR Discovery**: Search `docs/adr/` for relevant architectural decisions
3. **Language Consistency**: Use terminology from `CONTEXT.md` in all generated content
4. **Decision Awareness**: Reference relevant ADRs when making architectural suggestions

## Getting Started

To complete the setup:
1. Create `CONTEXT.md` with project domain language and key concepts
2. Add architectural decisions to `docs/adr/` as they're made
3. Keep `CONTEXT.md` updated as the domain evolves

## Benefits

- Single source of truth for domain language
- Consistent terminology across all agent interactions
- Historical context for architectural decisions
- Easy navigation and maintenance
