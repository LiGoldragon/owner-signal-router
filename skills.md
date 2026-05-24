# skills — owner-signal-router

Before editing, read:

- `~/primary/skills/contract-repo.md`
- `~/primary/skills/component-triad.md`
- `~/primary/skills/naming.md`
- `~/primary/skills/architecture-editor.md`
- `~/primary/skills/architectural-truth-tests.md`
- `~/primary/skills/nix-discipline.md`
- this repo's `ARCHITECTURE.md`
- `../signal-router/ARCHITECTURE.md`
- `../router/ARCHITECTURE.md`
- `../signal-persona-mind/ARCHITECTURE.md`

This repo owns only the owner-only PersonaRouter channel-policy
signal vocabulary. It contains no daemon, no database tables, no
actor runtime, no CLI parser, and no policy evaluation logic.
The caller is PersonaOrchestrate; do not document or implement Mind as
calling this contract directly.
