import assert from 'node:assert/strict';
import test from 'node:test';
import candidate from '../release-candidate.cjs';

const sha = 'a'.repeat(40);
function fixture(overrides = {}) {
  const calls = [];
  const github = {
    rest: {
      git: {
        getRef: async ({ ref }) => {
          if (ref === 'heads/main') return { data: { object: { sha } } };
          throw Object.assign(new Error('missing'), { status: 404 });
        },
      },
      repos: { compareCommitsWithBasehead: async () => ({ data: { status: 'ahead' } }) },
      checks: { listForRef() {} },
      actions: {
        listWorkflowRuns() {},
        createWorkflowDispatch: async (request) => calls.push(request),
      },
    },
    paginate: async (method) =>
      method === github.rest.actions.listWorkflowRuns
        ? []
        : candidate.REQUIRED_CHECKS.map((name, id) => ({
            name,
            id,
            app: { slug: 'github-actions' },
            status: 'completed',
            conclusion: 'success',
          })),
  };
  return {
    github,
    calls,
    context: { repo: { owner: 'example', repo: 'project' } },
    core: { info() {} },
    sha,
    version: '2.1.0',
    sleep: async () => {},
    ...overrides,
  };
}

test('dispatches the exact main version after every required check succeeds', async () => {
  const data = fixture();
  assert.equal(await candidate.startCandidate(data), 'dispatched');
  assert.deepEqual(data.calls[0], {
    owner: 'example',
    repo: 'project',
    workflow_id: 'release.yml',
    ref: 'main',
    inputs: { release_commit: sha, expected_version: '2.1.0' },
  });
});

test('does not publish an ordinary change using an existing ancestor tag', async () => {
  const data = fixture();
  data.github.rest.git.getRef = async ({ ref }) => ({
    data: {
      object: {
        type: 'commit',
        sha: ref === 'heads/main' ? sha : 'b'.repeat(40),
      },
    },
  });
  assert.equal(await candidate.startCandidate(data), 'already-tagged');
  assert.equal(data.calls.length, 0);
  data.github.rest.repos.compareCommitsWithBasehead = async () => ({
    data: { status: 'diverged' },
  });
  await assert.rejects(candidate.startCandidate(data), /not an ancestor/);
});

test('rejects untrusted or ambiguous version and commit values', async () => {
  for (const version of ['02.1.0', '2.1.0-rc.1', '2.1.0\n', '$(command)'])
    await assert.rejects(candidate.startCandidate(fixture({ version })), /canonical/);
  await assert.rejects(candidate.startCandidate(fixture({ sha: 'main' })), /commit ID/);
});

test('skips superseded main commits, including a change during check polling', async () => {
  const data = fixture();
  let reads = 0;
  data.github.rest.git.getRef = async ({ ref }) => {
    if (ref !== 'heads/main') throw Object.assign(new Error('missing'), { status: 404 });
    return { data: { object: { sha: ++reads === 1 ? sha : 'b'.repeat(40) } } };
  };
  assert.equal(await candidate.startCandidate(data), 'superseded');
  assert.equal(await candidate.startCandidate(data), 'superseded');
  assert.equal(data.calls.length, 0);
});

test('waits for pending checks and ignores older runs and other apps', async () => {
  const data = fixture();
  const original = data.github.paginate;
  let polls = 0;
  data.github.paginate = async (method) => {
    const runs = await original(method);
    if (method === data.github.rest.actions.listWorkflowRuns) return runs;
    if (++polls === 1) runs[0] = { ...runs[0], status: 'in_progress', conclusion: null };
    runs.push({ ...runs[1], id: -1, conclusion: 'failure' });
    runs.push({ ...runs[2], id: 999, app: { slug: 'untrusted' }, conclusion: 'failure' });
    return runs;
  };
  assert.equal(await candidate.startCandidate(data), 'dispatched');
  assert.equal(polls, 2);
});

test('deduplicates queued and active manual or automatic releases for the same main commit', async () => {
  for (const status of ['queued', 'in_progress', 'requested', 'waiting', 'pending']) {
    const data = fixture();
    const original = data.github.paginate;
    data.github.paginate = async (method, parameters) => {
      if (method !== data.github.rest.actions.listWorkflowRuns) return original(method);
      assert.deepEqual(parameters, {
        owner: 'example',
        repo: 'project',
        workflow_id: 'release.yml',
        branch: 'main',
        event: 'workflow_dispatch',
        head_sha: sha,
        per_page: 100,
      });
      return [{ head_sha: sha, head_branch: 'main', event: 'workflow_dispatch', status }];
    };
    assert.equal(await candidate.startCandidate(data), 'already-running');
    assert.equal(data.calls.length, 0);
  }
});

test('completed releases and unrelated commits, branches or events do not suppress a new dispatch', async () => {
  const data = fixture();
  const original = data.github.paginate;
  const active = {
    head_sha: sha,
    head_branch: 'main',
    event: 'workflow_dispatch',
    status: 'queued',
  };
  data.github.paginate = async (method) =>
    method === data.github.rest.actions.listWorkflowRuns
      ? [
          { ...active, status: 'completed', conclusion: 'failure' },
          { ...active, head_sha: 'b'.repeat(40) },
          { ...active, head_branch: 'codex/package-validation' },
          { ...active, event: 'pull_request' },
        ]
      : original(method);
  assert.equal(await candidate.startCandidate(data), 'dispatched');
  assert.equal(data.calls.length, 1);
});

test('release-run lookup failures cannot bypass duplicate dispatch protection', async () => {
  const data = fixture();
  const original = data.github.paginate;
  data.github.paginate = async (method) => {
    if (method === data.github.rest.actions.listWorkflowRuns)
      throw new Error('registry unavailable');
    return original(method);
  };
  await assert.rejects(candidate.startCandidate(data), /registry unavailable/);
  assert.equal(data.calls.length, 0);
});

test('fails closed for failed, skipped or missing checks and API errors', async () => {
  for (const conclusion of ['failure', 'skipped', 'cancelled']) {
    const data = fixture();
    data.github.paginate = async () => [
      {
        name: candidate.REQUIRED_CHECKS[0],
        id: 1,
        app: { slug: 'github-actions' },
        status: 'completed',
        conclusion,
      },
    ];
    await assert.rejects(candidate.startCandidate(data), /Release blocked/);
    assert.equal(data.calls.length, 0);
  }
  const data = fixture();
  data.github.paginate = async () => [];
  await assert.rejects(candidate.startCandidate(data), /ten minutes/);
  data.github.rest.git.getRef = async () => {
    throw Object.assign(new Error('denied'), { status: 403 });
  };
  await assert.rejects(candidate.startCandidate(data), /denied/);
});

test('resolves annotated tags with bounded nesting and fails on invalid tag objects', async () => {
  const data = fixture();
  data.github.rest.git.getRef = async () => ({ data: { object: { type: 'tag', sha } } });
  data.github.rest.git.getTag = async () => ({ data: { object: { type: 'commit', sha } } });
  assert.equal(await candidate.findTagCommit(data.github, data.context.repo, 'v2.1.0'), sha);
  data.github.rest.git.getTag = async () => ({ data: { object: { type: 'tag', sha } } });
  await assert.rejects(
    candidate.findTagCommit(data.github, data.context.repo, 'v2.1.0'),
    /nesting/,
  );
  data.github.rest.git.getTag = async () => ({ data: { object: { type: 'tree', sha } } });
  await assert.rejects(
    candidate.findTagCommit(data.github, data.context.repo, 'v2.1.0'),
    /identify a commit/,
  );
});
