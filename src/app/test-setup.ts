// React-tree test setup (appended after test/setup.ts for the react project).
// jsdom lacks the browser APIs Mantine components touch at mount.

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
if (typeof globalThis.ResizeObserver === "undefined") {
  (globalThis as Record<string, unknown>).ResizeObserver = ResizeObserverStub;
}
