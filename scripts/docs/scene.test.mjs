import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { JSDOM, VirtualConsole } from 'jsdom';

const script = readFileSync(new URL('../../docs/scene.js', import.meta.url), 'utf8');
const fixture = `<!doctype html><html data-motion="on" data-theme="vivid"><body>
  <section data-scene-room>
    <button data-scene-lamp aria-pressed="true" hidden>Desk lamp</button>
    <p data-scene-status role="status"></p>
    ${['tools', 'prompts', 'resources']
      .map(
        (name) => `
      <button data-scene-drawer aria-controls="${name}" aria-expanded="false" hidden>${name} drawer</button>
      <div id="${name}"><a id="${name}-link" href="#${name}">${name} content</a></div>
    `,
      )
      .join('')}
    <details data-scene-book><summary>Prompt book</summary><p>Use explicit preferences.</p></details>
    <div data-scene-depth="8" aria-hidden="true"></div>
    <div data-scene-depth="-200" aria-hidden="true"></div>
    <div data-scene-depth="NaN" aria-hidden="true"></div>
    <div id="reading" data-scene-depth="10"><p>Stable reading surface.</p></div>
    <div id="interactive" data-scene-depth="10" aria-hidden="true"><button>Stable target</button></div>
  </section>
</body></html>`;
const settle = () => new Promise((resolve) => setImmediate(resolve));

function scene(t, { motion = 'on', reduced = false, runScript = true, hash = '' } = {}) {
  const errors = [];
  const virtualConsole = new VirtualConsole();
  virtualConsole.on('jsdomError', (error) => errors.push(error));
  const dom = new JSDOM(fixture, {
    url: `https://nathcymru.github.io/Diffyn/${hash}`,
    runScripts: 'outside-only',
    pretendToBeVisual: true,
    virtualConsole,
  });
  const { window } = dom;
  const { document } = window;
  const root = document.documentElement;
  root.dataset.motion = motion;
  if (!runScript) root.classList.add('no-js');
  const query = (selector) => {
    const element = document.querySelector(selector);
    assert.ok(element, `Fixture includes ${selector}`);
    return element;
  };
  const room = query('[data-scene-room]');
  let rect = { top: 100, left: 0, width: 800, height: 600, right: 800, bottom: 700 };
  room.getBoundingClientRect = () => rect;
  [...room.querySelectorAll('[data-scene-depth]')].forEach((layer, index) => {
    layer.getBoundingClientRect = () => {
      const shift = Number.parseFloat(layer.style.getPropertyValue('--scene-y')) || 0;
      const top = rect.top + index * 120 + shift;
      return { top, bottom: top + 80, left: 0, right: 80, width: 80, height: 80 };
    };
  });
  window.innerHeight = 800;
  const media = new window.EventTarget();
  media.matches = reduced;
  window.matchMedia = (queryText) => {
    assert.equal(queryText, '(prefers-reduced-motion: reduce)');
    return media;
  };
  const frames = new Map();
  let nextFrame = 1;
  window.requestAnimationFrame = (callback) => {
    const id = nextFrame++;
    frames.set(id, callback);
    return id;
  };
  window.cancelAnimationFrame = (id) => frames.delete(id);
  t.after(() => {
    window.close();
    assert.deepEqual(errors, [], 'Scene interactions do not throw browser errors');
  });
  if (runScript)
    window.eval(`${script}\n//# sourceURL=${new URL('../../docs/scene.js', import.meta.url).href}`);
  return {
    window,
    document,
    root,
    room,
    query,
    pending: () => frames.size,
    render() {
      const callbacks = [...frames.values()];
      frames.clear();
      callbacks.forEach((callback) => callback(window.performance.now()));
    },
    pointer(x, y, type = 'mouse') {
      const event = new window.MouseEvent('pointermove', { clientX: x, clientY: y });
      Object.defineProperty(event, 'pointerType', { value: type });
      room.dispatchEvent(event);
    },
    scroll(top) {
      rect = { ...rect, top, bottom: top + rect.height };
      window.dispatchEvent(new window.Event('scroll'));
    },
    roomHeight(height) {
      rect = { ...rect, height, bottom: rect.top + height };
    },
    reduce(value) {
      media.matches = value;
      media.dispatchEvent(new window.Event('change'));
    },
    motion(value) {
      root.dataset.motion = value;
      window.dispatchEvent(new window.Event('diffyn-motion'));
    },
    offsets() {
      return [...document.querySelectorAll('[data-scene-depth][aria-hidden="true"]')]
        .filter((layer) => !layer.querySelector('button'))
        .map((layer) =>
          ['--scene-x', '--scene-y'].map((name) =>
            Number.parseFloat(layer.style.getPropertyValue(name)),
          ),
        );
    },
  };
}

