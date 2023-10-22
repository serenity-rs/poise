Release guide:
- Write changelog (see CHANGELOG.md for template)
- Push version bump commit
  - Add changelog to CHANGELOG.md
  - Update /Cargo.toml version
  - Update /macros/Cargo.toml version
  - Update macros dependency version in /Cargo.toml
- Add version tag with `git tag v0.3.0` and `git push origin --tags`
- Make GitHub release based on new tag
- Release on crates.io with `cargo publish`, first in /macros, then in root
