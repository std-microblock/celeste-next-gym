import { useEffect, useRef } from "react";

interface AtlasEntry {
  x: number;
  y: number;
  width: number;
  height: number;
  drawOffsetX: number;
  drawOffsetY: number;
  frameWidth: number;
  frameHeight: number;
}

interface GameplayAtlas {
  image: HTMLImageElement;
  entries: Record<string, AtlasEntry>;
}

let gameplayAtlasPromise: Promise<GameplayAtlas> | null = null;

function loadGameplayAtlas(): Promise<GameplayAtlas> {
  if (!gameplayAtlasPromise) {
    gameplayAtlasPromise = Promise.all([
      fetch("/assets/original/gameplay/gameplay-selected.json").then(
        (response) => response.json(),
      ) as Promise<{ entries: Record<string, AtlasEntry> }>,
      new Promise<HTMLImageElement>((resolve, reject) => {
        const image = new Image();
        image.src = "/assets/original/gameplay/gameplay-selected.png";
        image.onload = () => resolve(image);
        image.onerror = () => reject(new Error("原版 Gameplay 图集加载失败"));
      }),
    ]).then(([manifest, image]) => ({ image, entries: manifest.entries }));
  }
  return gameplayAtlasPromise;
}

/** Draws the original Celeste strawberry frames directly from the Gameplay atlas. */
export function GameplayStrawberry({ scale = 5 }: { scale?: number }) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    let active = true;
    let animation = 0;
    let atlas: GameplayAtlas | null = null;
    const startedAt = performance.now();
    void loadGameplayAtlas().then((loaded) => {
      atlas = loaded;
      const first = loaded.entries["collectables/strawberry/normal00"];
      if (canvas.current && first) {
        canvas.current.width = first.frameWidth * scale;
        canvas.current.height = first.frameHeight * scale;
      }
      const draw = (now: number) => {
        if (!active || !atlas || !canvas.current) return;
        const frame = Math.floor((now - startedAt) / 100) % 7;
        const entry =
          atlas.entries[
            `collectables/strawberry/normal${String(frame).padStart(2, "0")}`
          ];
        const context = canvas.current.getContext("2d");
        if (entry && context) {
          context.imageSmoothingEnabled = false;
          context.clearRect(0, 0, canvas.current.width, canvas.current.height);
          context.drawImage(
            atlas.image,
            entry.x,
            entry.y,
            entry.width,
            entry.height,
            entry.drawOffsetX * scale,
            entry.drawOffsetY * scale,
            entry.width * scale,
            entry.height * scale,
          );
        }
        animation = requestAnimationFrame(draw);
      };
      animation = requestAnimationFrame(draw);
    });
    return () => {
      active = false;
      cancelAnimationFrame(animation);
    };
  }, [scale]);

  return (
    <canvas
      ref={canvas}
      className="training-strawberry-sprite"
      role="img"
      aria-label="草莓"
    />
  );
}