test('lamp toggles only local glow and respects palette changes without persisting state', async (t) => {
  const p = scene(t);
  const lamp = p.query('[data-scene-lamp]');
  assert.equal(lamp.hidden, false);
  assert.equal(p.room.dataset.lighting, 'on');
  lamp.focus();
  lamp.click();
  assert.equal(p.room.dataset.lighting, 'off');
  assert.equal(lamp.getAttribute('aria-pressed'), 'false');
  assert.equal(p.query('[data-scene-status]').textContent, 'Desk lamp off.');
  assert.equal(p.document.activeElement, lamp);
  assert.equal(p.root.dataset.theme, 'vivid');
  p.root.dataset.theme = 'muted';
  await settle();
  assert.equal(p.room.dataset.sceneTheme, 'muted');
  assert.equal(p.room.dataset.lighting, 'off');
  lamp.click();
  assert.equal(p.room.dataset.lighting, 'on');
  assert.equal(lamp.textContent, 'Desk lamp');
  assert.equal(p.window.localStorage.length, 0);
});

test('three drawers disclose independent content and return focus before hiding it', (t) => {
  const p = scene(t);
  for (const name of ['tools', 'prompts', 'resources']) {
    assert.equal(p.query(`#${name}`).hidden, true);
    const button = p.query(`[aria-controls="${name}"]`);
    assert.equal(button.hidden, false);
    button.click();
    assert.equal(p.query(`#${name}`).hidden, false);
    assert.equal(button.getAttribute('aria-expanded'), 'true');
  }
  p.query('#tools-link').focus();
  const tools = p.query('[aria-controls="tools"]');
  tools.click();
  assert.equal(p.query('#tools').hidden, true);
  assert.equal(tools.getAttribute('aria-expanded'), 'false');
  assert.equal(p.document.activeElement, tools);
  assert.equal(p.query('#prompts').hidden, false);
  assert.equal(p.query('#resources').hidden, false);
  assert.equal(p.window.localStorage.length, 0);
});

test('fragment links reveal the containing drawer without moving keyboard focus', (t) => {
  const p = scene(t, { hash: '#prompts-link' });
  assert.equal(p.query('#prompts').hidden, false);
  assert.equal(p.query('#tools').hidden, true);
  const lamp = p.query('[data-scene-lamp]');
  lamp.focus();
  p.window.history.replaceState(null, '', '#resources-link');
  p.window.dispatchEvent(new p.window.Event('hashchange'));
  assert.equal(p.query('#resources').hidden, false);
  assert.equal(p.document.activeElement, lamp);
});

test('native books open with or without the engine and no-script content stays available', (t) => {
  for (const runScript of [true, false]) {
    const p = scene(t, { runScript });
    const book = p.query('[data-scene-book]');
    p.query('summary').click();
    assert.equal(book.open, true);
    p.query('summary').click();
    assert.equal(book.open, false);
    if (!runScript) {
      assert.equal(p.query('[data-scene-lamp]').hidden, true);
      for (const panel of p.document.querySelectorAll(
        '[id="tools"], [id="prompts"], [id="resources"]',
      )) {
        assert.equal(panel.hidden, false);
      }
      assert.equal(p.pending(), 0);
    }
  }
});

