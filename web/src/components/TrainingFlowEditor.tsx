import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { createInitialState, type KeyBindings } from "../model";
import type {
  TrainingDocument,
  TrainingModule,
  TrainingTrigger,
  TrainingVariant,
} from "../training/catalog";
import {
  createTrainingModule,
  validateTrainingProject,
  type ProjectValidationIssue,
  type TrainingProject,
} from "../training/editorProject";
import type { TrainingInput } from "../training/session";
import { WasmClient } from "../simulator/wasmClient";
import type { VisualTheme } from "../visualThemes";
import { GameView } from "./GameView";
import { TrainingGround } from "./TrainingGround";

type Target = { type: "module"; index: number } | { type: "finish" };
type TestResult = { id: string; ok: boolean; detail: string };

const KEY_OPTIONS = [
  "up",
  "down",
  "left",
  "right",
  "jump",
  "dash",
  "crouch_dash",
  "grab",
] as const;
const LIMIT_FIELDS = [
  "max_candidates",
  "max_input_frames",
  "max_trie_nodes",
  "max_cache_bytes",
  "max_expression_operations",
] as const;

function scalar(value: string): number | string {
  const trimmed = value.trim();
  return /^-?\d+$/.test(trimmed) ? Number(trimmed) : trimmed;
}

function cloneProject(project: TrainingProject): TrainingProject {
  return structuredClone(project);
}

function pointInMap(
  clientX: number,
  clientY: number,
  svg: SVGSVGElement,
  project: TrainingProject,
) {
  const rect = svg.getBoundingClientRect();
  const bounds = project.map.bounds;
  const scale = Math.min(
    rect.width / bounds.width,
    rect.height / bounds.height,
  );
  const offsetX = (rect.width - bounds.width * scale) / 2;
  const offsetY = (rect.height - bounds.height * scale) / 2;
  return {
    x: bounds.x + (clientX - rect.left - offsetX) / scale,
    y: bounds.y + (clientY - rect.top - offsetY) / scale,
  };
}

function JsonField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: unknown;
  onChange: (value: unknown) => void;
}) {
  const serialized = JSON.stringify(value, null, 2);
  const [text, setText] = useState(serialized);
  const [error, setError] = useState("");
  useEffect(() => {
    setText(serialized);
    setError("");
  }, [serialized]);
  return (
    <label className="training-json-field">
      <small>{label}</small>
      <textarea
        spellCheck={false}
        value={text}
        onChange={(event) => {
          const next = event.target.value;
          setText(next);
          try {
            onChange(JSON.parse(next));
            setError("");
          } catch (caught) {
            setError(caught instanceof Error ? caught.message : "JSON 无效");
          }
        }}
      />
      {error && <em>{error}</em>}
    </label>
  );
}

function TextList({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string[];
  onChange: (value: string[]) => void;
}) {
  return (
    <label className="training-text-list">
      <small>{label}</small>
      <textarea
        value={value.join("\n")}
        onChange={(event) =>
          onChange(event.target.value.split("\n").filter((line) => line.trim()))
        }
      />
    </label>
  );
}

function BoundsFields({
  value,
  onChange,
}: {
  value: TrainingTrigger["bounds"];
  onChange: (value: TrainingTrigger["bounds"]) => void;
}) {
  return (
    <div className="training-bounds-fields">
      {(["x", "y", "width", "height"] as const).map((field) => (
        <label key={field}>
          <small>{field.toUpperCase()}</small>
          <input
            type="number"
            value={value[field]}
            onChange={(event) =>
              onChange({ ...value, [field]: Number(event.target.value) })
            }
          />
        </label>
      ))}
    </div>
  );
}

