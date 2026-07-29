import { useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import {
  formatObjectiveOutput,
  objectiveOutputName,
  type TrainingObjectiveSeries as TrainingObjectiveSeriesData,
} from "../training/course";
import type { FrameWindow } from "../training/session";

export type TrainingObjectiveSeries = TrainingObjectiveSeriesData;

function contiguousSegments(
  points: Array<{ frame: number; value: number }>,
): Array<Array<{ frame: number; value: number }>> {
  const segments: Array<Array<{ frame: number; value: number }>> = [];
  for (const point of points) {
    const segment = segments.at(-1);
    if (!segment || point.frame > segment.at(-1)!.frame + 1)
      segments.push([point]);
    else segment.push(point);
  }
  return segments;
}

function steppedPolylinePoints(
  points: Array<{ frame: number; value: number }>,
  x: (frame: number) => number,
  y: (value: number) => number,
): string {
  const first = points[0];
  if (!first) return "";
  const coordinates = [`${x(first.frame)},${y(first.value)}`];
  for (let index = 1; index < points.length; index += 1) {
    const previous = points[index - 1];
    const current = points[index];
    const midpoint = (x(previous.frame) + x(current.frame)) / 2;
    coordinates.push(
      `${midpoint},${y(previous.value)}`,
      `${midpoint},${y(current.value)}`,
      `${x(current.frame)},${y(current.value)}`,
    );
  }
  return coordinates.join(" ");
}

function ObjectiveCurve({
  series,
  from,
  to,
}: {
  series: TrainingObjectiveSeries[];
  from: number;
  to: number;
}) {
  const span = Math.max(1, to - from);
  return (
    <svg
      className="training-objective-curve"
      viewBox="0 0 100 50"
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      {series.map((objective, objectiveIndex) => {
        const visible = objective.points.filter(
          (point) => point.frame >= from && point.frame <= to,
        );
        if (visible.length === 0) return null;
        const values = visible.map((point) => point.value);
        const minimum = Math.min(...values);
        const maximum = Math.max(...values);
        const range = maximum - minimum;
        const x = (frame: number) => ((frame - from) / span) * 100;
        const y = (value: number) =>
          range === 0 ? 25 : 44 - ((value - minimum) / range) * 36;
        return (
          <g
            key={`${objective.expression}-${objectiveIndex}`}
            className={`training-objective-series objective-${objectiveIndex % 3}`}
          >
            {contiguousSegments(visible).map((segment, segmentIndex) => (
              <polyline
                key={segmentIndex}
                points={steppedPolylinePoints(segment, x, y)}
              />
            ))}
          </g>
        );
      })}
    </svg>
  );
}

function ObjectiveHoverLayer({
  series,
  from,
  to,
  targetFrame,
  windows,
  actualInputs,
  failureFrame,
}: {
  series: TrainingObjectiveSeries[];
  from: number;
  to: number;
  targetFrame?: number;
  windows: FrameWindow[];
  actualInputs: readonly { frame: number }[];
  failureFrame?: number;
}) {
  const [tooltip, setTooltip] = useState<{
    frame: number;
    x: number;
    y: number;
  } | null>(null);
  const frames =
    series[0]?.points.filter(
      (point) => point.frame >= from && point.frame <= to,
    ) ?? [];
  if (frames.length === 0) return null;
  const span = Math.max(1, to - from);
  const frameWidth = 100 / span;
  const tooltipFrame =
    tooltip === null
      ? undefined
      : frames.find((frame) => frame.frame === tooltip.frame);
  const tooltipPointTypes = tooltipFrame
    ? pointTypesAt(
        tooltipFrame.frame,
        tooltipFrame.successful,
        targetFrame,
        windows,
        actualInputs,
        failureFrame,
      )
    : [];
  const showTooltip = (frame: number, x: number, y: number) =>
    setTooltip({ frame, x, y });
  return (
    <>
      <div
        className="training-objective-hover-layer"
        aria-label="Fuzz objective 按操作帧输出"
      >
        {frames.map((frame) => {
          const center = ((frame.frame - from) / span) * 100;
          const left = Math.max(0, center - frameWidth / 2);
          const right = Math.min(100, center + frameWidth / 2);
          const pointTypes = pointTypesAt(
            frame.frame,
            frame.successful,
            targetFrame,
            windows,
            actualInputs,
            failureFrame,
          );
          const details = series.flatMap((objective) => {
            const point = objective.points.find(
              (candidate) => candidate.frame === frame.frame,
            );
            return point === undefined
              ? []
              : [
                  `${objectiveOutputName(objective.expression)} ${formatObjectiveOutput(objective.expression, point.value)}`,
                ];
          });
          return (
            <i
              key={frame.frame}
              className={`training-objective-hit ${frame.successful ? "successful" : "failed"}`}
              style={{ left: `${left}%`, width: `${right - left}%` }}
              tabIndex={0}
              aria-label={`在 F${frame.frame} 操作：${pointTypes.join("、")}；${details.join("；")}`}
              onMouseEnter={(event) =>
                showTooltip(frame.frame, event.clientX, event.clientY)
              }
              onMouseMove={(event) =>
                showTooltip(frame.frame, event.clientX, event.clientY)
              }
              onMouseLeave={() => setTooltip(null)}
              onFocus={(event) => {
                const rect = event.currentTarget.getBoundingClientRect();
                showTooltip(frame.frame, rect.left + rect.width / 2, rect.top);
              }}
              onBlur={() => setTooltip(null)}
            />
          );
        })}
      </div>
      {tooltip && tooltipFrame && typeof document !== "undefined"
        ? createPortal(
            <span
              className={`training-objective-tooltip training-objective-tooltip-portal ${tooltipFrame.successful ? "successful" : "failed"}`}
              role="tooltip"
              style={tooltipPosition(tooltip.x, tooltip.y)}
            >
              <b>在 F{tooltipFrame.frame} 操作</b>
              <em>{tooltipPointTypes.join(" · ")}</em>
              {series.map((objective, index) => {
                const point = objective.points.find(
                  (candidate) => candidate.frame === tooltipFrame.frame,
                );
                return point === undefined ? null : (
                  <strong key={`${objective.expression}-${index}`}>
                    <span>{objectiveOutputName(objective.expression)}</span>
                    {formatObjectiveOutput(objective.expression, point.value)}
                  </strong>
                );
              })}
            </span>,
            document.body,
          )
        : null}
    </>
  );
}

function pointTypesAt(
  frame: number,
  successful: boolean,
  targetFrame: number | undefined,
  windows: FrameWindow[],
  actualInputs: readonly { frame: number }[],
  failureFrame: number | undefined,
): string[] {
  const pointTypes = [
    ...(targetFrame === frame ? ["Fuzz 最佳点"] : []),
    ...(actualInputs.some((input) => input.frame === frame) ? ["你的输入"] : []),
    ...(windows.some((window) => frame >= window.from && frame <= window.to)
      ? ["成功窗口"]
      : []),
    ...(failureFrame === frame ? ["失败点"] : []),
  ];
  if (pointTypes.length === 0)
    pointTypes.push(successful ? "可行候选" : "未通过候选");
  return pointTypes;
}

function tooltipPosition(x: number, y: number) {
  const gap = 12;
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;
  return {
    ...(x < viewportWidth / 2
      ? { left: Math.max(8, x + gap) }
      : { right: Math.max(8, viewportWidth - x + gap) }),
    ...(y < viewportHeight / 2
      ? { top: Math.max(8, y + gap) }
      : { bottom: Math.max(8, viewportHeight - y + gap) }),
  };
}

export interface TrainingTimelineProps {
  frame: number;
  frameCount: number;
  fuzzStart: number | null;
  targetFrame?: number;
  windows: FrameWindow[];
  actualInputs: readonly { frame: number; keys: readonly string[] }[];
  failureFrame?: number;
  resetFrame: number;
  objectives: TrainingObjectiveSeries[];
  followTarget?: boolean;
  onSeek(frame: number, manual?: boolean): void;
  onSetReset(frame: number): void;
}

/** A review-only timeline for lessons.  It intentionally has no TAS editing affordances. */
export function TrainingTimeline({
  frame,
  frameCount,
  fuzzStart,
  targetFrame,
  windows,
  actualInputs,
  failureFrame,
  resetFrame,
  objectives,
  followTarget = false,
  onSeek,
  onSetReset,
}: TrainingTimelineProps) {
  const track = useRef<HTMLDivElement>(null);
  const pointerStart = useRef<number | null>(null);
  const dragViewportStartRef = useRef<number | null>(null);
  const [dragViewportStart, setDragViewportStart] = useState<number | null>(
    null,
  );
  const maximum = Math.max(1, frameCount);
  const viewportFrames = Math.min(maximum, 48);
  const viewportFocus =
    followTarget && targetFrame !== undefined ? targetFrame : frame;
  const automaticViewportStart = Math.max(
    0,
    Math.min(
      maximum - viewportFrames,
      viewportFocus - Math.floor(viewportFrames / 2),
    ),
  );
  const viewportStart = dragViewportStart ?? automaticViewportStart;
  const viewportEnd = viewportStart + viewportFrames;
  const inViewport = (value: number) =>
    value >= viewportStart && value <= viewportEnd;
  const percent = (value: number) =>
    `${((value - viewportStart) / Math.max(1, viewportFrames)) * 100}%`;
  const clampedPercent = (value: number) =>
    `${Math.max(0, Math.min(100, ((value - viewportStart) / Math.max(1, viewportFrames)) * 100))}%`;
  const visibleWindows = windows.flatMap((window) => {
    const from = Math.max(window.from, viewportStart);
    const to = Math.min(window.to, viewportEnd);
    return from <= to ? [{ ...window, from, to }] : [];
  });
  const inputLabels = useMemo(
    () =>
      actualInputs.map((input) => ({
        ...input,
        label: input.keys.join("+").toUpperCase(),
      })),
    [actualInputs],
  );
  const selectFrame = (
    clientX: number,
    allowEdgeScroll = false,
  ): number | undefined => {
    const rect = track.current?.getBoundingClientRect();
    if (!rect) return undefined;
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    let start = dragViewportStartRef.current ?? viewportStart;
    if (allowEdgeScroll && ratio <= 0.04 && start > 0) start -= 1;
    else if (
      allowEdgeScroll &&
      ratio >= 0.96 &&
      start < maximum - viewportFrames
    )
      start += 1;
    if (start !== dragViewportStartRef.current) {
      dragViewportStartRef.current = start;
      setDragViewportStart(start);
    }
    const next = Math.round(start + ratio * viewportFrames);
    onSeek(next, true);
    return next;
  };
  return (
    <section className="training-timeline panel-frame" aria-label="训练时间线">
      <div className="training-timeline-head">
        <div>
          <small>
            TIMELINE · F{viewportStart}–F{viewportStart + viewportFrames}
          </small>
          <h2>时间线</h2>
        </div>
        <output>F{frame}</output>
      </div>
      <div
        ref={track}
        className="training-track"
        role="slider"
        aria-label="训练回看时间线"
        aria-valuemin={0}
        aria-valuemax={maximum}
        aria-valuenow={frame}
        tabIndex={0}
        onPointerDown={(event) => {
          pointerStart.current = event.clientX;
          dragViewportStartRef.current = viewportStart;
          setDragViewportStart(viewportStart);
          event.currentTarget.setPointerCapture?.(event.pointerId);
          selectFrame(event.clientX);
        }}
        onPointerMove={(event) => {
          if (
            !event.currentTarget.hasPointerCapture ||
            event.currentTarget.hasPointerCapture(event.pointerId)
          )
            selectFrame(event.clientX, true);
        }}
        onPointerUp={(event) => {
          const selected = selectFrame(event.clientX);
          if (
            pointerStart.current !== null &&
            Math.abs(pointerStart.current - event.clientX) < 4 &&
            selected !== undefined
          )
            onSetReset(selected);
          pointerStart.current = null;
          dragViewportStartRef.current = null;
          setDragViewportStart(null);
          if (event.currentTarget.hasPointerCapture?.(event.pointerId))
            event.currentTarget.releasePointerCapture?.(event.pointerId);
        }}
        onKeyDown={(event) => {
          if (event.code === "ArrowLeft" || event.code === "ArrowRight") {
            event.preventDefault();
            onSeek(frame + (event.code === "ArrowLeft" ? -1 : 1), true);
          }
        }}
      >
        <ObjectiveCurve
          series={objectives}
          from={viewportStart}
          to={viewportEnd}
        />
        <ObjectiveHoverLayer
          series={objectives}
          from={viewportStart}
          to={viewportEnd}
          targetFrame={targetFrame}
          windows={windows}
          actualInputs={actualInputs}
          failureFrame={failureFrame}
        />
        {visibleWindows.map((window, index) => (
          <i
            key={`${window.from}-${window.to}-${index}`}
            className="training-window"
            style={{
              left: percent(window.from),
              width: `${Math.max(0.8, ((window.to - window.from + 1) / viewportFrames) * 100)}%`,
            }}
            title={`成功窗口 F${window.from}–F${window.to}`}
          />
        ))}
        {fuzzStart !== null && inViewport(fuzzStart) && (
          <b
            className="training-marker fuzz"
            style={{ left: percent(fuzzStart) }}
          >
            F0
            <span className="training-tooltip">
              操作起点：训练定义的入口输入是本地 F0
            </span>
          </b>
        )}
        {targetFrame !== undefined && (
          <b
            className={`training-marker target ${inViewport(targetFrame) ? "" : targetFrame < viewportStart ? "offscreen before" : "offscreen after"}`}
            style={{ left: clampedPercent(targetFrame) }}
          >
            {inViewport(targetFrame)
              ? "◆"
              : targetFrame < viewportStart
                ? "‹"
                : "›"}
            <span className="training-tooltip">
              下一最佳关键点：F{targetFrame}
              {objectives[0]?.points.find(
                (point) => point.frame === targetFrame,
              ) === undefined
                ? ""
                : `；${objectives[0].expression} = ${objectives[0].points.find((point) => point.frame === targetFrame)!.value.toFixed(2)}`}
            </span>
          </b>
        )}
        {inputLabels
          .filter((input) => inViewport(input.frame))
          .map((input) => (
            <b
              key={`${input.frame}-${input.label}`}
              className="training-marker input"
              style={{ left: percent(input.frame) }}
            >
              ●
              <span className="training-tooltip">
                你的输入：F{input.frame} {input.label}
              </span>
            </b>
          ))}
        {failureFrame !== undefined && inViewport(failureFrame) && (
          <b
            className="training-marker failure"
            style={{ left: percent(failureFrame) }}
          >
            ×
            <span className="training-tooltip">失败发生在 F{failureFrame}</span>
          </b>
        )}
        {inViewport(resetFrame) && (
          <button
            className="training-marker reset"
            style={{ left: percent(resetFrame) }}
            title={`R 点 F${resetFrame}`}
            onClick={(event) => {
              event.stopPropagation();
              onSetReset(resetFrame);
            }}
          >
            R
          </button>
        )}
        <b className="training-playhead" style={{ left: percent(frame) }} />
      </div>
      <div className="training-timeline-actions">
        <span className="training-legend">
          <i className="objective" />
          悬停查看该帧操作的 Objective
          <i className="fuzz" />
          操作起点
          <i className="target" />
          最佳点
          <i className="window" />
          成功窗口
          <i className="input" />
          你的输入
          <i className="failure" />
          失败
          <i className="reset" />R 点
        </span>
        <button onClick={() => onSetReset(frame)}>设为 R 点 F{frame}</button>
      </div>
    </section>
  );
}

export interface TrainingResultTimelineProps {
  targetFrame?: number;
  windows: FrameWindow[];
  actualInputs: readonly { frame: number; keys: readonly string[] }[];
  failureFrame?: number;
  objectives: TrainingObjectiveSeries[];
}

export function TrainingResultTimeline({
  targetFrame,
  windows,
  actualInputs,
  failureFrame,
  objectives,
}: TrainingResultTimelineProps) {
  const inputFrames = actualInputs.map((input) => input.frame);
  const points = [
    ...inputFrames,
    ...windows.flatMap((window) => [window.from, window.to]),
    ...(targetFrame === undefined ? [] : [targetFrame]),
    ...(failureFrame === undefined ? [] : [failureFrame]),
  ];
  const minimum = points.length === 0 ? 0 : Math.min(...points);
  const maximum = Math.max(...points, minimum + 16);
  const padding = 3;
  const from = minimum - padding;
  const span = Math.max(16, maximum - minimum + padding * 2);
  const percent = (value: number) => `${((value - from) / span) * 100}%`;
  const outputAt = (value: number) => {
    const objective = objectives[0];
    const point = objective?.points.find(
      (candidate) => candidate.frame === value,
    );
    return objective && point
      ? formatObjectiveOutput(objective.expression, point.value)
      : undefined;
  };
  return (
    <div className="training-result-timeline" aria-label="本次操作时间线">
      <ObjectiveCurve series={objectives} from={from} to={from + span} />
      <ObjectiveHoverLayer
        series={objectives}
        from={from}
        to={from + span}
        targetFrame={targetFrame}
        windows={windows}
        actualInputs={actualInputs}
        failureFrame={failureFrame}
      />
      {windows.map((window, index) => (
        <i
          key={`${window.from}-${window.to}-${index}`}
          className="training-window"
          style={{
            left: percent(window.from),
            width: `${Math.max(1.5, ((window.to - window.from + 1) / span) * 100)}%`,
          }}
        />
      ))}
      {targetFrame !== undefined && (
        <b
          className="training-result-target"
          style={{ left: percent(targetFrame) }}
        >
          <span>
            最佳操作 F{targetFrame}
            {outputAt(targetFrame) ? ` · ${outputAt(targetFrame)}` : ""}
          </span>
        </b>
      )}
      {actualInputs.map((input, index) => (
        <b
          key={`${input.frame}-${index}`}
          className={`training-result-input label-row-${(index % 3) + 1}`}
          style={{ left: percent(input.frame) }}
        >
          <span>
            实际 F{input.frame} {input.keys.join("+").toUpperCase()}
            {outputAt(input.frame) ? ` · ${outputAt(input.frame)}` : ""}
          </span>
        </b>
      ))}
      {failureFrame !== undefined && (
        <b
          className="training-result-failure"
          style={{ left: percent(failureFrame) }}
        >
          <span>失败 F{failureFrame}</span>
        </b>
      )}
    </div>
  );
}
