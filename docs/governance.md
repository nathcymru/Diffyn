# Project governance

Diffyn is maintained by **Nathan ([nathcymru](https://github.com/nathcymru))**, the repository owner and sole current maintainer. Contributions and technical discussion are open to the community.

## Roles and responsibilities

| Role | Current holder | Responsibilities |
| --- | --- | --- |
| Maintainer and release manager | Nathan / `nathcymru` | Triage issues, assess proposed changes, maintain compatibility and documentation, review dependency updates, manage access, verify releases and maintain the roadmap |
| Security contact | Nathan / `nathcymru` | Handle private reports, investigate impact, coordinate fixes and advisories, and credit reporters according to the security policy |
| Contributors | Authors of individual contributions | Follow the contribution and conduct policies, certify contribution rights, provide appropriate tests and documentation, and address review findings |

Automated checks and review tools assist these roles. They do not hold project authority or count as independent human maintainers.

## Decisions and review

Routine changes use pull requests. Explain the problem, alternatives that materially affect users, resulting behavior and verification. Discuss significant interface or scope changes in an issue or Discussion before implementation; the maintainer records the decision and rationale publicly, except for private security reports or personal information.

The maintainer makes final acceptance and release decisions. Disagreement can be raised respectfully in the relevant discussion with evidence or a concrete alternative. The [Code of Conduct](../.github/CODE_OF_CONDUCT.md) applies throughout.

Required checks remain mandatory. Repository rules request one human approval and allow the owner a pull-request exception to that approval requirement. The exception follows the person merging the PR, so it also applies when the owner merges another contributor's change. It does not constitute independent human review or waive required checks. The [maintenance guide](maintenance.md) describes the current controls.

New maintainers may be appointed after sustained contributions demonstrate the technical knowledge, judgment and availability needed for the role. Record the appointment and responsibilities here, then grant only the required access.

## Continuity

The maintainer reports that a trusted contact is designated through [GitHub Account Successor](https://docs.github.com/en/repositories/creating-and-managing-repositories/access-to-repositories#about-successors). The maintainer's continuity declaration is recorded in the [OpenSSF application](https://www.bestpractices.dev/en/projects/15973). GitHub administers successor requests under its published process.

Diffyn currently has one active maintainer. Review the designated contact and the recovery instructions when repository ownership, distribution channels or access requirements change. A handover must account for issues, proposed changes, releases, GitHub Packages and the documentation deployment.

The source, issues and release instructions remain public. The MIT licence permits continued development in a fork, subject to its terms, but a fork does not itself transfer control of the existing repository or package coordinates.

## Related policies

- [Contribution requirements](../.github/CONTRIBUTING.md)
- [Security reporting and response](../.github/SECURITY.md)
- [Roadmap](roadmap.md)
- [Release process](releases/README.md)