function TeachingFields({
  module,
  change,
}: {
  module: TrainingModule;
  change: (mutator: (module: TrainingModule) => void) => void;
}) {
  return (
    <fieldset>
      <legend>教学步骤</legend>
      {module.tutorial.teaching.steps.map((step, index) => (
        <article className="training-step-editor" key={index}>
          <header>
            <strong>STEP {index + 1}</strong>
            <button
              onClick={() =>
                change((draft) => {
                  draft.tutorial.teaching.steps.splice(index, 1);
                })
              }
            >
              删除
            </button>
          </header>
          <label>
            <small>PROMPT</small>
            <input
              value={step.prompt}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.teaching.steps[index].prompt =
                    event.target.value;
                })
              }
            />
          </label>
          <div className="training-paired-fields">
            <label>
              <small>顺序错误标题</small>
              <input
                value={step.order_error.title}
                onChange={(event) =>
                  change((draft) => {
                    draft.tutorial.teaching.steps[index].order_error.title =
                      event.target.value;
                  })
                }
              />
            </label>
            <label>
              <small>窗口错误标题</small>
              <input
                value={step.window_error.title}
                onChange={(event) =>
                  change((draft) => {
                    draft.tutorial.teaching.steps[index].window_error.title =
                      event.target.value;
                  })
                }
              />
            </label>
          </div>
          <div className="training-paired-fields">
            <label>
              <small>顺序错误说明</small>
              <textarea
                value={step.order_error.body}
                onChange={(event) =>
                  change((draft) => {
                    draft.tutorial.teaching.steps[index].order_error.body =
                      event.target.value;
                  })
                }
              />
            </label>
            <label>
              <small>窗口错误说明</small>
              <textarea
                value={step.window_error.body}
                onChange={(event) =>
                  change((draft) => {
                    draft.tutorial.teaching.steps[index].window_error.body =
                      event.target.value;
                  })
                }
              />
            </label>
          </div>
        </article>
      ))}
      <button
        className="training-add-row"
        onClick={() =>
          change((draft) => {
            draft.tutorial.teaching.steps.push({
              prompt: "完成下一个动作。",
              order_error: { title: "动作顺序不正确", body: "请按提示输入。" },
              window_error: {
                title: "错过输入窗口",
                body: "请在可行窗口内输入。",
              },
            });
          })
        }
      >
        ＋ 添加步骤
      </button>
    </fieldset>
  );
}

