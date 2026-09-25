# Security audit - 2026-09-25

## Scope

Audit target: launcher code that can ship in Migurinth (`apps/app`, `apps/app-frontend`,
`packages/app-lib`) plus the fork delta against current Modrinth `main`.

Final Modrinth base: `722cc3504edf7e5d00f1cef90733ef127515353a`. Launcher-relevant
source is unchanged from audited commit `e977cc867165a26837d53b1403851b56943f7c0d`; the two newer
commits only touch Labrinth/backend files.
Historical `MiguVT/migurinth` and `BreezeDelegate/migurinth` were identical before this
refresh. The historical fork had not integrated current Modrinth since early 2026.

## Results

- No executable/native payload in the repository was modified by the historical fork.
- No launcher-specific dynamic `eval`/`Function` execution, encrypted source blob, or
  obfuscated bootstrap was found in the fork delta. The production bundler does warn about
  a direct `eval` inside upstream third-party `ace-builds` CoffeeScript worker code; this is
  part of the file-editor dependency, not Migurinth-added code, and is tracked as upstream
  dependency surface rather than hidden fork behavior.
- Process execution in the launcher is explicit Rust/Java launcher functionality, not a
  hidden fork addition.
- Gitleaks found no private secret in `apps/app` or `packages/app-lib`. The only launcher
  frontend matches are Modrinth's public Stripe test publishable key (`pk_test_*`), which
  is client-side by design.
- The Tauri updater private key is stored outside Git with mode `0600`; only the public key
  and BreezeDelegate release URL are committed.
- PostHog, Sentry, survey/Tally bootstrap, and their unnecessary CSP permissions are removed/no-op in Migurinth.
- Migurinth 0.21.5-migurinth.2 removes the remaining launcher ad surface rather than merely no-oping it: the ad helper and promotion component are deleted, the consent event contract is removed and regenerated, Modrinth+ upsell UI is absent, ad page-context flags are fixed to false, and obsolete translated ad-consent strings are pruned.
- The Retro appearance theme is exposed to normal Migurinth users; no Modrinth server-side subscription or profile-badge entitlement is forged.
- A production pnpm audit filtered to `@modrinth/app-frontend` reports 0 advisories after
  Migurinth's dependency/privacy changes. The same audit on current Modrinth `main`
  reports 27 launcher-path advisories, including high/critical entries via PostHog and
  older SVGO/PostCSS/nanoid/fflate dependency paths.
- Thirteen committed Java `.class` files were discovered under
  `packages/app-lib/java/bin/main`. They are unused by the build (Gradle creates
  `theseus.jar` from `java/src`) and had drifted from the current source. They are removed
  and `/bin/` is now ignored. The generic upstream cleanup is proposed in
  `modrinth/code#7702`.
- `./gradlew clean test shadowJar --no-daemon` succeeds after deleting the stale classes.
- The Migurinth offline-username validation unit test passes.
- Launcher frontend TypeScript check and production Vite build pass on Node 24.15.0 / pnpm
  10.33.2.
- Windows `x86_64-pc-windows-msvc` cross-check and release build pass on the canonical Linux VPS
  through `cargo-xwin` and LLVM. Tauri itself produces the official `Migurinth.exe`, NSIS setup,
  `.nsis.zip` updater artifact, and updater signatures; no custom installer implementation is used.
- Both official Windows updater signatures and all four Linux updater signatures were independently
  verified with Minisign against the public key embedded in the release configuration.
- The official Windows NSIS setup was smoke-tested in a disposable Wine prefix with WebView2
  pre-registered: silent install returned 0, the installed executable matched the release binary
  SHA-256 exactly, `modrinth://` and `.mrpack` registration plus uninstall metadata were present,
  and silent uninstall removed the payload and uninstall registry key. A normal Wine first install
  returns code 2 specifically while attempting Microsoft's native WebView2 bootstrapper; this is a
  Wine/runtime limitation, not an installer payload failure.

## Limits

`cargo-audit` was not available preinstalled and compiling the audit tool itself exceeded
this session's command window. This audit therefore does not claim a completed RustSec
advisory scan. Rust dependencies are still built from the current upstream lockfile, and
no new third-party Rust dependency is introduced by the Migurinth delta.