test('pointer and scroll depth are bounded, batched and restricted to decorative objects', (t) => {
  const p = scene(t);
  p.render();
  p.pointer(500, 500);
  p.pointer(800, 700);
  p.scroll(0);
  assert.equal(p.pending(), 1);
  p.render();
  assert.ok(p.offsets().some(([x, y]) => x !== 0 || y !== 0));
  assert.ok(
    p
      .offsets()
      .every(
        ([x, y]) =>
          Number.isFinite(x) && Math.abs(x) <= 12 && Number.isFinite(y) && Math.abs(y) <= 24,
      ),
  );
  assert.equal(p.query('#reading').style.cssText, '');
  assert.equal(p.query('#interactive').style.cssText, '');
  assert.equal(p.pending(), 0, 'The engine does not run a perpetual rendering loop');
  p.room.dispatchEvent(new p.window.Event('pointerleave'));
  p.render();
  assert.ok(p.offsets().every(([x]) => x === 0));
  p.scroll(100);
  p.render();
  p.pointer(800, 700, 'touch');
  assert.equal(p.pending(), 0, 'Touch gestures do not trigger pointer parallax');
});

test('pause and reduced motion immediately cancel queued depth while objects remain usable', (t) => {
  const p = scene(t);
  p.pointer(800, 700);
  p.render();
  assert.ok(p.offsets().some(([x]) => x !== 0));
  p.pointer(500, 500);
  assert.equal(p.pending(), 1);
  p.motion('off');
  assert.equal(p.pending(), 0);
  assert.ok(p.offsets().every(([x, y]) => x === 0 && y === 0));
  p.scroll(0);
  p.pointer(800, 700);
  assert.equal(p.pending(), 0);
  p.query('[data-scene-lamp]').click();
  p.query('[aria-controls="tools"]').click();
  assert.equal(p.room.dataset.lighting, 'off');
  assert.equal(p.query('#tools').hidden, false);
  p.motion('on');
  p.pointer(800, 700);
  p.reduce(true);
  assert.equal(p.pending(), 0);
  assert.ok(p.offsets().every(([x, y]) => x === 0 && y === 0));
  p.reduce(false);
  p.render();
  assert.ok(
    p.offsets().some(([, y]) => y !== 0),
    'Local scroll depth resumes only when allowed',
  );
});

test('hero objects respond to local scrolling in a tall room without render feedback drift', (t) => {
  const p = scene(t);
  p.roomHeight(6000);
  p.render();
  const initial = p.offsets();
  p.scroll(0);
  p.render();
  assert.notDeepEqual(
    p.offsets(),
    initial,
    'A visible object reacts before the middle of a long page',
  );
  const afterScroll = p.offsets();
  for (let repeat = 0; repeat < 5; repeat++) {
    p.window.dispatchEvent(new p.window.Event('resize'));
    p.render();
    assert.deepEqual(
      p.offsets(),
      afterScroll,
      'Reading the transformed rectangle must not accumulate displacement',
    );
  }
  p.pointer(400, 700);
  p.render();
  assert.notDeepEqual(
    p.offsets(),
    afterScroll,
    'Vertical pointer movement uses the visible viewport',
  );
});

test('initial reduced motion and a hidden document never start depth rendering', (t) => {
  const p = scene(t, { reduced: true });
  assert.equal(p.pending(), 0);
  assert.ok(p.offsets().every(([x, y]) => x === 0 && y === 0));
  p.reduce(false);
  p.pointer(800, 700);
  Object.defineProperty(p.document, 'hidden', { configurable: true, value: true });
  p.document.dispatchEvent(new p.window.Event('visibilitychange'));
  assert.equal(p.pending(), 0);
  assert.ok(p.offsets().every(([x, y]) => x === 0 && y === 0));
});
