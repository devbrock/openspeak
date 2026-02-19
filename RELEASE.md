# OpenSpeak Release Checklist

## 1. Preflight

- Ensure `main` is green and tested locally.
- Confirm `cargo tauri build` succeeds locally.
- Confirm tray flow, global hotkey, overlay, and dictation still work.

## 2. Versioning

Update version in:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

Use the same semantic version in all three files.

## 3. Commit + tag

```bash
git add .
git commit -m "release: vX.Y.Z"
git push origin main
git tag vX.Y.Z
git push origin vX.Y.Z
```

## 4. GitHub Actions

- Open Actions tab and watch `Release` workflow.
- Verify the workflow finishes successfully.
- Open GitHub Releases and validate the generated release entry.

## 5. Artifact validation

Verify release assets include:

- macOS app bundle (`.app`)
- macOS installer disk image (`.dmg`)

Download and test install from the Releases tab:

1. Open `.dmg`
2. Drag OpenSpeak to `/Applications`
3. Launch app and run a dictation smoke test

## 6. Signing / notarization (recommended)

If you want the smoothest install experience, configure:

- `APPLE_SIGNING_IDENTITY`
- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

in repository Secrets before tagging.
