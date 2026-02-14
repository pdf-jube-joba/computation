(() => {
  const renderMath = window.renderMathInElement;
  if (typeof renderMath !== "function") {
    console.warn("KaTeX auto-render is not available.");
    return;
  }

  const parseMacros = text => {
    const macros = {};
    for (const rawLine of text.split(/\r?\n/)) {
      const line = rawLine.trim();
      if (!line || line.startsWith("#")) continue;
      const colonIndex = line.indexOf(":");
      if (colonIndex < 0) continue;
      const name = line.slice(0, colonIndex).trim();
      const body = line.slice(colonIndex + 1).trim();
      if (!name || !body) continue;
      macros[name] = body;
    }
    return macros;
  };

  const fetchMacros = async () => {
    const current = document.currentScript;
    if (!current || !current.src) return {};
    const macrosUrl = new URL("../macros.txt", current.src).toString();
    try {
      const response = await fetch(macrosUrl, { cache: "no-store" });
      if (!response.ok) {
        console.warn("KaTeX macros.txt fetch failed:", response.status);
        return {};
      }
      return parseMacros(await response.text());
    } catch (err) {
      console.warn("KaTeX macros.txt fetch error:", err);
      return {};
    }
  };

  const render = async () => {
    const macros = await fetchMacros();
    renderMath(document.body, {
      delimiters: [
        { left: "\\(", right: "\\)", display: false },
        { left: "\\[", right: "\\]", display: true },
      ],
      macros,
      throwOnError: false,
      ignoredTags: ["script", "noscript", "style", "textarea", "pre", "code"],
    });
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", render, { once: true });
  } else {
    render();
  }
})();