function FuzzFields({
  module,
  change,
}: {
  module: TrainingModule;
  change: (mutator: (module: TrainingModule) => void) => void;
}) {
  const fuzz = module.tutorial.fuzz;
  return (
    <fieldset className="fuzz-fields">
      <legend>Fuzz v1</legend>
      <label>
        <small>OBSERVE UNTIL</small>
        <input
          value={String(fuzz.observe_until)}
          onChange={(event) =>
            change((draft) => {
              draft.tutorial.fuzz.observe_until = scalar(event.target.value);
            })
          }
        />
      </label>
      <TextList
        label="SUCCESS · 每行一个 Rhai 条件"
        value={fuzz.success}
        onChange={(value) =>
          change((draft) => {
            draft.tutorial.fuzz.success = value;
          })
        }
      />
      <h4>变量</h4>
      {fuzz.variables.map((variable, index) => (
        <div className="training-row variable" key={index}>
          <input
            aria-label="变量名"
            placeholder="name"
            value={variable.name}
            onChange={(event) =>
              change((draft) => {
                draft.tutorial.fuzz.variables[index].name = event.target.value;
              })
            }
          />
          <input
            aria-label="起点"
            placeholder="from"
            value={String(variable.range.from)}
            onChange={(event) =>
              change((draft) => {
                draft.tutorial.fuzz.variables[index].range.from = scalar(
                  event.target.value,
                );
              })
            }
          />
          <input
            aria-label="终点"
            placeholder="to"
            value={String(variable.range.to)}
            onChange={(event) =>
              change((draft) => {
                draft.tutorial.fuzz.variables[index].range.to = scalar(
                  event.target.value,
                );
              })
            }
          />
          <input
            aria-label="步长"
            type="number"
            placeholder="step"
            value={variable.range.step ?? ""}
            onChange={(event) =>
              change((draft) => {
                const value = Number(event.target.value);
                if (value)
                  draft.tutorial.fuzz.variables[index].range.step = value;
                else delete draft.tutorial.fuzz.variables[index].range.step;
              })
            }
          />
          <button
            onClick={() =>
              change((draft) => {
                draft.tutorial.fuzz.variables.splice(index, 1);
              })
            }
          >
            ×
          </button>
        </div>
      ))}
      <button
        className="training-add-row"
        onClick={() =>
          change((draft) => {
            draft.tutorial.fuzz.variables.push({
              name: `frame_${draft.tutorial.fuzz.variables.length + 1}`,
              range: { from: 0, to: 30 },
            });
          })
        }
      >
        ＋ 添加变量
      </button>
      <h4>输入</h4>
      {fuzz.inputs.map((input, index) => (
        <article className="fuzz-input-editor" key={index}>
          <header>
            <strong>INPUT {index + 1}</strong>
            <button
              onClick={() =>
                change((draft) => {
                  draft.tutorial.fuzz.inputs.splice(index, 1);
                })
              }
            >
              删除
            </button>
          </header>
          <div className="training-paired-fields">
            <label>
              <small>ID</small>
              <input
                value={input.id}
                onChange={(event) =>
                  change((draft) => {
                    draft.tutorial.fuzz.inputs[index].id = event.target.value;
                  })
                }
              />
            </label>
            <label>
              <small>AT</small>
              <input
                value={String(input.at)}
                onChange={(event) =>
                  change((draft) => {
                    draft.tutorial.fuzz.inputs[index].at = scalar(
                      event.target.value,
                    );
                  })
                }
              />
            </label>
          </div>
          <div className="training-key-grid">
            {KEY_OPTIONS.map((key) => (
              <label key={key}>
                <input
                  type="checkbox"
                  checked={input.keys.includes(key)}
                  onChange={(event) =>
                    change((draft) => {
                      const keys = new Set(
                        draft.tutorial.fuzz.inputs[index].keys,
                      );
                      if (event.target.checked) keys.add(key);
                      else keys.delete(key);
                      draft.tutorial.fuzz.inputs[index].keys = [...keys];
                    })
                  }
                />
                {key}
              </label>
            ))}
          </div>
          <div className="training-paired-fields">
            <label>
              <small>HELD TIME</small>
              <input
                placeholder="留空 / 数字 / hold::inf"
                value={
                  input.held_time === undefined ? "" : String(input.held_time)
                }
                onChange={(event) =>
                  change((draft) => {
                    if (!event.target.value)
                      delete draft.tutorial.fuzz.inputs[index].held_time;
                    else
                      draft.tutorial.fuzz.inputs[index].held_time = scalar(
                        event.target.value,
                      );
                  })
                }
              />
            </label>
            <label className="training-check">
              <input
                type="checkbox"
                checked={input.verify !== false}
                onChange={(event) =>
                  change((draft) => {
                    draft.tutorial.fuzz.inputs[index].verify =
                      event.target.checked;
                  })
                }
              />
              需要玩家命中
            </label>
          </div>
          <div className="training-paired-fields">
            <label>
              <small>BEFORE INPUT</small>
              <textarea
                value={
                  Array.isArray(input.before_input)
                    ? input.before_input.join("\n")
                    : (input.before_input ?? "")
                }
                onChange={(event) =>
                  change((draft) => {
                    const lines = event.target.value
                      .split("\n")
                      .filter(Boolean);
                    if (!lines.length)
                      delete draft.tutorial.fuzz.inputs[index].before_input;
                    else draft.tutorial.fuzz.inputs[index].before_input = lines;
                  })
                }
              />
            </label>
            <label>
              <small>AFTER INPUT</small>
              <textarea
                value={
                  Array.isArray(input.after_input)
                    ? input.after_input.join("\n")
                    : (input.after_input ?? "")
                }
                onChange={(event) =>
                  change((draft) => {
                    const lines = event.target.value
                      .split("\n")
                      .filter(Boolean);
                    if (!lines.length)
                      delete draft.tutorial.fuzz.inputs[index].after_input;
                    else draft.tutorial.fuzz.inputs[index].after_input = lines;
                  })
                }
              />
            </label>
          </div>
        </article>
      ))}
      <button
        className="training-add-row"
        onClick={() =>
          change((draft) => {
            const input: TrainingInput = {
              id: `input-${draft.tutorial.fuzz.inputs.length + 1}`,
              keys: ["jump"],
              at: 0,
              verify: true,
            };
            draft.tutorial.fuzz.inputs.push(input);
          })
        }
      >
        ＋ 添加输入
      </button>
      <h4>排序目标</h4>
      {fuzz.objectives.map((objective, index) => (
        <div className="training-row objective" key={index}>
          <select
            value={objective.type}
            onChange={(event) =>
              change((draft) => {
                const current = draft.tutorial.fuzz.objectives[index];
                draft.tutorial.fuzz.objectives[index] =
                  event.target.value === "approach"
                    ? {
                        type: "approach",
                        expression: current.expression,
                        target: 0,
                      }
                    : {
                        type: event.target.value as "maximize" | "minimize",
                        expression: current.expression,
                      };
              })
            }
          >
            <option value="maximize">maximize</option>
            <option value="minimize">minimize</option>
            <option value="approach">approach</option>
          </select>
          <input
            value={objective.expression}
            onChange={(event) =>
              change((draft) => {
                draft.tutorial.fuzz.objectives[index].expression =
                  event.target.value;
              })
            }
          />
          {objective.type === "approach" && (
            <input
              aria-label="目标值"
              type="number"
              value={objective.target}
              onChange={(event) =>
                change((draft) => {
                  const item = draft.tutorial.fuzz.objectives[index];
                  if (item.type === "approach")
                    item.target = Number(event.target.value);
                })
              }
            />
          )}
          <button
            onClick={() =>
              change((draft) => {
                draft.tutorial.fuzz.objectives.splice(index, 1);
              })
            }
          >
            ×
          </button>
        </div>
      ))}
      <button
        className="training-add-row"
        onClick={() =>
          change((draft) => {
            draft.tutorial.fuzz.objectives.push({
              type: "maximize",
              expression: "final.speed.x",
            });
          })
        }
      >
        ＋ 添加目标
      </button>
      <div className="training-paired-fields">
        <JsonField
          label="SEARCH BINDINGS"
          value={fuzz.search.bindings}
          onChange={(value) =>
            change((draft) => {
              draft.tutorial.fuzz.search.bindings = value as Record<
                string,
                number
              >;
            })
          }
        />
        <label>
          <small>SEARCH OUTPUT · 逗号分隔</small>
          <textarea
            value={fuzz.search.output.join(", ")}
            onChange={(event) =>
              change((draft) => {
                draft.tutorial.fuzz.search.output = event.target.value
                  .split(",")
                  .map((item) => item.trim())
                  .filter(Boolean);
              })
            }
          />
        </label>
      </div>
      <h4>资源上限</h4>
      <div className="training-limit-grid">
        {LIMIT_FIELDS.map((field) => (
          <label key={field}>
            <small>{field}</small>
            <input
              type="number"
              placeholder="默认"
              value={fuzz.limits?.[field] ?? ""}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.fuzz.limits ??= {};
                  const value = Number(event.target.value);
                  if (value) draft.tutorial.fuzz.limits[field] = value;
                  else delete draft.tutorial.fuzz.limits[field];
                })
              }
            />
          </label>
        ))}
      </div>
    </fieldset>
  );
}

