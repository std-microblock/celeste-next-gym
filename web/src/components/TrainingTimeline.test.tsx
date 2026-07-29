import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  TrainingResultTimeline,
  type TrainingObjectiveSeries,
} from "./TrainingTimeline";

const objectives: TrainingObjectiveSeries[] = [
  {
    expression: "final.speed.x",
    points: [
      { frame: 0, value: 325, successful: true },
      { frame: 1, value: 325, successful: true },
      { frame: 2, value: 191.67, successful: false },
    ],
  },
];

describe("training result timeline", () => {
  it("layers the objective curve, feasible window, and timing markers without point glyphs", () => {
    const { container } = render(
      <TrainingResultTimeline
        targetFrame={0}
        windows={[{ from: 0, to: 1 }]}
        actualInputs={[
          { frame: 0, keys: ["dash"] },
          { frame: 1, keys: ["jump"] },
        ]}
        objectives={objectives}
      />,
    );

    expect(
      container.querySelector(".training-objective-curve polyline"),
    ).toBeInTheDocument();
    expect(container.querySelector(".training-window")).toBeInTheDocument();
    expect(container.querySelectorAll("circle")).toHaveLength(0);
    expect(
      container.querySelectorAll(".training-result-axis-end"),
    ).toHaveLength(0);
    expect(screen.getByText("最佳操作 F0 · 325 px/s")).toBeInTheDocument();
    expect(screen.getByText("实际 F0 DASH · 325 px/s")).toBeInTheDocument();
    expect(screen.getByText("实际 F1 JUMP · 325 px/s")).toBeInTheDocument();
    expect(
      screen.getByLabelText(
        "在 F0 操作：Fuzz 最佳点、你的输入、成功窗口；水平速度 325 px/s",
      ),
    ).toBeInTheDocument();
  });

  it("holds each frame output until the following frame boundary", () => {
    const { container } = render(
      <TrainingResultTimeline
        windows={[]}
        actualInputs={[]}
        objectives={objectives}
      />,
    );

    const points = container
      .querySelector(".training-objective-curve polyline")
      ?.getAttribute("points")
      ?.split(" ")
      .map((point) => point.split(",").map(Number));
    expect(points).toHaveLength(6);
    expect(points?.[1]?.[1]).toBe(points?.[0]?.[1]);
    expect(points?.[2]?.[0]).toBe(points?.[1]?.[0]);
    expect(points?.[3]?.[1]).toBe(points?.[2]?.[1]);
    expect(points?.[4]?.[0]).toBe(points?.[3]?.[0]);
    expect(points?.[5]?.[1]).toBe(points?.[4]?.[1]);
    expect(points?.[1]?.[0]).toBeCloseTo(100 / 16);
    expect(points?.[3]?.[0]).toBeCloseTo(200 / 16);
  });

  it("exposes per-frame Fuzzer speed and success explanation through custom hover details", () => {
    const { container } = render(
      <TrainingResultTimeline
        windows={[{ from: 0, to: 1 }]}
        actualInputs={[]}
        objectives={objectives}
      />,
    );

    const timeline = within(container);
    expect(timeline.queryByText("在 F0 操作")).not.toBeInTheDocument();
    expect(
      timeline.getByLabelText(
        "在 F2 操作：未通过候选；水平速度 191.67 px/s",
      ),
    ).toBeInTheDocument();
    const failedHit = timeline.getByLabelText(
      "在 F2 操作：未通过候选；水平速度 191.67 px/s",
    );
    fireEvent.mouseMove(failedHit, { clientX: 240, clientY: 180 });

    const tooltip = screen.getByRole("tooltip");
    expect(tooltip).toHaveTextContent("在 F2 操作");
    expect(tooltip).toHaveTextContent("未通过候选");
    expect(tooltip).toHaveTextContent("水平速度");
    expect(tooltip).toHaveTextContent("191.67 px/s");
    expect(tooltip.parentElement).toBe(document.body);
    expect(tooltip).toHaveStyle({ left: "252px", top: "192px" });
    expect(timeline.queryByText("终态未满足成功条件")).not.toBeInTheDocument();

    const hits = container.querySelectorAll<HTMLElement>(
      ".training-objective-hit",
    );
    expect(Number.parseFloat(hits[0]?.style.width ?? "")).toBeCloseTo(
      100 / 16,
    );
    expect(Number.parseFloat(hits[0]?.style.left ?? "")).toBeCloseTo(0);
    expect(Number.parseFloat(hits[1]?.style.left ?? "")).toBeCloseTo(
      100 / 16,
    );
  });

  it("normalizes result frames to a local F0 origin", () => {
    const shiftedObjectives = objectives.map((objective) => ({
      ...objective,
      points: objective.points.map((point) => ({
        ...point,
        frame: point.frame + 40,
      })),
    }));
    const { container } = render(
      <TrainingResultTimeline
        frameOrigin={40}
        targetFrame={40}
        windows={[{ from: 40, to: 41 }]}
        actualInputs={[{ frame: 40, keys: ["dash"] }]}
        failureFrame={42}
        objectives={shiftedObjectives}
      />,
    );

    const timeline = within(container);
    expect(
      timeline.getByText("最佳操作 F0 · 325 px/s"),
    ).toBeInTheDocument();
    expect(
      timeline.getByText("实际 F0 DASH · 325 px/s"),
    ).toBeInTheDocument();
    expect(timeline.getByText("失败 F2")).toBeInTheDocument();
    expect(
      container.querySelector<HTMLElement>(".training-result-target")?.style
        .left,
    ).toBe("0%");
  });
});
