// React-tree test setup (appended after test/setup.ts for the react project).
// jsdom lacks browser observers used by desktop component frameworks.

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
if (typeof globalThis.ResizeObserver === "undefined") {
  (globalThis as Record<string, unknown>).ResizeObserver = ResizeObserverStub;
}
