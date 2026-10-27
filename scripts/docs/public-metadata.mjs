/** Check the public metadata contracts that GitHub's schema checks do not cover. */
export function preferredReportErrors(citation) {
  const report = citation?.['preferred-citation'];
  if (report?.type !== 'report') return [];
  if (typeof report.institution?.name !== 'string' || !report.institution.name.trim()) {
    return ['A preferred report needs an explicit institution name for GitHub citation rendering.'];
  }
  return [];
}

export function fundingDestinationErrors(funding, repositoryUrl) {
  const destination = `${repositoryUrl}/blob/main/.github/DONATE.md`;
  if (!Array.isArray(funding?.custom) || !funding.custom.includes(destination)) {
    return ['The Sponsor button must link to the project charity donation page.'];
  }
  return [];
}
