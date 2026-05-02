class TestResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

globalThis.ResizeObserver ??= TestResizeObserver;

Element.prototype.scrollIntoView ??= function scrollIntoView() {};
