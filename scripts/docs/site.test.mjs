import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { JSDOM, VirtualConsole } from 'jsdom';

const html = readFileSync(new URL('../../docs/index.html', import.meta.url), 'utf8');
const script = readFileSync(new URL('../../docs/site.js', import.meta.url), 'utf8');
const sceneScript = readFileSync(new URL('../../docs/scene.js', import.meta.url), 'utf8');
const css = readFileSync(new URL('../../docs/styles.css', import.meta.url), 'utf8');
const installCommand = 'cargo install --git https://github.com/nathcymru/Diffyn --tag v0.1.0 --locked diffyn';
const configuration = {
  mcpServers: { diffyn: { command: '/absolute/path/to/diffyn', args: [] } },
};
const sample = 'Open the document. Check the title. Save your changes. Close the document.';
const settle = () => new Promise((resolve) => setImmediate(resolve));

// Load the shipped page and script; emulate only browser APIs absent from jsdom.
function page(t, { preferences = {}, reducedMotion = false, blockedStorage = false, clipboard = 'available', loadScript = true, loadScene = false, controlledFrames = false } = {}) {
  const errors = [];
  const virtualConsole = new VirtualConsole();
  virtualConsole.on('jsdomError', (error) => errors.push(error));
  const dom = new JSDOM(html, {
    url: 'https://nathcymru.github.io/Diffyn/',
    runScripts: 'outside-only',
    pretendToBeVisual: true,
    virtualConsole,
  });
  const { window } = dom;
  const { document } = window;
  const query = (selector) => {
    const element = document.querySelector(selector);
    assert.ok(element, `The page includes ${selector}`);
    return element;
  };
  const style = document.createElement('style');
  style.textContent = css;
  document.head.append(style);
  for (const [key, value] of Object.entries(preferences)) window.localStorage.setItem(key, value);
  if (blockedStorage) {
    Object.defineProperty(window, 'localStorage', {
      get() { throw new window.DOMException('Storage blocked', 'SecurityError'); },
    });
  }
  const media = new window.EventTarget();
  media.matches = reducedMotion;
  window.matchMedia = (queryText) => {
    assert.equal(queryText, '(prefers-reduced-motion: reduce)');
    return media;
  };
  const copied = [];
  Object.defineProperty(window, 'isSecureContext', { value: true });
  if (clipboard !== 'unavailable') {
    Object.defineProperty(window.navigator, 'clipboard', {
      value: { async writeText(text) {
        if (clipboard === 'denied') throw new window.DOMException('Permission denied', 'NotAllowedError');
        copied.push(text);
      } },
    });
  }
  const scrolled = [];
  window.HTMLElement.prototype.scrollIntoView = function () { scrolled.push(this.id); };
  const frames = new Map();
  let nextFrame = 1;
  if (controlledFrames) {
    window.requestAnimationFrame = (callback) => {
      const id = nextFrame++;
      frames.set(id, callback);
      return id;
    };
    window.cancelAnimationFrame = (id) => frames.delete(id);
  }
  window.HTMLDialogElement.prototype.showModal = function () { this.setAttribute('open', ''); };
  window.HTMLDialogElement.prototype.close = function () {
    this.removeAttribute('open');
    this.dispatchEvent(new window.Event('close'));
  };
  t.after(() => {
    window.close();
    assert.deepEqual(errors, [], 'Page interactions cause no uncaught browser errors');
  });
  if (loadScript) {
    window.eval(script);
    if (loadScene) window.eval(sceneScript);
  }
  return {
    window, document, query, copied, scrolled,
    root: document.documentElement,
    storage: () => Object.fromEntries(Object.entries(window.localStorage)),
    pendingFrames: () => frames.size,
    setReducedMotion(value) {
      media.matches = value;
      media.dispatchEvent(new window.Event('change'));
    },
  };
}

test('vivid and muted palettes remain light and persist across page loads', (t) => {
  const first = page(t);
  assert.equal(first.root.dataset.theme, 'vivid');
  assert.equal(first.window.getComputedStyle(first.root).colorScheme, 'light');
  first.query('#theme-toggle').click();
  assert.equal(first.root.dataset.theme, 'muted');
  assert.equal(first.query('#theme-toggle').getAttribute('aria-pressed'), 'true');
  assert.equal(first.query('meta[name="theme-color"]').content, '#faf7ef');
  assert.equal(first.window.getComputedStyle(first.root).colorScheme, 'light');
  assert.deepEqual(first.storage(), { 'diffyn-theme': 'muted' });
  const restored = page(t, { preferences: first.storage() });
  assert.equal(restored.root.dataset.theme, 'muted');
  restored.query('#theme-toggle').click();
  assert.equal(restored.root.dataset.theme, 'vivid');
  assert.equal(restored.query('#theme-toggle').getAttribute('aria-pressed'), 'false');
  assert.equal(restored.query('meta[name="theme-color"]').content, '#fff9ec');
  assert.deepEqual(restored.storage(), { 'diffyn-theme': 'vivid' });
  const legacy = page(t, { preferences: { 'diffyn-theme': 'dark' } });
  assert.equal(legacy.root.dataset.theme, 'vivid');
  assert.equal(legacy.window.getComputedStyle(legacy.root).colorScheme, 'light');
});

