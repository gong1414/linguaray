import "@testing-library/jest-dom";

// Node 22 ships an experimental bare-global localStorage whose getter returns
// undefined (no --localstorage-file). jsdom has a working one on `window`.
// Override the global with a simple in-memory shim so bare `localStorage`
// (theme.test.ts, InputPanel.test.tsx) resolves without depending on Node's
// experimental impl or jsdom's origin-dependent one.
if (typeof globalThis.localStorage === "undefined" || !globalThis.localStorage) {
  const store = new Map<string, string>();
  const shim: Storage = {
    get length() { return store.size; },
    clear: () => store.clear(),
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => { store.set(k, String(v)); },
    removeItem: (k: string) => { store.delete(k); },
    key: (i: number) => Array.from(store.keys())[i] ?? null,
  };
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    writable: true,
    value: shim,
  });
}

// jsdom lacks matchMedia; Solid components + reduced-motion checks may read it.
if (!window.matchMedia) {
  // @ts-expect-error partial mock
  window.matchMedia = () => ({
    matches: false,
    addEventListener() {},
    removeEventListener() {},
    onchange: null,
    dispatchEvent: () => false,
    addListener() {},
    removeListener() {},
  });
}
