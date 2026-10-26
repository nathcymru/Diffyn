# Security policy

## Report a vulnerability privately

**[Report a vulnerability](https://github.com/nathcymru/Diffyn/security/advisories/new)**
through GitHub's private reporting form.

- **Keep exploit details private** while maintainers assess the report.
- **Include the affected version or commit** and the relevant operating system.
- **Provide a minimal reproduction** using synthetic data.
- **Explain the impact** and any conditions needed to reproduce it.
- **Remove credentials and personal information** from attachments and logs.

Ordinary bug reports and feature requests are **public**.
Use the private form for vulnerabilities or security-sensitive findings.

## Supported versions

| Development line | Security fixes |
| --- | --- |
| Current `main` branch, version `0.1.x` | Supported through updates to `main`. |
| Older commits and unsupported forks | Update to the current development line. |

Diffyn is under active development and does not provide a separate long-term
support release.

## Security boundaries

- **Transport:** local stdio connects a trusted client to one server process.
- **Inputs:** text and profile values receive bounded validation.
- **Resources:** exact URI matching selects text embedded at compile time.
- **Network:** the application makes no outbound model calls or runtime source fetches.
- **Storage:** the application does not persist submitted text or profiles.
- **Logging:** operational errors exclude submitted content.
- **Host responsibility:** the client controls model use, rendering and its own retention policies.

Process memory, operating-system crash dumps and the upstream client remain
outside the server's no-persistence guarantee.

## Security review and disclosure

- **Maintainers assess** reproducibility, impact and affected versions.
- **Fixes receive relevant tests** and are coordinated with the reporter where practical.
- **Public advisories describe** confirmed vulnerabilities and available mitigations.
- **Automated checks cover known patterns and advisories** without guaranteeing that every vulnerability or malicious dependency is detected.

See the [architecture](../docs/architecture.md) for data flow and the
[maintenance guide](../docs/maintenance.md) for automated checks.
