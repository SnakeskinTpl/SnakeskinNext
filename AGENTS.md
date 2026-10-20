# Collaboration

- The author is building the language themselves and uses the assistant for consultation, reviews, and explicitly requested implementation work. Do not take over implementation or expand the scope of a requested change.
- Communicate with the author in Russian. Use English for code identifiers, comments, documentation in source files, diagnostics, and commit messages.
- Treat the Wiki as a draft written at different times. Ask about relevant ambiguities rather than assuming every example is a final specification. Current decisions by the author take precedence.
- For reviews, lead with concrete correctness issues and distinguish them from API choices and cosmetic suggestions. Do not edit code unless implementation is requested.

# Code and tests

- Keep comments short and focused on contracts or behavior that is not obvious from the code.
- Treat tests as documentation: separate setup, actions, assertions, and distinct scenario steps with blank lines.
- Prefer meaningful behavioral tests, including boundary cases and interactions between combinators. Avoid tests that merely repeat implementation details.
- Keep public re-exports explicit rather than using wildcard re-exports in the public facade.
- Follow the surrounding naming and API conventions. Keep unrelated changes out of the task.
- Run `cargo test` after behavioral changes and renames that affect module paths or APIs. Report failures and material limitations accurately.

# Git

- Use Conventional Commits: `type(scope): Description`, with an optional scope.
- Keep the type lowercase and start the description with a capital letter.
- Example: `feat(parsec): Implement parser combinator core`.
- Common types include `feat`, `fix`, `refactor`, `test`, `docs`, and `chore`.
- Commit only work within the authorized scope. Preserve unrelated working tree and staged changes.
- Do not create commits, rewrite history, or push unless requested by the author.
