(() => {
  "use strict";
  const root = document.documentElement;
  root.classList.remove("no-js");
  const themeButton = document.getElementById("theme-toggle");
  const motionButton = document.getElementById("motion-toggle");
  const motionPreference = window.matchMedia("(prefers-reduced-motion: reduce)");

  // Store appearance preferences only. Example text and checkbox state stay in memory.
  function readPreference(key) {
    try { return localStorage.getItem(key); } catch { return null; }
  }
  function savePreference(key, value) {
    try { localStorage.setItem(key, value); } catch { /* The controls also work without storage. */ }
  }
  function setTheme(theme) {
    const muted = theme === "muted";
    root.dataset.theme = muted ? "muted" : "vivid";
    themeButton.setAttribute("aria-pressed", String(muted));
    themeButton.title = muted ? "Muted palette on. Select for vivid colours." : "Vivid palette on. Select for muted colours.";
    document.querySelector('meta[name="theme-color"]').content = muted ? "#faf7ef" : "#fff9ec";
  }
  let requestedMotion = readPreference("diffyn-motion") !== "off";
  function setMotion() {
    const paused = motionPreference.matches || !requestedMotion;
    root.dataset.motion = paused ? "off" : "on";
    motionButton.setAttribute("aria-pressed", String(paused));
    motionButton.disabled = motionPreference.matches;
    motionButton.title = motionPreference.matches ? "Animation is paused by your device's reduced-motion preference." : paused ? "Resume animation" : "Pause animation";
    motionButton.setAttribute("aria-label", motionPreference.matches ? "Animation paused by your device preference" : "Pause animation");
    document.getElementById("motion-symbol").textContent = paused ? "▷" : "Ⅱ";
    window.dispatchEvent(new Event("diffyn-motion"));
  }
  setTheme(readPreference("diffyn-theme"));
  setMotion();
  themeButton.addEventListener("click", () => {
    setTheme(root.dataset.theme === "vivid" ? "muted" : "vivid");
    savePreference("diffyn-theme", root.dataset.theme);
  });
  motionButton.addEventListener("click", () => {
    requestedMotion = !requestedMotion;
    setMotion();
    savePreference("diffyn-motion", requestedMotion ? "on" : "off");
  });
  motionPreference.addEventListener("change", setMotion);

  // Only peripheral scenery shifts; reading surfaces and keyboard targets stay still.
  const depthLayers = [...document.querySelectorAll("[data-depth]")];
  let depthFrame = 0;
  function updateDepth() {
    depthFrame = 0;
    const moving = root.dataset.motion === "on";
    const worldTop = document.getElementById("main").offsetTop;
    depthLayers.forEach((layer) => {
      const amount = Math.max(-24, Math.min(24, Number(layer.dataset.depth) || 0));
      const position = (layer.offsetTop + worldTop - window.scrollY) / Math.max(1, window.innerHeight);
      const offset = moving ? Math.max(-1, Math.min(1, position)) * amount : 0;
      layer.style.setProperty("--depth-shift", `${offset.toFixed(2)}px`);
    });
  }
  function scheduleDepth() {
    if (!depthFrame) depthFrame = window.requestAnimationFrame(updateDepth);
  }
  window.addEventListener("scroll", scheduleDepth, { passive: true });
  window.addEventListener("resize", scheduleDepth, { passive: true });
  window.addEventListener("diffyn-motion", scheduleDepth);
  updateDepth();

  // These fixed examples illustrate presentation choices; they do not run the Rust server.
  const demoButtons = [...document.querySelectorAll("[data-demo]")];
  function showDemo(name) {
    if (!["space", "steps", "meaning"].includes(name)) return;
    demoButtons.forEach((button) => button.setAttribute("aria-pressed", String(button.dataset.demo === name)));
    document.querySelectorAll(".demo-panel").forEach((panel) => { panel.hidden = panel.id !== `demo-${name}`; });
  }
  demoButtons.forEach((button) => button.addEventListener("click", () => showDemo(button.dataset.demo)));
  document.querySelectorAll("[data-open-demo]").forEach((link) => {
    link.addEventListener("click", (event) => {
      event.preventDefault();
      showDemo(link.dataset.openDemo);
      document.getElementById("playground").scrollIntoView({ block: "start" });
      demoButtons.find((button) => button.dataset.demo === link.dataset.openDemo).focus({ preventScroll: true });
    });
  });
  const spacingButton = document.getElementById("spacing-toggle");
  const readingSample = document.getElementById("reading-sample");
  const sampleChunks = ["Open the document. Check the title.", "Save your changes. Close the document."];
  spacingButton.addEventListener("click", () => {
    const roomy = spacingButton.getAttribute("aria-pressed") !== "true";
    const chunks = roomy ? sampleChunks : [sampleChunks.join(" ")];
    readingSample.replaceChildren(...chunks.map((text) => {
      const paragraph = document.createElement("p");
      paragraph.textContent = text;
      return paragraph;
    }));
    spacingButton.setAttribute("aria-pressed", String(roomy));
    spacingButton.textContent = roomy ? "Show as one paragraph ↔" : "Give the words more room ↔";
  });
  const checkboxes = [...document.querySelectorAll(".demo-checklist input")];
  function updateChecklist() {
    document.getElementById("checklist-progress").textContent = `${checkboxes.filter((box) => box.checked).length} of ${checkboxes.length} steps checked`;
  }
  checkboxes.forEach((checkbox) => checkbox.addEventListener("change", updateChecklist));
  document.getElementById("reset-checklist").addEventListener("click", () => {
    checkboxes.forEach((checkbox) => { checkbox.checked = false; });
    updateChecklist();
  });
  const reminderButton = document.getElementById("pause-note");
  const reminder = document.getElementById("desk-reminder");
  reminderButton.setAttribute("aria-expanded", "false");
  reminderButton.setAttribute("aria-controls", "desk-reminder");
  reminderButton.addEventListener("click", () => {
    reminder.hidden = !reminder.hidden;
    reminderButton.setAttribute("aria-expanded", String(!reminder.hidden));
  });

  const status = document.getElementById("copy-status");
  const dialog = document.getElementById("copy-dialog");
  const manualText = document.getElementById("manual-copy");
  let statusTimer;
  let lastCopyButton;
  function announce(message) {
    window.clearTimeout(statusTimer);
    status.textContent = message;
    status.classList.add("is-visible");
    statusTimer = window.setTimeout(() => {
      status.classList.remove("is-visible");
      status.textContent = "";
    }, 6000);
  }
  function selectManualText() {
    manualText.focus();
    manualText.select();
    manualText.setSelectionRange(0, manualText.value.length);
  }
  function showManualCopy(text, label) {
    manualText.value = text;
    manualText.setAttribute("aria-label", `${label} to copy`);
    dialog.showModal();
    selectManualText();
  }
  document.querySelectorAll("[data-copy]").forEach((button) => {
    button.addEventListener("click", async () => {
      const source = document.getElementById(button.dataset.copy);
      if (!source) return;
      const text = source.textContent;
      const label = button.dataset.copyLabel || "Text";
      lastCopyButton = button;
      button.disabled = true;
      try {
        if (!navigator.clipboard || !window.isSecureContext) throw new Error("Clipboard unavailable");
        await navigator.clipboard.writeText(text);
        announce(`${label} copied. Paste it when you are ready.`);
      } catch {
        showManualCopy(text, label);
      } finally {
        button.disabled = false;
      }
    });
  });
  document.getElementById("select-copy-text").addEventListener("click", selectManualText);
  dialog.addEventListener("close", () => { if (lastCopyButton) lastCopyButton.focus(); });
})();
