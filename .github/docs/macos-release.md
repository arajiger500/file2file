# macOS release requirements

The release workflow builds separate native installers for Apple silicon and Intel Macs, Windows x64 and Linux x64. Tag builds, release-related changes on main, and manual runs on main create a draft only after all four platforms pass. Turn off `create_release` on a manual run to build packages only. Tags must match the existing version manifests.

For macOS downloads trusted by Gatekeeper, configure a valid Developer ID Application signature and Apple notarization. Ad-hoc signatures are sufficient for preview integrity checks but do not establish Gatekeeper trust for public downloads. See [Tauri's macOS signing guide](https://v2.tauri.app/distribute/sign/macos/).

Configure these **repository Actions secrets** in GitHub Settings → Secrets and variables → Actions:

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64 of the exported Developer ID Application `.p12`, including its private key |
| `APPLE_CERTIFICATE_PASSWORD` | Password protecting that `.p12` |
| `APPLE_SIGNING_IDENTITY` | Full `Developer ID Application: … (TEAMID)` identity |
| `APPLE_ID` | Apple account used for notarization |
| `APPLE_PASSWORD` | App-specific password for that account, not its login password |
| `APPLE_TEAM_ID` | Ten-character Apple Developer team ID |

Credentials stay in repository secrets; never commit them. Tauri imports the signing certificate and performs notarization during packaging. Verification then checks the disk image, application signature, expected native architecture, installed copy, stapled ticket, Gatekeeper assessment and launch. A screenshot check waits up to 60 seconds for nonblank content inside the application window; merely keeping a process alive cannot pass verification. Verification logs and any available desktop screenshot are saved as Actions artifacts.

Pull requests build preview packages with ad-hoc macOS signatures and never create a release. Other builds automatically sign and notarize when all six Apple secrets are configured. Without complete credentials, they build ad-hoc installers and explicitly label the draft as unnotarized. Select `signed_macos` on a manual run to require signing; missing credentials then fail the build instead of falling back. Certificate variables stay unset for ad-hoc packaging because Tauri treats even empty certificate variables as an import request.

The draft release description is derived from the verification reports for both Mac architectures, so ad-hoc builds never claim notarization. A draft contains six installers and SHA-256 checksums; published releases are never replaced. Existing downloads are not repaired by changing the workflow. The automated checks complement installation testing on an actual Mac.

Ad-hoc installers do not establish publisher identity or Gatekeeper trust. After verifying the checksum and source, use the macOS Privacy & Security **Open Anyway** approval flow if available. Managed Macs may prohibit this; use a signed and notarized build or build from source. Do not disable Gatekeeper globally.