function ModuleInspector({
  module,
  onChange,
}: {
  module: TrainingModule;
  onChange: (module: TrainingModule) => void;
}) {
  const change = (mutator: (module: TrainingModule) => void) => {
    const next = structuredClone(module);
    mutator(next);
    onChange(next);
  };
  return (
    <div className="training-inspector-fields">
      <fieldset>
        <legend>模块与 Trigger</legend>
        <div className="training-paired-fields">
          <label>
            <small>MODULE ID</small>
            <input
              value={module.id}
              onChange={(event) =>
                change((draft) => {
                  draft.id = event.target.value;
                })
              }
            />
          </label>
          <label>
            <small>TRIGGER ID</small>
            <input
              value={module.trigger.id}
              onChange={(event) =>
                change((draft) => {
                  draft.trigger.id = event.target.value;
                })
              }
            />
          </label>
        </div>
        <BoundsFields
          value={module.trigger.bounds}
          onChange={(value) =>
            change((draft) => {
              draft.trigger.bounds = value;
            })
          }
        />
      </fieldset>
      <fieldset>
        <legend>教程</legend>
        <div className="training-paired-fields">
          <label>
            <small>TUTORIAL ID</small>
            <input
              value={module.tutorial.id}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.id = event.target.value;
                })
              }
            />
          </label>
          <label>
            <small>TITLE</small>
            <input
              value={module.tutorial.title}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.title = event.target.value;
                })
              }
            />
          </label>
        </div>
        <label>
          <small>SUMMARY</small>
          <textarea
            value={module.tutorial.summary}
            onChange={(event) =>
              change((draft) => {
                draft.tutorial.summary = event.target.value;
              })
            }
          />
        </label>
      </fieldset>
      <fieldset>
        <legend>入口</legend>
        <label>
          <small>ENTRY INPUT ID</small>
          <input
            value={module.tutorial.entry.input_id}
            onChange={(event) =>
              change((draft) => {
                draft.tutorial.entry.input_id = event.target.value;
              })
            }
          />
        </label>
        <label>
          <small>HINT</small>
          <textarea
            value={module.tutorial.entry.hint}
            onChange={(event) =>
              change((draft) => {
                draft.tutorial.entry.hint = event.target.value;
              })
            }
          />
        </label>
        <TextList
          label="CHECK · 每行一个 Rhai 条件"
          value={module.tutorial.entry.check}
          onChange={(value) =>
            change((draft) => {
              draft.tutorial.entry.check = value;
            })
          }
        />
        <div className="training-paired-fields">
          <label>
            <small>失败标题</small>
            <input
              value={module.tutorial.entry.failure.title}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.entry.failure.title = event.target.value;
                })
              }
            />
          </label>
          <label>
            <small>失败说明</small>
            <textarea
              value={module.tutorial.entry.failure.body}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.entry.failure.body = event.target.value;
                })
              }
            />
          </label>
        </div>
      </fieldset>
      <TeachingFields module={module} change={change} />
      <fieldset>
        <legend>辅助</legend>
        <div className="training-paired-fields">
          <label>
            <small>结果延迟帧</small>
            <input
              type="number"
              value={module.tutorial.assist.result_sample_after_input_frames}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.assist.result_sample_after_input_frames =
                    Number(event.target.value);
                })
              }
            />
          </label>
          <label className="training-check">
            <input
              type="checkbox"
              checked={module.tutorial.assist.auto_slowdown.enabled_by_default}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.assist.auto_slowdown.enabled_by_default =
                    event.target.checked;
                })
              }
            />
            默认自动慢放
          </label>
          <label>
            <small>慢放半径帧</small>
            <input
              type="number"
              value={module.tutorial.assist.auto_slowdown.radius_frames}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.assist.auto_slowdown.radius_frames = Number(
                    event.target.value,
                  );
                })
              }
            />
          </label>
          <label>
            <small>最低倍率</small>
            <input
              type="number"
              step="0.05"
              value={module.tutorial.assist.auto_slowdown.minimum_multiplier}
              onChange={(event) =>
                change((draft) => {
                  draft.tutorial.assist.auto_slowdown.minimum_multiplier =
                    Number(event.target.value);
                })
              }
            />
          </label>
        </div>
      </fieldset>
      <FuzzFields module={module} change={change} />
      <fieldset>
        <legend>离线校验</legend>
        <JsonField
          label="VALIDATION INITIAL STATE"
          value={module.validation.initial_state}
          onChange={(value) =>
            change((draft) => {
              draft.validation.initial_state =
                value as TrainingModule["validation"]["initial_state"];
            })
          }
        />
        <label className="training-check">
          <input
            type="checkbox"
            checked={Boolean(module.validation.fuzz)}
            onChange={(event) =>
              change((draft) => {
                if (event.target.checked)
                  draft.validation.fuzz = structuredClone(draft.tutorial.fuzz);
                else delete draft.validation.fuzz;
              })
            }
          />
          使用单独的 validation.fuzz
        </label>
        {module.validation.fuzz && (
          <JsonField
            label="VALIDATION FUZZ"
            value={module.validation.fuzz}
            onChange={(value) =>
              change((draft) => {
                draft.validation.fuzz = value as TrainingDocument["fuzz"];
              })
            }
          />
        )}
      </fieldset>
    </div>
  );
}

