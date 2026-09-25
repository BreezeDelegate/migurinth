# Migurinth fork maintenance

## Upstream model

Migurinth is maintained as a privacy-oriented fork of the Modrinth App.

Canonical upstreams:

- Product upstream: `https://github.com/modrinth/code.git` (`source` remote)
- Historical Migurinth upstream: `https://github.com/MiguVT/migurinth.git` (`parent` remote)
- This fork: `https://github.com/BreezeDelegate/migurinth.git` (`origin` remote)

The 2026-09-25 refresh rebased the maintained fork behavior onto current Modrinth `main`
(`722cc3504edf7e5d00f1cef90733ef127515353a`). The launcher-relevant code was last changed
in the audited baseline `e977cc867165a26837d53b1403851b56943f7c0d` (three commits after `v0.21.5`);
the two later upstream commits only touch Labrinth/backend files.
Do not merge the old Migurinth branch forward wholesale: Modrinth refactored many of the
same files, and a direct merge produced 32 conflicts. Instead, start from current Modrinth
`main` and reapply the invariants below explicitly.

## Fork invariants

Keep these behaviors when syncing from Modrinth:

1. Migurinth product name, application identifier, icons, user agent, and visible branding.
2. Offline Minecraft accounts with 3-16 character ASCII usernames (`A-Z`, `a-z`, `0-9`, `_`).
3. Portable data mode using `MigurinthData` next to the executable when enabled.
4. No Modrinth ads window or ads-consent bridge.
5. No PostHog analytics and no Sentry client reporting in the launcher frontend.
6. No survey bootstrap or third-party Tally script.
7. Updater endpoint must use `BreezeDelegate/migurinth` releases and the BreezeDelegate
   Tauri updater public key. The private updater key must never be committed.
8. Preserve dependency overrides in `pnpm-workspace.yaml` that keep the launcher frontend
   clear of known npm advisories.

## Sync procedure

1. `git fetch source --tags --prune`
2. Create a new branch from `source/main`.
3. Reapply the fork invariants above; do not resolve an old merge by choosing the entire
   Migurinth side of refactored files.
4. Run `pnpm install --frozen-lockfile` with the Node version in `.nvmrc` and pnpm version
   in the root `packageManager` field.
5. Run the launcher frontend typecheck/build.
6. Copy `packages/app-lib/.env.prod` to `packages/app-lib/.env` for Rust/Tauri builds.
7. Run the offline-account unit test and Gradle `clean test shadowJar`.
8. Run dependency, secret, binary-payload, and custom-delta scans before producing release
   bundles.
9. Sign updater artifacts only with the private key stored outside the repository.

## Windows release from the canonical Linux VPS

GitHub-hosted Windows compute is not required. The canonical Linux VPS builds the official Tauri
Windows bundle with MSVC compatibility through `cargo-xwin`; Tauri then creates its normal NSIS
installer and v1-compatible updater archive (`.nsis.zip`) using the existing project configuration
and hooks.

Prerequisites on the VPS are the Rust `x86_64-pc-windows-msvc` target, `cargo-xwin`, LLVM/Clang,
and NSIS. Debian exposes `clang-cl` as a versioned binary, so the canonical environment provides a
user-local `clang-cl` alias without modifying the system toolchain.

Release sequence:

1. copy `packages/app-lib/.env.prod` to `packages/app-lib/.env` for the build only;
2. run Tauri with `cargo-xwin` as the runner, target `x86_64-pc-windows-msvc`, and an override that
   sets `bundle.targets` to `['nsis']` while retaining `tauri-release.conf.json`;
3. require the official Tauri outputs: `Migurinth.exe`, the NSIS setup, the `.nsis.zip` updater
   artifact, and both updater signatures;
4. verify both signatures with Minisign against the updater public key committed in
   `tauri-release.conf.json`;
5. smoke the official setup in a disposable Wine prefix with WebView2 pre-registered. Wine cannot
   run Microsoft's native WebView2 bootstrapper, so a normal first-install WebView2 bootstrap is not
   a meaningful Wine compatibility test; with that prerequisite stubbed, installation, registry
   associations, payload hash, and uninstallation must all pass.

The updater signatures authenticate the release to Migurinth. They are not Microsoft Authenticode
signatures, so Windows may still present normal publisher/SmartScreen warnings for an unsigned
independent build. Do not reintroduce Modrinth's DigiCert credentials or other upstream private
signing secrets into this fork.

## Public contributions

Upstreamable fixes must be isolated from Migurinth-specific branding/privacy changes.
The first cleanup found during the 2026-09-25 audit is proposed independently as
`modrinth/code#7702` (remove stale committed Java `.class` artifacts).
