(() => {
  const bundleBaseUrl = "/wasm_bundle/";
  const assetBaseUrl = "/assets/";
  let mountModelPromise = null;

  function loadMountModel() {
    if (!mountModelPromise) {
      mountModelPromise = import(`${assetBaseUrl}script.js`).then(mod => {
        if (typeof mod.mountModel !== "function") {
          throw new Error("assets/script.js does not export mountModel");
        }
        return mod.mountModel;
      });
    }
    return mountModelPromise;
  }

  function collectModelRoots(root) {
    if (!(root instanceof Element || root instanceof Document)) {
      return [];
    }
    const nodes = [];
    if (root instanceof Element && root.matches("[data-model]")) {
      nodes.push(root);
    }
    nodes.push(...root.querySelectorAll("[data-model]"));
    return nodes;
  }

  function extractDefaultValue(root, name) {
    const template = root.querySelector(`template[data-${name}]`);
    if (template) {
      return template.content.textContent || "";
    }
    return "";
  }

  async function mountModels(root) {
    const modelRoots = collectModelRoots(root).filter(node => node.dataset.wasmMounted !== "1");
    if (modelRoots.length === 0) {
      return;
    }

    const mountModel = await loadMountModel();
    await Promise.all(modelRoots.map(async modelRoot => {
      const model = (modelRoot.dataset.model || "").trim();
      if (!model) {
        modelRoot.textContent = "missing data-model";
        return;
      }

      modelRoot.dataset.wasmMounted = "1";
      try {
        await mountModel(modelRoot, {
          model,
          code: extractDefaultValue(modelRoot, "default-code"),
          ainput: extractDefaultValue(modelRoot, "default-ainput"),
          rinput: extractDefaultValue(modelRoot, "default-rinput"),
          bundleBaseUrl,
          assetBaseUrl,
        });
      } catch (error) {
        delete modelRoot.dataset.wasmMounted;
        modelRoot.textContent = `failed to load ${model}: ${error}`;
      }
    }));
  }

  document.addEventListener("md-preview:render", event => {
    void mountModels(event.target);
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
      void mountModels(document);
    }, {once: true});
  } else {
    void mountModels(document);
  }
})();
