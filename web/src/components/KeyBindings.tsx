import { useEffect, useState } from "react";
import {
  ACTIONS,
  ACTION_GLYPHS,
  ACTION_LABELS,
  bindingLabel,
  type Action,
  type KeyBindings as Bindings,
} from "../model";
import type { GamepadDirectionSource } from "../input/gamepad";

interface KeyBindingsProps {
  bindings: Bindings;
  gamepadDirectionSource: GamepadDirectionSource;
  gamepadName: string | null;
  gamepadSupported: boolean;
  onChange(action: Action, code: string): void;
  onGamepadDirectionSourceChange(source: GamepadDirectionSource): void;
  onClose(): void;
}

export function KeyBindings({
  bindings,
  gamepadDirectionSource,
  gamepadName,
  gamepadSupported,
  onChange,
  onGamepadDirectionSourceChange,
  onClose,
}: KeyBindingsProps) {
  const [listening, setListening] = useState<Action | null>(null);

  useEffect(() => {
    if (!listening) return;
    const capture = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.code === "Escape") setListening(null);
      else {
        onChange(listening, event.code);
        setListening(null);
      }
    };
    window.addEventListener("keydown", capture, true);
    return () => window.removeEventListener("keydown", capture, true);
  }, [listening, onChange]);

  return (
    <div
      className="modal-backdrop"
      onPointerDown={(event) =>
        event.target === event.currentTarget && onClose()
      }
    >
      <section
        className="bindings-modal panel-frame"
        role="dialog"
        aria-modal="true"
        aria-labelledby="bindings-title"
      >
        <div className="panel-heading">
          <div>
            <small>CONTROLS</small>
            <h2 id="bindings-title">控制设置</h2>
          </div>
          <button className="icon-button" onClick={onClose} aria-label="关闭">
            ×
          </button>
        </div>
        <p className="modal-copy">
          点击一个键位，再按下新按键。重复键位会自动交换。
        </p>
        <div className="binding-list">
          {ACTIONS.map((action) => (
            <button
              key={action}
              className={listening === action ? "binding listening" : "binding"}
              onClick={() => setListening(action)}
            >
              <span className={`action-glyph ${action}`}>
                {ACTION_GLYPHS[action]}
              </span>
              <strong>{ACTION_LABELS[action]}</strong>
              <kbd>
                {listening === action
                  ? "按下按键…"
                  : bindingLabel(bindings[action])}
              </kbd>
            </button>
          ))}
        </div>
        <div className="gamepad-settings">
          <div className="gamepad-heading">
            <div>
              <small>GAMEPAD</small>
              <strong>手柄</strong>
            </div>
            <span className={gamepadName ? "connected" : ""}>
              {!gamepadSupported
                ? "浏览器不支持"
                : gamepadName
                  ? "已连接"
                  : "等待连接"}
            </span>
          </div>
          <label>
            <span>方向输入</span>
            <select
              aria-label="手柄方向输入"
              value={gamepadDirectionSource}
              disabled={!gamepadSupported}
              onChange={(event) =>
                onGamepadDirectionSourceChange(
                  event.target.value as GamepadDirectionSource,
                )
              }
            >
              <option value="stick">左摇杆</option>
              <option value="dpad">十字键</option>
            </select>
          </label>
          {gamepadName && <p title={gamepadName}>{gamepadName}</p>}
          <small>A / × 跳跃　X / □ 冲刺　肩键或扳机抓取</small>
        </div>
      </section>
    </div>
  );
}
