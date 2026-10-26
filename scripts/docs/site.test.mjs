import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { JSDOM, VirtualConsole } from 'jsdom';

const html = readFileSync(new URL('../../docs/index.html', import.meta.url), 'utf8');
const script = readFileSync(new URL('../../docs/site.js', import.meta.url), 'utf8');
const css = readFileSync(new URL('../../docs/styles.css', import.meta.url), 'utf8');
const installCommand = 'cargo install --git https://github.com/nathcymru/Diffyn --tag v0.1.0 --locked diffyn';
const configuration = {
  mcpServers: { diffyn: { command: '/absolute/path/to/diffyn', args: [] } },
};
const sample = 'Open the document. Check the title. Save your changes. Close the document.';
const settle = () => new Promise((resolve) => setImmediate(resolve));

// Load the shipped page and script; emulate only browser APIs absent from jsdom.
function page(t, { preferences = {}, reducedMotion = false, blockedStorage = false, clipboard = 'available', loadScript = true, controlledFrames = false } = {}) {
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
  if (loadScript) window.eval(script);
  return {
    window, document, query, copied, scrolled,
    root: document.documentElement,
    storage: () => Object.fromEntries(Object.entries(window.localStorage)),
    pendingFrames: () => frames.size,
    renderFrame() {
      assert.ok(controlledFrames, 'The test controls the animation-frame boundary');
      const callbacks = [...frames.values()];
      frames.clear();
      callbacks.forEach((callback) => callback(window.performance.now()));
    },
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

// jsdom has no layout engine, so supply viewport geometry at the browser boundary.
function depthScene(p) {
  const layers = [...p.document.querySelectorAll('[data-depth]')];
  assert.ok(layers.length > 0, 'The page contains peripheral scenery');
  Object.defineProperty(p.query('#main'), 'offsetTop', { value: 120 });
  layers.forEach((layer, index) => Object.defineProperty(layer, 'offsetTop', { value: 280 + index * 200 }));
  p.window.innerHeight = 800;
  return {
    layers,
    shifts: () => layers.map((layer) => Number.parseFloat(layer.style.getPropertyValue('--depth-shift'))),
    scroll(y) {
      p.window.scrollY = y;
      p.window.dispatchEvent(new p.window.Event('scroll'));
    },
  };
}

test('scroll depth stays small, batches rendering and leaves reading controls stationary', (t) => {
  const p = page(t, { controlledFrames: true });
  const scene = depthScene(p);
  for (const layer of scene.layers) {
    assert.equal(layer.getAttribute('aria-hidden'), 'true');
    assert.equal(layer.querySelector('a, button, input, [tabindex]'), null);
  }
  for (const selector of ['#playground', '#hero-heading', '#quickstart']) {
    assert.equal(p.query(selector).closest('[data-depth]'), null);
  }
  scene.scroll(0);
  scene.scroll(50);
  scene.scroll(100);
  assert.equal(p.pendingFrames(), 1, 'A burst of scroll events schedules only one render');
  p.renderFrame();
  const first = scene.shifts();
  assert.ok(first.some((shift) => shift !== 0));
  assert.equal(p.pendingFrames(), 0, 'Depth does not start a perpetual animation loop');
  p.window.innerHeight = 400;
  p.window.dispatchEvent(new p.window.Event('resize'));
  p.renderFrame();
  assert.notDeepEqual(scene.shifts(), first, 'Depth responds to the current viewport');
  scene.scroll(100000);
  p.renderFrame();
  assert.ok(scene.shifts().every((shift) => Number.isFinite(shift) && Math.abs(shift) <= 24));
});

test('pause and system reduced motion reset depth even when a scroll frame is queued', (t) => {
  const p = page(t, { controlledFrames: true });
  const scene = depthScene(p);
  scene.scroll(100);
  p.renderFrame();
  assert.ok(scene.shifts().some((shift) => shift !== 0));
  scene.scroll(200);
  p.query('#motion-toggle').click();
  assert.ok(scene.layers.every((layer) => p.window.getComputedStyle(layer).translate === '0 0'), 'CSS stops visible depth before the queued render');
  p.renderFrame();
  assert.ok(scene.shifts().every((shift) => shift === 0));
  scene.scroll(300);
  p.renderFrame();
  assert.ok(scene.shifts().every((shift) => shift === 0), 'Scrolling while paused does not restore depth');
  p.query('#motion-toggle').click();
  p.renderFrame();
  assert.ok(scene.shifts().some((shift) => shift !== 0));
  p.setReducedMotion(true);
  p.renderFrame();
  assert.ok(scene.shifts().every((shift) => shift === 0));
  assert.equal(p.query('#motion-toggle').disabled, true);
  p.setReducedMotion(false);
  p.renderFrame();
  assert.ok(scene.shifts().some((shift) => shift !== 0));
});

test('without JavaScript depth is not initialized and installation remains readable', (t) => {
  const p = page(t, { loadScript: false });
  assert.equal(p.root.classList.contains('no-js'), true);
  assert.equal(p.window.getComputedStyle(p.root).scrollBehavior, 'auto');
  for (const element of p.document.querySelectorAll('[data-depth]')) {
    assert.equal(element.style.getPropertyValue('--depth-shift'), '');
  }
  assert.equal(p.window.getComputedStyle(p.query('.view-controls')).display, 'none');
  assert.equal(p.query('#install-command').textContent, installCommand, 'Installation stays readable without scripts');
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
