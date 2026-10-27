import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { runInNewContext } from 'node:vm';
import { parse } from 'yaml';

const workflow = parse(
  readFileSync(new URL('../../.github/workflows/labeling.yml', import.meta.url), 'utf8'),
);
const step = workflow.jobs.labels.steps[0];

async function labelFiles(files, { changedFiles = files.length, listError, writeError } = {}) {
  const writes = [];
  const calls = [];
  const context = {
    repo: { owner: 'example', repo: 'project' },
    payload: { pull_request: { number: 42, title: '${process.exit(1)}' } },
  };
  const github = {
    rest: {
      pulls: {
        async get(args) {
          calls.push('get');
          assert.equal(args.pull_number, 42);
          return { data: { changed_files: changedFiles } };
        },
        listFiles: Symbol('listFiles'),
      },
      issues: {
        async addLabels(args) {
          calls.push('addLabels');
          if (writeError) throw writeError;
          writes.push(JSON.parse(JSON.stringify(args)));
        },
      },
    },
    async paginate(method, args) {
      calls.push('paginate');
      assert.equal(method, github.rest.pulls.listFiles);
      assert.equal(args.owner, 'example');
      assert.equal(args.repo, 'project');
      assert.equal(args.pull_number, 42);
      assert.equal(args.per_page, 100);
      if (listError) throw listError;
      return files;
    },
  };
  await runInNewContext(`(async () => {\n${step.with.script}\n})()`, { github, context });
  return { writes, calls };
}

test('labeling executes trusted inline code without checkout or PR expression interpolation', () => {
  assert.deepEqual(Object.keys(workflow.on), ['pull_request_target']);
  assert.deepEqual(workflow.permissions, { contents: 'read', 'pull-requests': 'write' });
  assert.equal(workflow.jobs.labels.steps.length, 1);
  assert.match(step.uses, /^actions\/github-script@[0-9a-f]{40}$/);
  assert.ok(
    !step.with.script.includes('${{'),
    'PR values are read as data, not interpolated into code',
  );
});

test('release work gets useful area labels without altering human labels or milestones', async () => {
  const result = await labelFiles([
    { filename: 'docs/releases/v2.0.0.md' },
    { filename: '.github/workflows/release.yml' },
    { filename: 'Cargo.lock' },
  ]);
  assert.deepEqual(result.writes, [
    {
      owner: 'example',
      repo: 'project',
      issue_number: 42,
      labels: ['documentation', 'ci', 'dependencies', 'release'],
    },
  ]);
  assert.deepEqual(result.calls, ['get', 'paginate', 'addLabels']);
});

test('rename detection retains the area of the removed path', async () => {
  const result = await labelFiles([
    { filename: 'README.old', previous_filename: 'docs/obsolete.md' },
    { filename: '.github/CONTRIBUTING.md' },
    { filename: 'skills/diffyn-task-support/SKILL.md' },
  ]);
  assert.deepEqual(result.writes[0].labels, ['documentation']);
});

test('dependency and container paths map to their relevant areas', async () => {
  const result = await labelFiles([
    { filename: 'fuzz/Cargo.toml' },
    { filename: 'scripts/docs/package-lock.json' },
    { filename: '.github/dependabot.yml' },
    { filename: 'Dockerfile' },
    { filename: '.dockerignore' },
  ]);
  assert.deepEqual(result.writes[0].labels, ['ci', 'dependencies', 'release']);
});

test('unrelated code and submitted strings cannot select arbitrary labels or run code', async () => {
  const result = await labelFiles([
    { filename: 'src/tools/layout.rs' },
    { filename: 'README.md.exe' },
    { filename: 'security' },
    { filename: '${process.exit(1)}' },
    { filename: '"; throw new Error("unexpected"); //' },
    { filename: '$(touch /tmp/unexpected)' },
  ]);
  assert.deepEqual(result.writes, []);
});

test('all paginated files contribute to the selected labels', async () => {
  const files = Array.from({ length: 100 }, (_, index) => ({ filename: `src/module${index}.rs` }));
  files.push({ filename: 'docs/architecture.md' });
  const result = await labelFiles(files);
  assert.deepEqual(result.writes[0].labels, ['documentation']);
});

test('incomplete file lists, excessive changes and API errors fail visibly', async () => {
  await assert.rejects(labelFiles([{ filename: 'README.md' }], { changedFiles: 2 }), /incomplete/);
  await assert.rejects(labelFiles([], { changedFiles: 3001 }), /API limit/);
  await assert.rejects(labelFiles([], { listError: new Error('read denied') }), /read denied/);
  await assert.rejects(
    labelFiles([{ filename: 'README.md' }], { writeError: new Error('write denied') }),
    /write denied/,
  );
});