export function TrainingFlowEditor({
  project,
  theme,
  bindings,
  ready,
  onChange,
}: {
  project: TrainingProject;
  theme: VisualTheme;
  bindings: KeyBindings;
  ready: boolean;
  onChange: (project: TrainingProject) => void;
}) {
  const client = useMemo(() => new WasmClient(), []);
  const [target, setTarget] = useState<Target>({ type: "module", index: 0 });
  const [raw, setRaw] = useState(false);
  const [preview, setPreview] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResults, setTestResults] = useState<TestResult[]>([]);
  const drag = useRef<{
    target: Target;
    point: { x: number; y: number };
    bounds: TrainingTrigger["bounds"];
  } | null>(null);
  useEffect(() => () => client.dispose(), [client]);

  const issues = validateTrainingProject(project);
  const selectedModule =
    target.type === "module" ? project.training.modules[target.index] : null;
  const selectedTrigger =
    target.type === "module"
      ? selectedModule?.trigger
      : project.training.finish.trigger;
  const variant: TrainingVariant = useMemo(
    () => ({
      id: project.id,
      title: project.training.title,
      summary: project.training.summary,
      map: project.map,
      training: project.training,
      initial: createInitialState(project.map),
    }),
    [project],
  );

  const updateTraining = (mutator: (project: TrainingProject) => void) => {
    const next = cloneProject(project);
    mutator(next);
    onChange(next);
  };
  const updateModule = (index: number, module: TrainingModule) =>
    updateTraining((draft) => {
      draft.training.modules[index] = module;
    });

  const runTests = async () => {
    setTesting(true);
    const next: TestResult[] = issues.map((item) => ({
      id: item.path,
      ok: item.severity !== "error",
      detail: `${item.path}: ${item.message}`,
    }));
    if (!issues.some((item) => item.severity === "error")) {
      try {
        await client.ready();
        for (const module of project.training.modules) {
          try {
            const result = await client.fuzzSearch(
              module.validation.initial_state,
              JSON.stringify(module.validation.fuzz ?? module.tutorial.fuzz),
              project.map,
            );
            const best = result.candidates[0];
            next.push({
              id: module.id,
              ok: Boolean(best),
              detail: best
                ? `${module.tutorial.title}: ${result.candidates.length} 条成功路线 · objective ${best.objective_values.map((value) => value.toFixed(2)).join(", ") || "—"}`
                : `${module.tutorial.title}: 没有成功候选`,
            });
          } catch (error) {
            next.push({
              id: module.id,
              ok: false,
              detail: `${module.tutorial.title}: ${error instanceof Error ? error.message : "WASM Fuzz 失败"}`,
            });
          }
        }
      } catch (error) {
        next.push({
          id: "wasm",
          ok: false,
          detail: error instanceof Error ? error.message : "WASM 启动失败",
        });
      }
    }
    setTestResults(
      next.length
        ? next
        : [
            {
              id: "schema",
              ok: true,
              detail: "Schema、Trigger 与入口检查通过",
            },
          ],
    );
    setTesting(false);
  };

  const beginDrag = (
    event: ReactPointerEvent<SVGRectElement>,
    nextTarget: Target,
    bounds: TrainingTrigger["bounds"],
  ) => {
    event.stopPropagation();
    const svg = event.currentTarget.ownerSVGElement;
    if (!svg) return;
    drag.current = {
      target: nextTarget,
      point: pointInMap(event.clientX, event.clientY, svg, project),
      bounds: { ...bounds },
    };
    setTarget(nextTarget);
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const moveDrag = (event: ReactPointerEvent<SVGSVGElement>) => {
    if (!drag.current) return;
    const point = pointInMap(
      event.clientX,
      event.clientY,
      event.currentTarget,
      project,
    );
    const dx = Math.round((point.x - drag.current.point.x) / 8) * 8;
    const dy = Math.round((point.y - drag.current.point.y) / 8) * 8;
    const bounds = {
      ...drag.current.bounds,
      x: drag.current.bounds.x + dx,
      y: drag.current.bounds.y + dy,
    };
    updateTraining((draft) => {
      if (drag.current?.target.type === "finish")
        draft.training.finish.trigger.bounds = bounds;
      else if (drag.current?.target.type === "module")
        draft.training.modules[drag.current.target.index].trigger.bounds =
          bounds;
    });
  };

  if (preview)
    return (
      <div className="training-editor-preview">
        <div className="training-preview-bar">
          <strong>训练流程实时测试</strong>
          <span>使用当前未保存内容 · Trigger/Fuzz/结算均按训练模式运行</span>
          <button onClick={() => setPreview(false)}>返回编辑</button>
        </div>
        <TrainingGround
          techniqueId="hyper"
          variantId={project.id}
          bindings={bindings}
          theme={theme}
          onSelectTraining={() => {}}
          variantOverride={variant}
          editorPreview
        />
      </div>
    );

  return (
    <main className="training-flow-editor">
      <aside className="training-project-tree">
        <header>
          <small>TRAINING MAP V2</small>
          <h1>{project.training.title}</h1>
          <span>{project.training.modules.length} 个模块</span>
        </header>
        <button
          className={target.type === "finish" ? "selected finish" : "finish"}
          onClick={() => setTarget({ type: "finish" })}
        >
          <b>◆</b>
          <span>
            <strong>终点 Trigger</strong>
            <small>{project.training.finish.trigger.id}</small>
          </span>
        </button>
        <div className="training-module-list">
          {project.training.modules.map((module, index) => (
            <button
              className={
                target.type === "module" && target.index === index
                  ? "selected"
                  : ""
              }
              key={`${module.id}-${index}`}
              onClick={() => setTarget({ type: "module", index })}
            >
              <b>{String(index + 1).padStart(2, "0")}</b>
              <span>
                <strong>{module.tutorial.title}</strong>
                <small>
                  {module.id} · {module.trigger.id}
                </small>
              </span>
            </button>
          ))}
        </div>
        <button
          className="training-add-module"
          onClick={() =>
            updateTraining((draft) => {
              const index = draft.training.modules.length;
              draft.training.modules.push(
                createTrainingModule(draft.map, index),
              );
              setTarget({ type: "module", index });
            })
          }
        >
          ＋ 新建教程模块
        </button>
        {target.type === "module" && (
          <div className="training-tree-actions">
            <button
              onClick={() =>
                updateTraining((draft) => {
                  const copy = structuredClone(
                    draft.training.modules[target.index],
                  );
                  copy.id = `${copy.id}-copy`;
                  copy.trigger.id = `${copy.trigger.id}-copy`;
                  copy.tutorial.id = `${copy.tutorial.id}-copy`;
                  draft.training.modules.splice(target.index + 1, 0, copy);
                  setTarget({ type: "module", index: target.index + 1 });
                })
              }
            >
              复制
            </button>
            <button
              onClick={() =>
                updateTraining((draft) => {
                  draft.training.modules.splice(target.index, 1);
                  setTarget({
                    type: "module",
                    index: Math.max(0, target.index - 1),
                  });
                })
              }
            >
              删除
            </button>
          </div>
        )}
        <div className="training-validation-summary">
          <strong>
            {issues.filter((item) => item.severity === "error").length
              ? `${issues.filter((item) => item.severity === "error").length} 个错误`
              : "结构检查通过"}
          </strong>
          <span>{issues[0]?.message ?? "可以运行 WASM Fuzz 测试"}</span>
        </div>
      </aside>

      <section className="training-editor-stage">
        <div className="training-editor-toolbar">
          <div>
            <small>TRIGGER LAYOUT</small>
            <strong>{selectedModule?.tutorial.title ?? "终点区域"}</strong>
          </div>
          <button disabled={!ready || testing} onClick={() => void runTests()}>
            {testing ? "测试中…" : "运行全部 Fuzz 测试"}
          </button>
          <button
            className="primary"
            disabled={
              !ready || issues.some((item) => item.severity === "error")
            }
            onClick={() => setPreview(true)}
          >
            ▶ 实时测试训练流程
          </button>
        </div>
        <GameView
          map={project.map}
          state={createInitialState(project.map)}
          states={[]}
          frame={0}
          stale={false}
          theme={theme}
        >
          <svg
            className="training-trigger-overlay"
            viewBox={`${project.map.bounds.x} ${project.map.bounds.y} ${project.map.bounds.width} ${project.map.bounds.height}`}
            preserveAspectRatio="xMidYMid meet"
            onPointerMove={moveDrag}
            onPointerUp={() => {
              drag.current = null;
            }}
            onPointerCancel={() => {
              drag.current = null;
            }}
          >
            {project.training.modules.map((module, index) => (
              <g key={module.id}>
                <rect
                  className={
                    target.type === "module" && target.index === index
                      ? "module selected"
                      : "module"
                  }
                  {...module.trigger.bounds}
                  onPointerDown={(event) =>
                    beginDrag(
                      event,
                      { type: "module", index },
                      module.trigger.bounds,
                    )
                  }
                />
                <text
                  x={module.trigger.bounds.x + 3}
                  y={module.trigger.bounds.y + 10}
                >
                  {index + 1} · {module.tutorial.title}
                </text>
              </g>
            ))}
            <g>
              <rect
                className={
                  target.type === "finish" ? "finish selected" : "finish"
                }
                {...project.training.finish.trigger.bounds}
                onPointerDown={(event) =>
                  beginDrag(
                    event,
                    { type: "finish" },
                    project.training.finish.trigger.bounds,
                  )
                }
              />
              <text
                x={project.training.finish.trigger.bounds.x + 3}
                y={project.training.finish.trigger.bounds.y + 10}
              >
                FINISH
              </text>
            </g>
          </svg>
        </GameView>
        <section className="training-test-results" aria-live="polite">
          <header>
            <strong>测试结果</strong>
            <span>
              {testResults.length
                ? `${testResults.filter((item) => item.ok).length}/${testResults.length} 通过`
                : "尚未运行"}
            </span>
          </header>
          {testResults.map((result) => (
            <p className={result.ok ? "passed" : "failed"} key={result.id}>
              <i>{result.ok ? "✓" : "!"}</i>
              {result.detail}
            </p>
          ))}
        </section>
      </section>

      <aside className="training-flow-inspector">
        <header>
          <div>
            <small>FLOW INSPECTOR</small>
            <h2>{selectedModule?.tutorial.title ?? "地图与终点"}</h2>
          </div>
          <button onClick={() => setRaw((value) => !value)}>
            {raw ? "表单" : "JSON"}
          </button>
        </header>
        {raw ? (
          <JsonField
            label="完整 TrainingMapDocument"
            value={project.training}
            onChange={(value) =>
              updateTraining((draft) => {
                draft.training = value as TrainingProject["training"];
              })
            }
          />
        ) : selectedModule ? (
          <ModuleInspector
            module={selectedModule}
            onChange={(module) =>
              updateModule(target.type === "module" ? target.index : 0, module)
            }
          />
        ) : (
          <div className="training-inspector-fields">
            <fieldset>
              <legend>地图脚本</legend>
              <label>
                <small>ID</small>
                <input
                  value={project.training.id}
                  onChange={(event) =>
                    updateTraining((draft) => {
                      draft.training.id = event.target.value;
                    })
                  }
                />
              </label>
              <label>
                <small>TITLE</small>
                <input
                  value={project.training.title}
                  onChange={(event) =>
                    updateTraining((draft) => {
                      draft.training.title = event.target.value;
                    })
                  }
                />
              </label>
              <label>
                <small>SUMMARY</small>
                <textarea
                  value={project.training.summary}
                  onChange={(event) =>
                    updateTraining((draft) => {
                      draft.training.summary = event.target.value;
                    })
                  }
                />
              </label>
            </fieldset>
            <fieldset>
              <legend>终点 Trigger</legend>
              <label>
                <small>ID</small>
                <input
                  value={project.training.finish.trigger.id}
                  onChange={(event) =>
                    updateTraining((draft) => {
                      draft.training.finish.trigger.id = event.target.value;
                    })
                  }
                />
              </label>
              <BoundsFields
                value={project.training.finish.trigger.bounds}
                onChange={(value) =>
                  updateTraining((draft) => {
                    draft.training.finish.trigger.bounds = value;
                  })
                }
              />
              <label className="training-check">
                <input
                  type="checkbox"
                  checked={project.training.finish.require_all_modules}
                  onChange={(event) =>
                    updateTraining((draft) => {
                      draft.training.finish.require_all_modules =
                        event.target.checked;
                    })
                  }
                />
                需要完成全部模块
              </label>
            </fieldset>
          </div>
        )}
        {!raw && issues.length > 0 && (
          <section className="training-issue-list">
            <strong>结构检查</strong>
            {issues.map((item: ProjectValidationIssue) => (
              <p className={item.severity} key={`${item.path}-${item.message}`}>
                <code>{item.path}</code>
                {item.message}
              </p>
            ))}
          </section>
        )}
      </aside>
    </main>
  );
}