test('system reduced motion overrides stored animation and responds to device changes', (t) => {
  const ordinary = page(t);
  assert.equal(ordinary.root.dataset.motion, 'on');
  const p = page(t, { preferences: { 'diffyn-motion': 'on' }, reducedMotion: true });
  const button = p.query('#motion-toggle');
  assert.equal(p.root.dataset.motion, 'off');
  assert.equal(button.disabled, true);
  assert.equal(button.getAttribute('aria-pressed'), 'true');
  button.click();
  assert.equal(p.root.dataset.motion, 'off');
  p.setReducedMotion(false);
  assert.equal(p.root.dataset.motion, 'on');
  assert.equal(button.disabled, false);
  button.click();
  assert.equal(p.root.dataset.motion, 'off');
  assert.deepEqual(p.storage(), { 'diffyn-motion': 'off' });
  p.setReducedMotion(true);
  p.setReducedMotion(false);
  assert.equal(p.root.dataset.motion, 'off', 'Device changes preserve a manual pause');
  assert.equal(button.getAttribute('aria-pressed'), 'true');
  const restored = page(t, { preferences: p.storage() });
  assert.equal(restored.root.dataset.motion, 'off');
  assert.deepEqual(ordinary.storage(), {}, 'Default preferences do not write storage');
});

test('blocked storage leaves appearance, examples and copy controls usable', async (t) => {
  const p = page(t, { blockedStorage: true });
  p.query('#theme-toggle').click();
  p.query('#motion-toggle').click();
  p.query('[data-demo="meaning"]').click();
  assert.equal(p.root.dataset.theme, 'muted');
  assert.equal(p.root.dataset.motion, 'off');
  assert.equal(p.query('#demo-meaning').hidden, false);
  p.query('[data-copy="install-command"]').click();
  await settle();
  assert.deepEqual(p.copied, [installCommand]);
});

test('spacing changes paragraph breaks without changing any example words', (t) => {
  const p = page(t);
  const paragraphs = () => [...p.query('#reading-sample').querySelectorAll('p')].map((node) => node.textContent);
  assert.equal(paragraphs().length, 2);
  assert.equal(paragraphs().join(' '), sample);
  p.query('#spacing-toggle').click();
  assert.deepEqual(paragraphs(), [sample]);
  assert.equal(p.query('#spacing-toggle').getAttribute('aria-pressed'), 'false');
  p.query('#spacing-toggle').click();
  assert.equal(paragraphs().length, 2);
  assert.equal(paragraphs().join(' '), sample);
  assert.equal(p.query('#spacing-toggle').getAttribute('aria-pressed'), 'true');
  assert.deepEqual(p.storage(), {});
});

test('the shipped room integrates with appearance controls and drawer-to-demo navigation', async (t) => {
  const p = page(t, {
    loadScene: true,
    controlledFrames: true,
    preferences: { 'diffyn-theme': 'muted', 'diffyn-motion': 'off' },
  });
  const room = p.query('[data-scene-room]');
  assert.equal(room.dataset.sceneTheme, 'muted');
  assert.equal(p.pendingFrames(), 0);
  const drawers = [...p.document.querySelectorAll('[data-scene-drawer]')];
  assert.equal(drawers.length, 3);
  for (const drawer of drawers) {
    assert.equal(drawer.hidden, false);
    assert.equal(p.document.getElementById(drawer.getAttribute('aria-controls')).hidden, true);
  }
  const shortcut = p.query('[data-open-demo="meaning"]');
  const panel = shortcut.closest('.drawer-content');
  p.query(`[data-scene-drawer][aria-controls="${panel.id}"]`).click();
  assert.equal(panel.hidden, false);
  shortcut.click();
  assert.equal(p.query('#demo-meaning').hidden, false);
  assert.equal(p.document.activeElement, p.query('[data-demo="meaning"]'));
  assert.deepEqual(p.scrolled, ['playground']);
  p.query('[data-scene-lamp]').click();
  assert.equal(room.dataset.lighting, 'off');
  p.query('#theme-toggle').click();
  await settle();
  assert.equal(room.dataset.sceneTheme, 'vivid');
  assert.equal(room.dataset.lighting, 'off', 'Changing palette does not reset the lamp');
  p.query('#motion-toggle').click();
  assert.equal(p.pendingFrames(), 1, 'The preference event wakes the scene engine');
  p.query('#motion-toggle').click();
  assert.equal(p.pendingFrames(), 0, 'Pause cancels scene work synchronously');
  p.query('#motion-toggle').click();
  p.setReducedMotion(true);
  assert.equal(p.pendingFrames(), 0);
  assert.equal(p.query('#motion-toggle').disabled, true);
  assert.equal(p.root.dataset.motion, 'off');
  assert.deepEqual(p.storage(), { 'diffyn-theme': 'vivid', 'diffyn-motion': 'on' });
});

