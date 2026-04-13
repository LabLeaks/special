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
artifact: brew install LabLeaks/homebrew-tap/special
owner: gk
last_reviewed: 2026-04-12
result: confirmed local installation succeeded
installed_binary: /opt/homebrew/bin/special
*/
