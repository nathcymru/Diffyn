// Development-only, offline checks. Parsing HTML never executes its scripts.
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, realpathSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import MarkdownIt from 'markdown-it';
import GithubSlugger from 'github-slugger';
import { parseFragment } from 'parse5';
import { parseDocument } from 'yaml';
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import { Cite, plugins } from '@citation-js/core';
import '@citation-js/plugin-bibtex';
import { JSDOM } from 'jsdom';

const root = realpathSync(fileURLToPath(new URL('../..', import.meta.url)));
const errors = [];
const markdown = new MarkdownIt({ html: true });
const files = [...new Set(execFileSync('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard'], {
  cwd: root, encoding: 'utf8', maxBuffer: 8 * 1024 * 1024,
}).split('\0').filter(Boolean))].filter(file => existsSync(path.join(root, file)));
const documents = new Map();
const diagrams = [];
const report = (file, message) => errors.push(`${file}: ${message}`);
const read = file => readFileSync(path.join(root, file), 'utf8');
const attr = (node, name) => node.attrs?.find(item => item.name === name)?.value;
const children = node => node.childNodes ?? [];
const text = node => node.nodeName === '#text' ? node.value : children(node).map(text).join('');

function visit(node, callback) {
  callback(node);
  children(node).forEach(child => visit(child, callback));
}

// Use rendered tokens for inline formatting, raw HTML anchors, and image URLs.
// GithubSlugger supplies GitHub-compatible duplicate/Unicode heading anchors.
function inspectMarkdown(file) {
  if (documents.has(file)) return documents.get(file);
  const source = read(file);
  const tokens = markdown.parse(source, {});
  const document = { anchors: new Set(), links: [], tokens };
  documents.set(file, document);
  const slugger = new GithubSlugger();
  visit(parseFragment(markdown.renderer.render(tokens, markdown.options, {})), node => {
    const id = attr(node, 'id') ?? (node.tagName === 'a' ? attr(node, 'name') : undefined);
    if (id) document.anchors.add(id);
    if (/^h[1-6]$/.test(node.tagName ?? '')) document.anchors.add(slugger.slug(text(node)));
    if (node.tagName === 'a' && attr(node, 'href') !== undefined) {
      document.links.push({ href: attr(node, 'href'), label: text(node) });
    }
    if (node.tagName === 'img') {
      if (attr(node, 'alt') === undefined) report(file, 'An image is missing its alt attribute.');
      if (attr(node, 'src')) document.links.push({ href: attr(node, 'src') });
    }
    if (node.tagName === 'source' && attr(node, 'srcset')) {
      for (const source of attr(node, 'srcset').split(',')) {
        document.links.push({ href: source.trim().split(/\s+/)[0] });
      }
    }
  });
  for (const token of tokens) {
    if (token.type === 'fence' && token.info.trim().split(/\s+/)[0] === 'mermaid') {
      diagrams.push({ file, line: token.map[0] + 1, source: token.content });
    }
  }
  return document;
}

function localReference(href) {
  // Embedded MCP guidance uses canonical URLs so links also work outside GitHub.
  // Resolve this repository's main-branch URLs against the pending checkout.
  const github = href.match(/^https:\/\/github\.com\/nathcymru\/Diffyn\/(?:blob|raw|tree)\/(?:refs\/heads\/)?main\/(.*)$/i);
  const raw = href.match(/^https:\/\/raw\.githubusercontent\.com\/nathcymru\/Diffyn\/(?:refs\/heads\/)?main\/(.*)$/i);
  if (github || raw) return `/${(github ?? raw)[1]}`;
  return href;
}

function checkLink(file, href) {
  href = localReference(href);
  // Network references are handled separately by the scheduled Lychee check.
  if (/^(?:[a-z][a-z0-9+.-]*:|\/\/)/i.test(href)) return;
  const [location, encodedFragment] = href.split('#', 2);
  let local;
  let fragment;
  try {
    local = decodeURIComponent(location.split('?', 1)[0]);
    fragment = decodeURIComponent(encodedFragment ?? '');
  } catch {
    report(file, `Invalid URL encoding: ${href}`);
    return;
  }
  const target = local === '' ? path.join(root, file)
    : local.startsWith('/') ? path.join(root, local.slice(1))
      : path.resolve(root, path.dirname(file), local);
  if (!existsSync(target)) {
    report(file, `Missing local target: ${href}`);
    return;
  }
  const resolved = realpathSync(target);
  if (resolved !== root && !resolved.startsWith(`${root}${path.sep}`)) {
    report(file, `Local target escapes the repository: ${href}`);
    return;
  }
  if (!fragment) return;
  let relative = path.relative(root, resolved);
  if (statSync(resolved).isDirectory()) {
    relative = path.join(relative, 'README.md');
    if (!existsSync(path.join(root, relative))) {
      report(file, `Directory fragment has no README: ${href}`);
      return;
    }
  }
  if (/\.md$/i.test(relative)) {
    const anchors = inspectMarkdown(relative).anchors;
    if (!anchors.has(fragment) && !anchors.has(fragment.replace(/^user-content-/, ''))) {
      report(file, `Missing heading or HTML anchor: ${href}`);
    }
  } else if (/^L\d+(?:-L\d+)?$/.test(fragment)) {
    const range = fragment.match(/\d+/g).map(Number);
    const lineCount = read(relative).split('\n').length;
    if (range.some(line => line < 1 || line > lineCount) || range[0] > range.at(-1)) {
      report(file, `Line anchor is outside the target file: ${href}`);
    }
  }
}