test('the shipped page keeps drawer content and native books available without scripts', (t) => {
  const p = page(t, { loadScript: false });
  const drawers = [...p.document.querySelectorAll('[data-scene-drawer]')];
  assert.equal(drawers.length, 3);
  for (const drawer of drawers) {
    assert.equal(drawer.hidden, true);
    assert.equal(p.document.getElementById(drawer.getAttribute('aria-controls')).hidden, false);
  }
  assert.equal(p.query('[data-scene-lamp]').hidden, true);
  const book = p.query('[data-scene-book]');
  book.querySelector('summary').click();
  assert.equal(book.open, true);
  assert.equal(p.query('#install-command').textContent, installCommand);
});

test('demo controls select one example and shortcut links move focus to its control', (t) => {
  const p = page(t);
  for (const name of ['steps', 'meaning', 'space']) {
    p.query(`[data-demo="${name}"]`).click();
    assert.deepEqual([...p.document.querySelectorAll('.demo-panel')].filter((panel) => !panel.hidden).map((panel) => panel.id), [`demo-${name}`]);
    assert.deepEqual([...p.document.querySelectorAll('[data-demo][aria-pressed="true"]')].map((button) => button.dataset.demo), [name]);
  }
  p.query('[data-open-demo="meaning"]').click();
  assert.equal(p.query('#demo-meaning').hidden, false);
  assert.equal(p.document.activeElement, p.query('[data-demo="meaning"]'));
  assert.deepEqual(p.scrolled, ['playground']);
});

test('checklist progress, reset and reminders work without storing personal state', (t) => {
  const p = page(t);
  p.query('[data-demo="steps"]').click();
  const boxes = [...p.document.querySelectorAll('.demo-checklist input')];
  boxes[0].click();
  boxes[2].click();
  assert.equal(p.query('#checklist-progress').textContent, '2 of 4 steps checked');
  boxes[0].click();
  assert.equal(p.query('#checklist-progress').textContent, '1 of 4 steps checked');
  p.query('#reset-checklist').click();
  assert.ok(boxes.every((box) => !box.checked));
  assert.equal(p.query('#checklist-progress').textContent, '0 of 4 steps checked');
  const button = p.query('#pause-note');
  assert.equal(button.getAttribute('aria-controls'), 'desk-reminder');
  for (const visible of [true, false]) {
    button.click();
    assert.equal(p.query('#desk-reminder').hidden, !visible);
    assert.equal(button.getAttribute('aria-expanded'), String(visible));
  }
  assert.deepEqual(p.storage(), {});
});

test('copy buttons copy the complete install command and valid MCP configuration', async (t) => {
  const p = page(t);
  for (const [id, label] of [['install-command', 'Install command'], ['mcp-config', 'MCP configuration']]) {
    const button = p.query(`[data-copy="${id}"]`);
    button.click();
    await settle();
    assert.equal(p.copied.at(-1), p.query(`#${id}`).textContent);
    assert.equal(button.disabled, false);
    assert.equal(p.query('#copy-status').textContent, `${label} copied. Paste it when you are ready.`);
    assert.equal(p.query('#copy-dialog').open, false);
  }
  assert.equal(p.copied[0], installCommand);
  assert.deepEqual(JSON.parse(p.copied[1]), configuration);
  assert.deepEqual(p.storage(), {});
});

for (const clipboard of ['denied', 'unavailable']) {
  test(`clipboard ${clipboard} opens a selected manual copy field and restores focus on close`, async (t) => {
    const p = page(t, { clipboard });
    const button = p.query('[data-copy="mcp-config"]');
    button.focus();
    button.click();
    await settle();
    const dialog = p.query('#copy-dialog');
    const field = p.query('#manual-copy');
    assert.equal(dialog.open, true);
    assert.equal(field.value, p.query('#mcp-config').textContent);
    assert.equal(field.getAttribute('aria-label'), 'MCP configuration to copy');
    assert.equal(p.document.activeElement, field);
    assert.equal(field.selectionStart, 0);
    assert.equal(field.selectionEnd, field.value.length);
    assert.equal(button.disabled, false);
    assert.deepEqual(p.copied, []);
    field.setSelectionRange(2, 2);
    p.query('#select-copy-text').click();
    assert.equal(field.selectionStart, 0);
    assert.equal(field.selectionEnd, field.value.length);
    dialog.close();
    assert.equal(dialog.open, false);
    assert.equal(p.document.activeElement, button);
  });
}
