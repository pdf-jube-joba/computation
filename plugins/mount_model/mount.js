import {mountModel as mountMountedModel} from "./script.js";

export default function createMountEnhancer(config = {}) {
  const mountBaseUrl = normalizeMountBaseUrl(config);

  return async function enhance(root) {
    const modelRoots = root.matches?.("[data-model]")
      ? [root]
      : Array.from(root.querySelectorAll("[data-model]"));

    await Promise.all(modelRoots.map(async modelRoot => {
      const model = (modelRoot.dataset.model || "").trim();
      if (!model) {
        modelRoot.textContent = "missing data-model";
        return;
      }

      try {
        await mountMountedModel(modelRoot, {
          model,
          code: extractDefaultValue(modelRoot, "default-code"),
          ainput: extractDefaultValue(modelRoot, "default-ainput"),
          rinput: extractDefaultValue(modelRoot, "default-rinput"),
          bundleBaseUrl: mountBaseUrl,
          assetBaseUrl: mountBaseUrl,
        });
      } catch (error) {
        modelRoot.textContent = `failed to load ${model}: ${error}`;
      }
    }));
  };
}

export const mountModel = createMountEnhancer;

function normalizeMountBaseUrl(config) {
  const value = config.wasm_mount_url || config.bundleBaseUrl;
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error("mount enhancer requires wasm_mount_url or bundleBaseUrl");
  }
  return value.endsWith("/") ? value : `${value}/`;
}

function extractDefaultValue(root, name) {
  const template = root.querySelector(`template[data-${name}]`);
  if (template) {
    return template.content.textContent || "";
  }

  const script = root.querySelector(`script.${name}[type="text/plain"]`);
  if (script) {
    return script.textContent || "";
  }

  return "";
}
