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

class IntersectionObserverStub {
  root = null;
  rootMargin = "0px";
  thresholds = [0];
  observe() {}
  unobserve() {}
  disconnect() {}
  takeRecords() { return []; }
}
if (typeof globalThis.IntersectionObserver === "undefined") {
  (globalThis as Record<string, unknown>).IntersectionObserver = IntersectionObserverStub;
}

// rc-util probes pseudo-element scrollbar styles, which jsdom does not model.
const jsdomGetComputedStyle = window.getComputedStyle.bind(window);
window.getComputedStyle = ((element: Element) => jsdomGetComputedStyle(element)) as typeof window.getComputedStyle;

// Importing Ant Design X initializes its optional Web Notification adapter.
class NotificationStub {
  static permission: NotificationPermission = "denied";
  static requestPermission = async () => "denied" as NotificationPermission;
  onclick = null;
  onshow = null;
  onclose = null;
  onerror = null;
  constructor(_title: string, _options?: NotificationOptions) {}
  close() {}
}
if (typeof globalThis.Notification === "undefined") {
  (globalThis as Record<string, unknown>).Notification = NotificationStub;
}
