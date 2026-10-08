/**
 * Pluggable 2D canvas backend for the game renderer.
 *
 * The browser build uses the DOM (`document.createElement("canvas")`,
 * `Image`, `fetch`). Headless tooling (the agent CLI in `tools/gym-cli`)
 * installs an `@napi-rs/canvas` backend via `setRenderBackend`, which exposes
 * the same CanvasRenderingContext2D surface, so every draw routine in
 * `gameRenderer.ts` runs unchanged in both environments.
 *
 * Types intentionally stay as the DOM canvas types: the napi-rs canvas is
 * structurally compatible for everything the renderer touches.
 */
export interface RenderBackend {
  /** Create an offscreen canvas of the given size. */
  createCanvas(width: number, height: number): HTMLCanvasElement;
  /** Load an image from a root-relative asset URL such as `/assets/x.png`. */
  loadImage(url: string): Promise<CanvasImageSource & { width: number; height: number }>;
  /** Load and parse a JSON document from a root-relative asset URL. */
  loadJson(url: string): Promise<unknown>;
  /**
   * Optional: convert a finished, never-again-modified offscreen canvas into
   * the backend's fastest drawImage source; the browser keeps the canvas.
   *
   * Only implement this when the returned object is drawable *immediately and
   * synchronously* — the very next `drawImage` runs in the same tick. A source
   * that needs an asynchronous decode must not be returned here: e.g.
   * `@napi-rs/canvas` decodes `Image.src = <Buffer>` on a worker thread, so an
   * Image built from `canvas.toBuffer("image/png")` draws an empty bitmap and
   * silently blanks the whole frame. Such a backend should leave this
   * undefined and let `freezeRenderCanvas` return the canvas.
   */
  freeze?(canvas: HTMLCanvasElement): HTMLCanvasElement;
}

export const browserRenderBackend: RenderBackend = {
  createCanvas(width, height) {
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    return canvas;
  },
  loadImage(url) {
    return new Promise<HTMLImageElement>((resolve, reject) => {
      const image = new Image();
      image.src = url;
      image.onload = () => resolve(image);
      image.onerror = () => reject(new Error(`image failed to load: ${url}`));
    });
  },
  async loadJson(url) {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`asset failed to load: ${url} (${response.status})`);
    return response.json();
  },
};

let activeBackend: RenderBackend = browserRenderBackend;

export function setRenderBackend(backend: RenderBackend): void {
  activeBackend = backend;
}

export function renderBackend(): RenderBackend {
  return activeBackend;
}

export function createRenderCanvas(width = 1, height = 1): HTMLCanvasElement {
  return activeBackend.createCanvas(Math.max(1, width), Math.max(1, height));
}

/** Mark a cached offscreen canvas as final (see `RenderBackend.freeze`). */
export function freezeRenderCanvas(canvas: HTMLCanvasElement): HTMLCanvasElement {
  return activeBackend.freeze ? activeBackend.freeze(canvas) : canvas;
}
