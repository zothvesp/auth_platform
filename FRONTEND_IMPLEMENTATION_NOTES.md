# Frontend Implementation Notes

This document tracks useful discoveries that should influence upcoming frontend work, but are not necessarily part of the task that revealed them.

## Discovered Tradeoffs

- Mantine gives us accessible primitives, consistent form controls, notifications, loaders, and button states with less local code. The expected tradeoff was a larger initial CSS/JS chunk, so TanStack Router automatic route code-splitting is now enabled in `frontend/vite.config.ts`.

## UI Primitive Expectations

- Shared loading UI should go through `Spinner`, `InlineLoader`, or `PageLoader` from `frontend/src/components/ui`.
- Submit buttons should use the shared `Button` `state` API: `idle`, `loading`, `success`, or `error`.
- Feature work should prefer extending these small primitives before creating one-off button, loader, field, or toast patterns.

## Known But Out Of Scope

- Replace remaining page-level inline styles gradually where they duplicate shared UI behavior.
- Keep Mantine adoption incremental; avoid wrapping every component until there is repeated local code to remove.
