# Build runtime monitoring

**Harden-Runner monitors the Repeatable Linux build job** for unexpected network destinations and runtime activity. It runs before checkout in [Coverage and repeatability](../.github/workflows/verification.yml), alongside the existing comparison of two clean builds.

This monitoring runs on GitHub's hosted Ubuntu runner and is limited to that job. It is not included in the Diffyn executable and does not monitor people's MCP sessions.

## Configuration and access

The action is pinned to the full commit for [Harden-Runner v2.22.2](https://github.com/step-security/harden-runner/releases/tag/v2.22.2). Dependabot proposes updates through the existing dependency workflow.

- **Audit mode:** records outbound destinations without a project-specific allowlist.
- **Workflow permissions:** `contents: read`, using the job's temporary GitHub token.
- **Credentials:** no StepSecurity API key, repository secret or OIDC permission is configured.
- **Checks:** the existing Repeatable Linux build check remains the required check.

Repository administrators must allow `step-security/harden-runner@*` alongside the existing selected actions while retaining the requirement for full commit pins.

**Audit mode still applies StepSecurity's global blocklist.** The provider updates this list independently of action releases, and a blocked dependency endpoint can interrupt a build; a project allowlist cannot override it. Inspect the detection and contact the provider if the destination appears to have been blocked incorrectly. See [global blocklist behaviour](https://docs.stepsecurity.io/github-actions/harden-runner#global-block-list).

## Public telemetry

**Community insights are public**, including to people without a StepSecurity account. Runtime records can show outbound destinations, process and file activity, and the associated workflow steps. Use public-safe build inputs and keep credentials, private conversations and personal information out of command arguments and fixtures. The provider explains [public Community insights](https://www.stepsecurity.io/blog/behind-the-scenes-how-stepsecurity-detected-and-helped-remediate-the-largest-npm-supply-chain-attack) and its [data handling](https://app.stepsecurity.io/privacy).

Telemetry is enabled for audit mode; the pinned action supports disabling telemetry only in block mode. Changing that setting requires a reviewed network policy and an updated explanation of the evidence available to maintainers. See the [action inputs](https://github.com/step-security/harden-runner/blob/dc2769abcc0fcfbadfc1df54081c1608579ef6b2/action.yml).

## Community plan and GitHub App

The [Community plan](https://www.stepsecurity.io/pricing) is free for public repositories on GitHub-hosted runners, with no credit card required. Private repositories, self-hosted runners and advanced policy or GitHub Checks features require the paid plan. Moving this job to another environment requires a fresh compatibility and cost review.

The workflow action and [StepSecurity Actions Security app](https://github.com/marketplace/harden-runner-app) have separate permissions. The public hosted-runner audit works without installing the app. For an app installation, select **only Diffyn** and review the actual authorization screen, which can request more access than the Marketplace summary.

The app is installed with Diffyn as its only selected repository. GitHub lists the subscription as **Harden Runner Community, $0.00**. The provider also displays an introductory enterprise trial; this project uses Community runtime monitoring and does not depend on trial-only controls or paid GitHub Checks.

The installation screen reviewed on 27 October 2026 requested:

| Scope | Access |
| --- | --- |
| Actions, repository metadata and pull requests | Read |
| Repository secret metadata | Read |
| Checks | Write |
| Account email addresses | Read |

Secret metadata includes names and timestamps, not secret values. Account email access applies to the authorizing account and is separate from selecting a repository. Review any later permission expansion before accepting it.

## Review a run

1. **Open the completed job** and check the Harden-Runner step and job summary for its insights link.
2. **Confirm monitoring started**, since a passing build alone does not establish that telemetry was collected.
3. **Inspect new destinations and detections** against the checkout, container registry, Debian snapshot, Rust toolchain, Cargo dependencies and artifact upload in that run.
4. **Retain the build comparison result** as the evidence that both clean builds produced identical executable bytes.

Runtime observations complement the [verification checks](verification.md); they do not establish that every dependency or build is safe. Missing insights or an agent setup problem should be investigated even when the build succeeds.
