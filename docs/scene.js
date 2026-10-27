(() => {
  'use strict';

  // This layer adds room interactions only; preferences, examples and copy live in site.js.
  const root = document.documentElement;
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
  const scenes = [...document.querySelectorAll('[data-scene-room]')].map((room) => ({
    room,
    pointer: null,
    layers: [...room.querySelectorAll('[data-scene-depth]')].filter(
      (layer) =>
        layer.closest('[data-scene-room]') === room &&
        layer.getAttribute('aria-hidden') === 'true' &&
        !layer.matches('a, button, input, select, textarea, summary, [tabindex]') &&
        !layer.querySelector('a, button, input, select, textarea, summary, [tabindex]'),
    ),
  }));
  if (!scenes.length) return;

  let frame = 0;
  const verticalShifts = new WeakMap();
  const clamp = (value, limit) => Math.max(-limit, Math.min(limit, value));
  const motionAllowed = () =>
    root.dataset.motion === 'on' &&
    !root.classList.contains('no-js') &&
    !reducedMotion.matches &&
    !document.hidden;

  function clearDepth() {
    scenes.forEach((scene) => {
      scene.pointer = null;
      scene.layers.forEach((layer) => {
        layer.style.setProperty('--scene-x', '0px');
        layer.style.setProperty('--scene-y', '0px');
        verticalShifts.set(layer, 0);
      });
    });
  }

  function renderDepth() {
    frame = 0;
    if (!motionAllowed()) {
      clearDepth();
      return;
    }
    // Read geometry together before writing styles; the scene never moves a reading target.
    const viewportHeight = Math.max(1, window.innerHeight);
    const positions = scenes.flatMap((scene) => {
      const rect = scene.room.getBoundingClientRect();
      const visible = rect.bottom > 0 && rect.top < viewportHeight;
      const pointer = visible ? scene.pointer : null;
      const x = pointer ? clamp(((pointer.x - rect.left) / Math.max(1, rect.width)) * 2 - 1, 1) : 0;
      const y = pointer ? clamp((pointer.y / viewportHeight) * 2 - 1, 1) : 0;
      return scene.layers.map((layer) => {
        const layerRect = layer.getBoundingClientRect();
        // Remove our previous translation so successive renders cannot feed back into themselves.
        const previousY = verticalShifts.get(layer) || 0;
        const top = layerRect.top - previousY;
        const layerVisible = visible && layerRect.bottom - previousY > 0 && top < viewportHeight;
        return {
          layer,
          x: layerVisible ? x : 0,
          y: layerVisible ? y : 0,
          // A tall room or backdrop must not pin the whole hero to a clamped scroll position.
          scroll: layerVisible
            ? clamp(
                (viewportHeight / 2 - top - Math.min(layerRect.height, viewportHeight) / 2) /
                  viewportHeight,
                1,
              )
            : 0,
        };
      });
    });
    positions.forEach(({ layer, x, y, scroll }) => {
      const declaredDepth = Number(layer.dataset.sceneDepth);
      const depth = Number.isFinite(declaredDepth) ? clamp(declaredDepth, 12) : 0;
      const shiftY = clamp((y + scroll) * depth, 24).toFixed(2);
      layer.style.setProperty('--scene-x', `${(x * depth).toFixed(2)}px`);
      layer.style.setProperty('--scene-y', `${shiftY}px`);
      verticalShifts.set(layer, Number(shiftY));
    });
  }

  function scheduleDepth() {
    if (motionAllowed() && !frame) frame = window.requestAnimationFrame(renderDepth);
  }

  function syncPreferences() {
    scenes.forEach(({ room }) => {
      room.dataset.sceneTheme = root.dataset.theme === 'muted' ? 'muted' : 'vivid';
    });
    if (!motionAllowed()) {
      if (frame) window.cancelAnimationFrame(frame);
      frame = 0;
      clearDepth();
    } else {
      scheduleDepth();
    }
  }

  const drawers = [];
  scenes.forEach((scene) => {
    const { room } = scene;
    // Lamp state is transient and affects the illustration's glow, never theme or storage.
    const lamps = [...room.querySelectorAll('button[data-scene-lamp]')].filter(
      (button) => button.closest('[data-scene-room]') === room,
    );
    function setLamp(on, announce) {
      room.dataset.lighting = on ? 'on' : 'off';
      lamps.forEach((button) => button.setAttribute('aria-pressed', String(on)));
      const status = room.querySelector('[data-scene-status]');
      if (announce && status) status.textContent = on ? 'Desk lamp on.' : 'Desk lamp off.';
    }
    setLamp(lamps[0]?.getAttribute('aria-pressed') !== 'false', false);
    lamps.forEach((button) => {
      button.addEventListener('click', () => setLamp(room.dataset.lighting !== 'on', true));
      button.hidden = false;
    });

    room.querySelectorAll('button[data-scene-drawer]').forEach((button) => {
      if (button.closest('[data-scene-room]') !== room) return;
      const panel = document.getElementById(button.getAttribute('aria-controls'));
      // A missing or incorrectly scoped panel leaves its content alone and its button hidden.
      if (!panel || !room.contains(panel) || panel.contains(button)) return;
      function setOpen(open) {
        if (!open && panel.contains(document.activeElement)) button.focus({ preventScroll: true });
        button.setAttribute('aria-expanded', String(open));
        panel.hidden = !open;
        scheduleDepth();
      }
      button.addEventListener('click', () =>
        setOpen(button.getAttribute('aria-expanded') !== 'true'),
      );
      button.hidden = false;
      setOpen(button.getAttribute('aria-expanded') === 'true');
      drawers.push({ panel, setOpen });
    });

    // Details/summary provides the book's keyboard and no-script disclosure behavior natively.
    room.querySelectorAll('details[data-scene-book]').forEach((book) => {
      book.addEventListener('toggle', scheduleDepth);
    });
    room.addEventListener(
      'pointermove',
      (event) => {
        if (event.pointerType === 'touch' || !motionAllowed()) return;
        scene.pointer = { x: event.clientX, y: event.clientY };
        scheduleDepth();
      },
      { passive: true },
    );
    room.addEventListener(
      'pointerleave',
      () => {
        scene.pointer = null;
        scheduleDepth();
      },
      { passive: true },
    );
    room.addEventListener(
      'pointercancel',
      () => {
        scene.pointer = null;
        scheduleDepth();
      },
      { passive: true },
    );
    room.dataset.sceneReady = 'true';
  });

  function revealLinkedContent() {
    let id;
    try {
      id = decodeURIComponent(window.location.hash.slice(1));
    } catch {
      return;
    }
    const target = id ? document.getElementById(id) : null;
    if (target)
      drawers.forEach(({ panel, setOpen }) => {
        if (panel === target || panel.contains(target)) setOpen(true);
      });
  }

  window.addEventListener('scroll', scheduleDepth, { passive: true });
  window.addEventListener('resize', scheduleDepth, { passive: true });
  window.addEventListener('diffyn-motion', syncPreferences);
  window.addEventListener('hashchange', revealLinkedContent);
  document.addEventListener('visibilitychange', syncPreferences);
  reducedMotion.addEventListener('change', syncPreferences);
  new MutationObserver(syncPreferences).observe(root, {
    attributes: true,
    attributeFilter: ['data-motion', 'data-theme', 'class'],
  });
  revealLinkedContent();
  syncPreferences();
})();
