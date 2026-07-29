import { render, screen, within } from "@testing-library/react";
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

  it("draws frame values as midpoint steps without interpolated slopes", () => {
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
    expect(points).toHaveLength(7);
    expect(points?.[1]?.[1]).toBe(points?.[0]?.[1]);
    expect(points?.[2]?.[0]).toBe(points?.[1]?.[0]);
    expect(points?.[3]?.[1]).toBe(points?.[2]?.[1]);
    expect(points?.[4]?.[1]).toBe(points?.[3]?.[1]);
    expect(points?.[5]?.[0]).toBe(points?.[4]?.[0]);
    expect(points?.[6]?.[1]).toBe(points?.[5]?.[1]);
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
    expect(timeline.getAllByText("在 F0 操作")).toHaveLength(1);
    expect(
      timeline.getByLabelText(
        "在 F2 操作：未通过候选；水平速度 191.67 px/s",
      ),
    ).toBeInTheDocument();
    expect(timeline.getByText("未通过候选")).toBeInTheDocument();
    expect(timeline.getAllByText("水平速度")).toHaveLength(3);
    expect(timeline.getByText("191.67 px/s")).toBeInTheDocument();
    expect(timeline.queryByText("终态未满足成功条件")).not.toBeInTheDocument();

    const hit = container.querySelector<HTMLElement>(".training-objective-hit");
    expect(Number.parseFloat(hit?.style.width ?? "")).toBeCloseTo(100 / 22);
  });
});
