'use strict';

const REQUIRED_CHECKS = [
  'Rust and MCP (ubuntu-latest)',
  'Rust and MCP (macos-latest)',
  'CodeQL (actions)',
  'CodeQL (javascript-typescript)',
  'CodeQL (python)',
  'CodeQL (rust)',
  'Rust dependency policy',
  'Documentation dependency audit',
  'Documentation integrity',
  'Source style',
  'Source coverage',
  'Repeatable Linux build',
  'Fuzz smoke (tools_json)',
  'Fuzz smoke (tools_text)',
];

const pause = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));

async function hasActiveRelease(github, repo, sha) {
  const runs = await github.paginate(github.rest.actions.listWorkflowRuns, {
    ...repo,
    workflow_id: 'release.yml',
    branch: 'main',
    event: 'workflow_dispatch',
    head_sha: sha,
    per_page: 100,
  });
  const active = new Set(['queued', 'in_progress', 'requested', 'waiting', 'pending']);
  return runs.some(
    (run) =>
      run.head_sha === sha &&
      run.head_branch === 'main' &&
      run.event === 'workflow_dispatch' &&
      active.has(run.status),
  );
}

async function findTagCommit(github, repo, tag) {
  let reference;
  try {
    reference = (await github.rest.git.getRef({ ...repo, ref: `tags/${tag}` })).data.object;
  } catch (error) {
    if (error.status === 404) return null;
    throw error;
  }
  for (let depth = 0; depth < 8; depth += 1) {
    if (reference.type === 'commit') return reference.sha;
    if (reference.type !== 'tag') throw new Error('Release tag does not identify a commit');
    reference = (await github.rest.git.getTag({ ...repo, tag_sha: reference.sha })).data.object;
  }
  throw new Error('Release tag nesting exceeds the supported limit');
}

async function startCandidate({ github, context, core, version, sha, sleep = pause }) {
  if (
    !/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/.test(version) ||
    !/^[0-9a-f]{40}$/.test(sha)
  ) {
    throw new Error('Expected a canonical stable version and full commit ID');
  }
  const repo = context.repo;
  const isCurrent = async () =>
    (await github.rest.git.getRef({ ...repo, ref: 'heads/main' })).data.object.sha === sha;
  if (!(await isCurrent())) {
    core.info('Main has advanced; its own CI completion will evaluate the next release.');
    return 'superseded';
  }
  const tag = `v${version}`;
  const taggedCommit = await findTagCommit(github, repo, tag);
  if (taggedCommit) {
    const comparison = (
      await github.rest.repos.compareCommitsWithBasehead({
        ...repo,
        basehead: `${taggedCommit}...${sha}`,
      })
    ).data.status;
    if (!['ahead', 'identical'].includes(comparison)) {
      throw new Error(`${tag} is not an ancestor of main; investigate without moving the tag`);
    }
    core.info(`${tag} already exists; its original release workflow owns publication.`);
    return 'already-tagged';
  }
  for (let attempt = 0; attempt < 30; attempt += 1) {
    const runs = await github.paginate(github.rest.checks.listForRef, {
      ...repo,
      ref: sha,
      per_page: 100,
      filter: 'latest',
    });
    const latest = new Map();
    for (const run of runs) {
      if (
        run.app?.slug === 'github-actions' &&
        REQUIRED_CHECKS.includes(run.name) &&
        (!latest.has(run.name) || latest.get(run.name).id < run.id)
      ) {
        latest.set(run.name, run);
      }
    }
    const failed = REQUIRED_CHECKS.filter((name) => {
      const run = latest.get(name);
      return run?.status === 'completed' && run.conclusion !== 'success';
    });
    if (failed.length) throw new Error(`Release blocked by: ${failed.join(', ')}`);
    if (REQUIRED_CHECKS.every((name) => latest.get(name)?.conclusion === 'success')) {
      if (!(await isCurrent())) return 'superseded';
      if (await hasActiveRelease(github, repo, sha)) {
        core.info(`${tag} already has an active release workflow for this main commit.`);
        return 'already-running';
      }
      await github.rest.actions.createWorkflowDispatch({
        ...repo,
        workflow_id: 'release.yml',
        ref: 'main',
        inputs: { release_commit: sha, expected_version: version },
      });
      core.info(`Dispatched ${tag} from verified main commit ${sha}.`);
      return 'dispatched';
    }
    await sleep(20000);
  }
  throw new Error(
    'Required main checks did not complete within ten minutes; rerun after they finish',
  );
}

module.exports = { REQUIRED_CHECKS, findTagCommit, hasActiveRelease, startCandidate };
