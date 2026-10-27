import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { parse } from 'yaml';
import { fundingDestinationErrors, preferredReportErrors } from './public-metadata.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const citation = parse(fs.readFileSync(path.join(root, 'CITATION.cff'), 'utf8'));
const funding = parse(fs.readFileSync(path.join(root, '.github/FUNDING.yml'), 'utf8'));

test('the published report has the institution required by GitHub citation rendering', () => {
  assert.deepEqual(preferredReportErrors(citation), []);
  // Before PR #2 the entity author had no affiliation and the report lacked
  // an institution. Ruby-CFF's report formatter failed on that combination.
  const historical = structuredClone(citation);
  delete historical['preferred-citation'].institution;
  assert.ok(historical['preferred-citation'].authors.some((author) => author.name));
  assert.equal(preferredReportErrors(historical).length, 1);
  for (const institution of [null, {}, { name: '' }, { name: '  ' }]) {
    historical['preferred-citation'].institution = institution;
    assert.equal(preferredReportErrors(historical).length, 1);
  }
});

test('report-specific rendering requirements do not alter other citation types', () => {
  assert.deepEqual(preferredReportErrors({}), []);
  assert.deepEqual(preferredReportErrors({ 'preferred-citation': { type: 'article' } }), []);
});

test('the Sponsor button reaches the published charity choices', () => {
  assert.deepEqual(fundingDestinationErrors(funding, citation['repository-code']), []);
  const destination = new URL(funding.custom.find((url) => url.endsWith('/.github/DONATE.md')));
  assert.equal(destination.protocol, 'https:');
  assert.equal(destination.hostname, 'github.com');
  assert.ok(fs.statSync(path.join(root, '.github/DONATE.md')).isFile());
  // Before d841526 the only URL was a YAML comment; custom was an empty list.
  const historical = parse(
    '# https://github.com/nathcymru/Diffyn/blob/main/.github/DONATE.md\ncustom: []\n',
  );
  assert.equal(fundingDestinationErrors(historical, citation['repository-code']).length, 1);
  for (const custom of [undefined, 'https://example.invalid', ['https://example.invalid']]) {
    assert.equal(fundingDestinationErrors({ custom }, citation['repository-code']).length, 1);
  }
});
