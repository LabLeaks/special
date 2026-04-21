# Trigger Evals

## Should Trigger

- We need to change how `special health` and `special arch` divide responsibility.
- Update the architecture so repo traceability is code-first and modules traceability is ownership-first.
- Find the real `@module` root tree and revise the authoritative architecture text before implementation.
- We changed cross-cutting analysis design; update the module tree honestly in tracked source.

## Should Not Trigger

- Add one new product claim for this CLI behavior.
- Check whether this single module is honestly implemented.
- Install a bundled skill into `.agents/skills/`.
- Fix a parser bug without changing architecture intent.
