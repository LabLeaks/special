/**
@group SPECIAL.DISTRIBUTION.CRATES_IO
special crates.io package identity.
*/

/**
@spec SPECIAL.DISTRIBUTION.CRATES_IO.PACKAGE_NAME
special publishes the package as `special-cli`.
*/

/**
@spec SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME
special installs the `special` binary from the `special-cli` package.
*/

/**
@group SPECIAL.DISTRIBUTION.GITHUB_RELEASES
special GitHub release distribution.

@group SPECIAL.DISTRIBUTION.RELEASE_FLOW
special local release publication flow.
*/

/**
@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.REPOSITORY_URL
special release automation declares the `https://github.com/LabLeaks/special` repository URL.
*/

/**
@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.WORKFLOW
special keeps a GitHub Actions release workflow in `.github/workflows/release.yml`.
*/

/**
@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.PUBLISHED
special publishes GitHub Releases for versioned distribution.
*/

/**
@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.ARCHIVES
special GitHub release automation publishes versioned release archives for supported target platforms.
*/

/**
@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.CHECKSUMS
special GitHub release automation publishes checksums for its release archives.
*/

/**
@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.CHECKLIST
before publishing, the release script interactively confirms easy-to-forget release tasks such as updating `README.md` and `CHANGELOG.md`.
*/

/**
@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.YES_BYPASSES_CHECKLIST
the release script accepts `--yes` to bypass the interactive release checklist.
*/

/**
@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.DRY_RUN
the release script dry-run prints the planned checklist and publication commands without creating a tag, moving the main bookmark, pushing to origin, or updating Homebrew.
*/

/**
@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.MATCHES_MANIFEST_VERSION
the release script requires the requested tag version to exactly match the current `Cargo.toml` package version.
*/

/**
@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.PUSHES_MAIN_AND_TAG
the release script publishes the release revision by pushing both the `main` bookmark and the release tag to origin.
*/

/**
@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.VERIFIES_GITHUB_RELEASE
after pushing the release tag, the release script waits for the GitHub release artifacts to publish and verifies the release asset set.
*/

/**
@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.UPDATES_HOMEBREW
after the GitHub release is published, the release script updates the Homebrew tap formula for the current version and verifies the published formula against the release assets.
*/

/**
@group SPECIAL.DISTRIBUTION.HOMEBREW
special Homebrew distribution.
*/

/**
@spec SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA
special ships a Homebrew formula in LabLeaks/homebrew-tap.
*/

/**
@spec SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL
special installs the `special` binary from LabLeaks/homebrew-tap.
*/

/**
@attests SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL
artifact: brew upgrade special (confirmed local install for /opt/homebrew/bin/special at v0.2.0)
owner: gk
last_reviewed: 2026-04-13
*/
