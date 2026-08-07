import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { GymMap, SimState } from "../model";
import type { VisualTheme } from "../visualThemes";
import { GameView } from "./GameView";
import "./AIViewer.css";

interface LocalCell {
  channel: string;
  row: number;
  col: number;
  x: number;
  y: number;
}

interface AttentionToken {
  family: number;
  weight: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

interface AIDecision {
  decision: number;
  frame: number;
  action: number[];
  goal: { x: number; y: number };
  proprio: Record<string, number>;
  local_cells: LocalCell[];
  attention: AttentionToken[];
}

interface AIDemo {
  version: number;
  generated_at?: string;
  episode_index?: number;
  live?: boolean;
  outcome?: string;
  room_key?: string;
  room_revision?: number;
  checkpoint: string;
  checkpoint_step: number;
  map: GymMap;
  states: SimState[];
  decisions: AIDecision[];
  frame_skip: number;
  local_grid_size: number;
  local_grid_cell_size?: number;
  local_channels: string[];
}

interface RoomSwitchResponse {
  room_key: string;
  room_name: string;
  room_revision: number;
}

const LIVE_API = "http://127.0.0.1:4318";

async function fetchLatestDemo(): Promise<AIDemo> {
  let latestError: unknown;
  for (const path of [
    `${LIVE_API}/api/ai-demo`,
    "/ai-demo-live.json",
    "/ai-demo.json",
  ]) {
    try {
      const separator = path.includes("?") ? "&" : "?";
      const response = await fetch(`${path}${separator}t=${Date.now()}`, { cache: "no-store" });
      if (!response.ok) throw new Error(`AI demo HTTP ${response.status}`);
      return (await response.json()) as AIDemo;
    } catch (reason) {
      latestError = reason;
    }
  }
  throw latestError ?? new Error("AI demo unavailable");
}

async function fetchRandomRoomDemo(): Promise<AIDemo> {
  const response = await fetch(`${LIVE_API}/api/ai-demo/random-room`, {
    method: "POST",
  });
  if (!response.ok) throw new Error(`随机换 Room HTTP ${response.status}`);
  const queued = (await response.json()) as RoomSwitchResponse;
  for (let attempt = 0; attempt < 120; attempt += 1) {
    await new Promise((resolve) => window.setTimeout(resolve, 500));
    const document = await fetchLatestDemo();
    if (document.room_revision === queued.room_revision) return document;
  }
  throw new Error(`等待 ${queued.room_name} 的推理轨迹超时`);
}

const CHANNEL_COLORS: Record<string, string> = {
  solid: "#5b7cfa",
  jump_thru: "#74e0c1",
  hazard: "#ff4567",
  dream_block: "#b36bff",
  water: "#3ed8ff",
  utility: "#ffe26b",
  moving_solid: "#ff9e45",
  holdable: "#80ff72",
  goal: "#ffffff",
  visited: "#6e7b92",
};

const FAMILY_NAMES = [
  "padding",
  "solid",
  "jump-through",
  "hazard",
  "dream",
  "water",
  "utility",
  "moving solid",
  "holdable",
  "controller",
  "unknown",
];

function actionLabel(action: number[]): string {
  const horizontal = ["左", "·", "右"][action[0]] ?? "·";
  const vertical = ["上", "·", "下"][action[1]] ?? "·";
  const jump = action[2] ? "跳" : "";
  const dash = ["", "冲", "蹲冲"][action[3]] ?? "";
  const grab = action[4] ? "抓" : "";
  return [horizontal, vertical, jump, dash, grab].filter(Boolean).join(" ");
}

export function AIViewer({ theme }: { theme: VisualTheme }) {
  const [demo, setDemo] = useState<AIDemo | null>(null);
  const [error, setError] = useState("");
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(true);
  const [loading, setLoading] = useState(true);
  const [switchingRoom, setSwitchingRoom] = useState(false);
  const [preloadingRoom, setPreloadingRoom] = useState(false);
  const [speed, setSpeed] = useState(1);
  const demoRef = useRef<AIDemo | null>(null);
  const frameRef = useRef(0);
  const switchingRoomRef = useRef(false);
  const preloadedDemoRef = useRef<AIDemo | null>(null);
  const preloadPromiseRef = useRef<Promise<AIDemo> | null>(null);

  const loadLatest = useCallback(async (autoplay: boolean) => {
    setLoading(true);
    setError("");
    try {
      const document = await fetchLatestDemo();
      if (!document.states.length || !document.decisions.length)
        throw new Error("最近一次 AI 轨迹为空");
      if (preloadedDemoRef.current?.room_revision === document.room_revision)
        preloadedDemoRef.current = null;
      demoRef.current = document;
      setDemo(document);
      setFrame(0);
      frameRef.current = 0;
      setPlaying(autoplay);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      setPlaying(false);
    } finally {
      setLoading(false);
    }
  }, []);

  const preloadNextRoom = useCallback((): Promise<AIDemo> => {
    if (preloadedDemoRef.current) return Promise.resolve(preloadedDemoRef.current);
    if (preloadPromiseRef.current) return preloadPromiseRef.current;

    setPreloadingRoom(true);
    const promise = fetchRandomRoomDemo()
      .then((document) => {
        if (!document.states.length || !document.decisions.length)
          throw new Error("预加载的 AI 轨迹为空");
        if (
          document.room_revision === undefined ||
          demoRef.current?.room_revision !== document.room_revision
        ) {
          preloadedDemoRef.current = document;
        }
        return document;
      })
      .finally(() => {
        if (preloadPromiseRef.current === promise) {
          preloadPromiseRef.current = null;
          setPreloadingRoom(false);
        }
      });
    preloadPromiseRef.current = promise;
    return promise;
  }, []);

  const switchToRandomRoom = useCallback(async () => {
    if (switchingRoomRef.current) return;
    switchingRoomRef.current = true;
    setSwitchingRoom(true);
    setError("");
    try {
      let document = preloadedDemoRef.current ?? (await preloadNextRoom());
      while (
        document.room_revision !== undefined &&
        demoRef.current?.room_revision === document.room_revision
      ) {
        if (preloadedDemoRef.current === document) preloadedDemoRef.current = null;
        document = await preloadNextRoom();
      }
      if (preloadedDemoRef.current === document) preloadedDemoRef.current = null;
      demoRef.current = document;
      setDemo(document);
      setFrame(0);
      frameRef.current = 0;
      setPlaying(true);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      switchingRoomRef.current = false;
      setSwitchingRoom(false);
    }
  }, [preloadNextRoom]);

  useEffect(() => {
    void loadLatest(true);
  }, [loadLatest]);

  useEffect(() => {
    if (!demo?.live || preloadingRoom || preloadedDemoRef.current) return;
    void preloadNextRoom().catch((reason) => {
      setError(reason instanceof Error ? reason.message : String(reason));
    });
  }, [demo?.live, demo?.room_revision, preloadingRoom, preloadNextRoom]);

  useEffect(() => {
    if (!demo?.live) return;
    const timeout = window.setTimeout(() => void switchToRandomRoom(), 4 * 60 * 1000);
    return () => window.clearTimeout(timeout);
  }, [demo?.live, demo?.room_revision, switchToRandomRoom]);

  useEffect(() => {
    if (!demo || !playing) return;
    let animation = 0;
    let previous = performance.now();
    let carry = 0;
    const tick = (now: number) => {
      const elapsed = Math.min(100, now - previous);
      previous = now;
      carry += (elapsed * 60 * speed) / 1000;
      const advance = Math.floor(carry);
      if (advance > 0) {
        carry -= advance;
        const next = frameRef.current + advance;
        if (next >= demo.states.length - 1) {
          frameRef.current = demo.states.length - 1;
          setFrame(frameRef.current);
          setPlaying(false);
          void switchToRandomRoom();
          return;
        }
        frameRef.current = next;
        setFrame(frameRef.current);
      }
      animation = requestAnimationFrame(tick);
    };
    animation = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(animation);
  }, [demo, playing, speed, switchToRandomRoom]);

  const decision = useMemo(() => {
    if (!demo) return undefined;
    for (let index = demo.decisions.length - 1; index >= 0; index -= 1) {
      if (demo.decisions[index].frame <= frame) return demo.decisions[index];
    }
    return demo.decisions[0];
  }, [demo, frame]);

  if (error && !demo)
    return (
      <main className="ai-viewer ai-empty">
        <h1>AI 回放未生成</h1>
        <p>{error}</p>
        <button disabled={loading} onClick={() => void loadLatest(true)}>
          {loading ? "正在获取…" : "重试"}
        </button>
        <code>uv run celeste-live-demo --run-dir ...</code>
      </main>
    );
  if (!demo || !decision)
    return (
      <main className="ai-viewer ai-empty">
        <h1>加载 AI 轨迹…</h1>
      </main>
    );

  const state = demo.states[Math.min(frame, demo.states.length - 1)];
  const path = demo.states.slice(0, frame + 1);
  const localCells = decision.local_cells.filter((cell) => cell.channel !== "visited");
  const localCellSize = demo.local_grid_cell_size ?? 8;
  const visibleAttention = decision.attention.slice(0, 12);
  const generatedAt = demo.generated_at ? new Date(demo.generated_at).toLocaleTimeString() : "静态";

  return (
    <main className="ai-viewer">
      <section className="ai-stage panel-frame">
        <GameView
          map={demo.map}
          state={state}
          states={demo.states}
          frame={frame}
          stale={false}
          theme={theme}
        >
          {(viewport) => {
            const scale = Math.min(
              viewport.width / viewport.camera.width,
              viewport.height / viewport.camera.height,
            );
            const offsetX = (viewport.width - viewport.camera.width * scale) / 2;
            const offsetY = (viewport.height - viewport.camera.height * scale) / 2;
            const sx = (x: number) => offsetX + (x - viewport.camera.x) * scale;
            const sy = (y: number) => offsetY + (y - viewport.camera.y) * scale;
            return (
              <svg className="ai-world-overlay" width={viewport.width} height={viewport.height}>
                <polyline
                  className="ai-path"
                  points={path.map((item) => `${sx(item.pos.x)},${sy(item.pos.y)}`).join(" ")}
                />
                {localCells.map((cell, index) => (
                  <rect
                    key={`${cell.channel}-${cell.x}-${cell.y}-${index}`}
                    x={sx(cell.x)}
                    y={sy(cell.y)}
                    width={localCellSize * scale}
                    height={localCellSize * scale}
                    fill={CHANNEL_COLORS[cell.channel] ?? "#ffffff"}
                    opacity={cell.channel === "solid" ? 0.14 : 0.26}
                  />
                ))}
                {visibleAttention.map((token, index) => (
                  <rect
                    key={`attention-${index}`}
                    className="ai-attention-token"
                    x={sx(token.x)}
                    y={sy(token.y)}
                    width={token.width * scale}
                    height={token.height * scale}
                    opacity={Math.min(1, 0.2 + token.weight * 10)}
                    strokeWidth={Math.max(1, token.weight * 28)}
                  />
                ))}
                <circle
                  className="ai-goal"
                  cx={sx(decision.goal.x)}
                  cy={sy(decision.goal.y)}
                  r={6 * scale}
                />
              </svg>
            );
          }}
        </GameView>
        <div className="ai-transport">
          <button
            disabled={loading}
            onClick={() => {
              if (playing) setPlaying(false);
              else void loadLatest(true);
            }}
          >
            {loading ? "获取最新…" : playing ? "暂停" : "获取最新并播放"}
          </button>
          <button
            onClick={() => {
              frameRef.current = 0;
              setFrame(0);
              setPlaying(true);
            }}
          >
            重播当前
          </button>
          <button
            title="无操作时每 4 分钟也会自动随机换 Room"
            disabled={switchingRoom}
            onClick={() => void switchToRandomRoom()}
          >
            {switchingRoom
              ? "切换 Room…"
              : preloadingRoom
                ? "下一 Room 预加载中…"
                : "随机换 Room"}
          </button>
          <select value={speed} onChange={(event) => setSpeed(Number(event.target.value))}>
            <option value={0.25}>0.25×</option>
            <option value={0.5}>0.5×</option>
            <option value={1}>1×</option>
            <option value={2}>2×</option>
          </select>
          <input
            type="range"
            min={0}
            max={demo.states.length - 1}
            value={frame}
            onChange={(event) => {
              const next = Number(event.target.value);
              frameRef.current = next;
              setFrame(next);
            }}
          />
          <strong>
            F{frame} / {demo.states.length - 1}
          </strong>
        </div>
      </section>

      <aside className="ai-perception panel-frame">
        <header>
          <small>
            {demo.live ? "LIVE · ROOM FIXED · AUTO 4MIN" : "REPLAY"} · CHECKPOINT {demo.checkpoint_step.toLocaleString()} · {generatedAt}
          </small>
          <h1>AI 地图感知</h1>
          <p>{demo.map.name}</p>
          {error && <p className="ai-refresh-error">刷新失败，继续显示上一条：{error}</p>}
        </header>
        <section className="ai-action-card">
          <small>
            DECISION {decision.decision} · {demo.outcome ?? "recorded"}
          </small>
          <strong>{actionLabel(decision.action)}</strong>
          <span>
            pos {state.pos.x.toFixed(1)}, {state.pos.y.toFixed(1)} · speed {state.speed.x.toFixed(1)}, {state.speed.y.toFixed(1)}
          </span>
        </section>
        <section>
          <h2>
            局部 {demo.local_grid_size}×{demo.local_grid_size} · {localCellSize}px 精细视野
          </h2>
          <div
            className="ai-local-grid"
            style={{
              gridTemplateColumns: `repeat(${demo.local_grid_size}, 1fr)`,
              gridTemplateRows: `repeat(${demo.local_grid_size}, 1fr)`,
            }}
          >
            {decision.local_cells.map((cell, index) => (
              <i
                key={`${cell.channel}-${cell.row}-${cell.col}-${index}`}
                style={{
                  gridRow: cell.row + 1,
                  gridColumn: cell.col + 1,
                  background: CHANNEL_COLORS[cell.channel] ?? "#fff",
                }}
                title={cell.channel}
              />
            ))}
          </div>
          <div className="ai-legend">
            {demo.local_channels.slice(0, 9).map((channel) => (
              <span key={channel}>
                <i style={{ background: CHANNEL_COLORS[channel] }} />
                {channel}
              </span>
            ))}
          </div>
        </section>
        <section>
          <h2>全局实体注意力</h2>
          <ol className="ai-attention-list">
            {decision.attention.slice(0, 8).map((token, index) => (
              <li key={`${token.family}-${index}`}>
                <span>{FAMILY_NAMES[token.family] ?? `family ${token.family}`}</span>
                <meter min={0} max={Math.max(0.01, decision.attention[0]?.weight ?? 1)} value={token.weight} />
                <strong>{(token.weight * 100).toFixed(1)}%</strong>
              </li>
            ))}
          </ol>
        </section>
        <section className="ai-proprio">
          <h2>玩家向量</h2>
          {Object.entries(decision.proprio).map(([name, value]) => (
            <span key={name}>
              <small>{name}</small>
              <strong>{value.toFixed(3)}</strong>
            </span>
          ))}
        </section>
      </aside>
    </main>
  );
}