for (const file of files.filter(file => /\.md$/i.test(file))) inspectMarkdown(file);
for (const [file, document] of documents) {
  for (const { href } of document.links) checkLink(file, href);
}

// YAML 1.2 keeps CFF date strings intact and rejects duplicate mapping keys.
function readYaml(file) {
  const document = parseDocument(read(file), { uniqueKeys: true, version: '1.2' });
  if (document.errors.length) throw new Error(document.errors.map(error => error.message).join('; '));
  return document.toJS({ maxAliasCount: 100 });
}
for (const file of files.filter(file => /\.ya?ml$/i.test(file))) {
  try {
    const document = readYaml(file);
    if (file.startsWith('.github/workflows/')) {
      for (const job of Object.values(document.jobs ?? {})) {
        for (const step of job.steps ?? []) {
          if (step.uses && !/^[\w.-]+\/[\w./-]+@[a-f0-9]{40}$/.test(step.uses)) {
            report(file, `Action must use a full commit pin: ${step.uses}`);
          }
        }
      }
    }
  } catch (error) { report(file, error.message); }
}

try {
  const schema = JSON.parse(read('scripts/docs/vendor/cff-1.2.0.schema.json'));
  const ajv = new Ajv({ allErrors: true, strict: false });
  addFormats(ajv);
  const validate = ajv.compile(schema);
  if (!validate(readYaml('CITATION.cff'))) {
    report('CITATION.cff', ajv.errorsText(validate.errors));
  }
} catch (error) { report('CITATION.cff', error.message); }

let citationCount = 0;
try {
  const bibtex = plugins.config.get('@bibtex');
  bibtex.parse.strict = true;
  // Citation.js's strict map omits these standard BibTeX entry types.
  // A manual requires a title; misc has no mandatory BibTeX fields. Diffyn
  // additionally requires a title and source URL for every bibliography entry.
  bibtex.constants.required.bibtex.manual = ['title'];
  bibtex.constants.required.bibtex.misc = [];
  const bibliography = new Cite(read('docs/bibliography.bib'), { forceType: '@bibtex/text' }).data;
  citationCount = bibliography.length;
  const ids = new Set();
  if (!bibliography.length) report('docs/bibliography.bib', 'The bibliography has no entries.');
  for (const entry of bibliography) {
    if (!entry.id || ids.has(entry.id)) report('docs/bibliography.bib', `Missing or duplicate key: ${entry.id}`);
    ids.add(entry.id);
    if (!entry.title) report('docs/bibliography.bib', `Missing title for ${entry.id}`);
    if (!entry.URL || !/^https?:$/.test(new URL(entry.URL).protocol)) {
      report('docs/bibliography.bib', `Missing HTTP(S) source URL for ${entry.id}`);
    }
  }
  for (const [file, document] of documents) {
    for (const { href, label = '' } of document.links) {
      const reference = localReference(href).split(/[?#]/, 1)[0];
      const target = reference.startsWith('/') ? path.join(root, reference.slice(1))
        : path.resolve(root, path.dirname(file), reference);
      const key = label.trim();
      // Citation keys are single tokens; prose labels and the filename link
      // refer to the bibliography as a whole rather than to an individual entry.
      if (target === path.join(root, 'docs/bibliography.bib') &&
          /^[a-z0-9][a-z0-9._:-]*$/.test(key) &&
          !['bibliography', 'bibliography.bib'].includes(key) && !ids.has(key)) {
        report(file, `Unknown bibliography key: ${key}`);
      }
    }
  }
} catch (error) { report('docs/bibliography.bib', error.message); }

// Mermaid uses DOMPurify while parsing labels. JSDOM supplies an isolated DOM
// with scripts and external-resource loading disabled; diagrams are not rendered.
const dom = new JSDOM('');
globalThis.window = dom.window;
globalThis.document = dom.window.document;
const { default: mermaid } = await import('mermaid');
mermaid.initialize({ startOnLoad: false, securityLevel: 'strict' });
for (const file of files.filter(file => /\.(?:mmd|mermaid)$/i.test(file))) {
  diagrams.push({ file, line: 1, source: read(file) });
}
for (const { file, line, source } of diagrams) {
  try { await mermaid.parse(source); }
  catch (error) { report(`${file}:${line}`, `Invalid Mermaid: ${error.message}`); }
}
dom.window.close();

if (errors.length) {
  console.error(errors.join('\n'));
  process.exitCode = 1;
} else {
  console.log(`Validated ${documents.size} Markdown files, ${citationCount} bibliography entries, CITATION.cff, ${diagrams.length} Mermaid diagrams, local links/anchors, and YAML/action pins.`);
}
