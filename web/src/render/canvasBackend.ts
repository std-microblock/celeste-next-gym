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
   * the backend's fastest drawImage source. Only implement this when the
   * result is drawable *immediately and synchronously* — it is used by the very
   * next `drawImage`, in the same tick.
   */
  freeze?(canvas: HTMLCanvasElement): HTMLCanvasElement;
  /**
   * Optional async variant, for backends whose fastest source needs a decode.
   *
   * `@napi-rs/canvas` is such a backend: drawing a *canvas* makes Skia copy the
   * whole source surface on every call (an 8x8 tile blitted out of the
   * 1024x4600 gameplay atlas copies ~18 MB per tile, which OOMs a small
   * machine), while a decoded `Image` is free — but `Image.src =
   * canvas.toBuffer("image/png")` only finishes on a worker thread, and
   * drawing the Image before that draws an empty bitmap. So the decode has to
   * be awaited by a caller that can: `freezeRenderCanvasAsync`, used to
   * prepare the cached atlases / tile layers before the first frame.
   *
   * Implementations should pass non-canvas sources straight through.
   */
  freezeAsync?(canvas: HTMLCanvasElement): Promise<HTMLCanvasElement>;
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

/**
 * Mark a cached offscreen canvas as final (see `RenderBackend.freeze`).
 * Synchronous: only valid for backends whose frozen source is drawable in the
 * same tick. Callers that can await should use `freezeRenderCanvasAsync`.
 */
export function freezeRenderCanvas(canvas: HTMLCanvasElement): HTMLCanvasElement {
  return activeBackend.freeze ? activeBackend.freeze(canvas) : canvas;
}

/**
 * Awaitably mark a cached offscreen canvas as final (see
 * `RenderBackend.freezeAsync`), for backends that must decode first. On
 * backends without an async freeze this is a resolved pass-through.
 */
export async function freezeRenderCanvasAsync(canvas: HTMLCanvasElement): Promise<HTMLCanvasElement> {
  const backend = activeBackend;
  if (backend.freezeAsync) return await backend.freezeAsync(canvas);
  return freezeRenderCanvas(canvas);
}
