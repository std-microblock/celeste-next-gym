import { useEffect, useRef, useState, type CSSProperties } from "react";
import type { GameViewViewport } from "../camera";
import type { GymMap, SimState, Vec2 } from "../model";

const MAX_SPEED = 900;
const ACCELERATION = 5_400;
const TYPE_INTERVAL_MS = 28;
const PROMPT_ANCHOR_ABOVE_PLAYER = 15;

function approach(current: number, target: number, amount: number): number {
  return current < target
    ? Math.min(target, current + amount)
    : Math.max(target, current - amount);
}

export function mapPointTargetPercent(
  map: GymMap,
  point: Vec2,
  viewport: Pick<GameViewViewport, "width" | "height"> &
    Partial<Pick<GameViewViewport, "camera">>,
): Vec2 {
  if (viewport.width <= 0 || viewport.height <= 0) return { x: 50, y: 50 };
  const world = viewport.camera ?? map.bounds;
  const scale = Math.min(
    viewport.width / world.width,
    viewport.height / world.height,
  );
  const contentWidth = world.width * scale;
  const contentHeight = world.height * scale;
  const offsetX = (viewport.width - contentWidth) / 2;
  const offsetY = (viewport.height - contentHeight) / 2;
  return {
    x:
      ((offsetX + (point.x - world.x) * scale) / viewport.width) * 100,
    y:
      ((offsetY +
        (point.y - world.y) * scale) /
        viewport.height) *
      100,
  };
}

export function promptTargetPercent(
  map: GymMap,
  state: SimState,
  viewport: Pick<GameViewViewport, "width" | "height"> &
    Partial<Pick<GameViewViewport, "camera">>,
): Vec2 {
  return mapPointTargetPercent(
    map,
    { x: state.pos.x, y: state.pos.y - PROMPT_ANCHOR_ABOVE_PLAYER },
    viewport,
  );
}

export function TrainingPrompt({
  map,
  state,
  viewport,
  text,
  hidden = false,
}: {
  map: GymMap;
  state: SimState;
  viewport: Pick<GameViewViewport, "width" | "height"> &
    Partial<Pick<GameViewViewport, "camera">>;
  text: string;
  hidden?: boolean;
}) {
  const target = promptTargetPercent(map, state, viewport);
  const position = useRef<Vec2>(target);
  const velocity = useRef<Vec2>({ x: 0, y: 0 });
  const targetRef = useRef(target);
  const [renderedPosition, setRenderedPosition] = useState(target);
  const [visibleLength, setVisibleLength] = useState(0);

  targetRef.current = target;

  useEffect(() => {
    setVisibleLength(0);
    if (hidden) return;
    const timer = window.setInterval(() => {
      setVisibleLength((length) => {
        if (length >= text.length) {
          window.clearInterval(timer);
          return length;
        }
        return length + 1;
      });
    }, TYPE_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [hidden, text]);

  useEffect(() => {
    let animation = 0;
    let last = performance.now();
    const tick = (now: number) => {
      const delta = Math.min(0.05, (now - last) / 1000);
      last = now;
      const desiredX = targetRef.current.x - position.current.x;
      const desiredY = targetRef.current.y - position.current.y;
      const distance = Math.hypot(desiredX, desiredY);
      const desiredSpeed = Math.min(MAX_SPEED, distance * 14);
      const desiredVelocity =
        distance <= 0.01
          ? { x: 0, y: 0 }
          : {
              x: (desiredX / distance) * desiredSpeed,
              y: (desiredY / distance) * desiredSpeed,
            };
      velocity.current = {
        x: approach(
          velocity.current.x,
          desiredVelocity.x,
          ACCELERATION * delta,
        ),
        y: approach(
          velocity.current.y,
          desiredVelocity.y,
          ACCELERATION * delta,
        ),
      };
      position.current = {
        x: position.current.x + velocity.current.x * delta,
        y: position.current.y + velocity.current.y * delta,
      };
      setRenderedPosition(position.current);
      animation = requestAnimationFrame(tick);
    };
    animation = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(animation);
  }, []);

  if (hidden) return null;
  return (
    <div
      className="training-prompt"
      style={
        {
          "--prompt-x": `${renderedPosition.x}%`,
          "--prompt-y": `${renderedPosition.y}%`,
        } as CSSProperties
      }
    >
      {text.slice(0, visibleLength)}
      <i aria-hidden="true" />
    </div>
  );
}
