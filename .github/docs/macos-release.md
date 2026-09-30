# macOS release requirements

The release workflow builds separate native installers for Apple silicon and Intel Macs, Windows x64 and Linux x64. Tag builds create a draft only after all four platforms pass. Tags must match the existing version manifests.

macOS downloads need a valid Developer ID Application signature and Apple notarization. Ad-hoc signatures are sufficient for preview integrity checks but do not establish Gatekeeper trust for public downloads. See [Tauri's macOS signing guide](https://v2.tauri.app/distribute/sign/macos/).

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

Pull requests touching release files and manual workflow runs build preview packages with ad-hoc macOS signatures. They never create a release. Select `signed_macos` on a manual run to exercise signing and notarization before tagging. Preview packages still require normal macOS approval and are not a replacement for signed public releases.

Missing Apple credentials fail tag builds with their required secret names. The workflow never silently publishes unsigned macOS installers. Existing v0.2.0/v0.2.1 downloads are not repaired by changing the workflow: users need a newly signed and notarized build. The automated checks complement installation testing on an actual Mac.
