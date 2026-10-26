(() => {
  "use strict";

  document.documentElement.classList.remove("no-js");

  // This page only copies user-selected setup text; it makes no network calls.
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
        if (!navigator.clipboard || !window.isSecureContext) {
          throw new Error("Clipboard unavailable");
        }
        await navigator.clipboard.writeText(text);
        announce(`${label} copied. Paste it when you are ready.`);
      } catch {
        // A selectable dialog also works when clipboard permission is denied.
        showManualCopy(text, label);
      } finally {
        button.disabled = false;
      }
    });
  });

  document.getElementById("select-copy-text").addEventListener("click", selectManualText);
  dialog.addEventListener("close", () => {
    if (lastCopyButton) lastCopyButton.focus();
  });
})();
