import "@testing-library/jest-dom";

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
