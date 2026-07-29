import { readFile } from "node:fs/promises";
import { decode, encode } from "@msgpack/msgpack";
import init, {
  cache_simulation_map_msgpack,
  fuzz_search_cached_map_msgpack,
} from "../src/wasm/celeste_wasm.js";
import { trainingCatalog } from "../src/training/catalog.ts";
import { trainingEntryInput } from "../src/training/session.ts";

await init({
  module_or_path: await readFile(
    new URL("../src/wasm/celeste_wasm_bg.wasm", import.meta.url),
  ),
});

let validated = 0;
for (const technique of trainingCatalog) {
  for (const variant of technique.variants) {
    cache_simulation_map_msgpack(encode(variant.map));
    const triggerIds = new Set<string>();
    for (const module of variant.training.modules) {
      if (module.trigger.bounds.width <= 0 || module.trigger.bounds.height <= 0)
        throw new Error(
          `${technique.id}/${variant.id}/${module.id} trigger 尺寸必须为正数`,
        );
      const entry = trainingEntryInput(module.tutorial);
      if (!entry)
        throw new Error(
          `${technique.id}/${variant.id}/${module.id} entry.input_id 未指向可验证输入`,
        );
      if (entry.at !== 0)
        throw new Error(
          `${technique.id}/${variant.id}/${module.id} 入口输入不在本地 F0`,
        );
      if (triggerIds.has(module.trigger.id))
        throw new Error(
          `${technique.id}/${variant.id} trigger id ${module.trigger.id} 重复`,
        );
      triggerIds.add(module.trigger.id);
      if (
        module.end_trigger.bounds.width <= 0 ||
        module.end_trigger.bounds.height <= 0
      )
        throw new Error(
          `${technique.id}/${variant.id}/${module.id} end trigger 尺寸必须为正数`,
        );
      if (triggerIds.has(module.end_trigger.id))
        throw new Error(
          `${technique.id}/${variant.id} trigger id ${module.end_trigger.id} 重复`,
        );
      triggerIds.add(module.end_trigger.id);
      const result = decode(
        fuzz_search_cached_map_msgpack(
          encode(module.validation.initial_state),
          JSON.stringify(module.validation.fuzz ?? module.tutorial.fuzz),
        ),
      ) as {
        candidates?: Array<{
          final_state?: {
            dead?: boolean;
            pos?: { x: number };
            speed?: { x: number };
          };
        }>;
        error?: string;
      };
      if (!result.candidates?.length)
        throw new Error(
          `${technique.id}/${variant.id}/${module.id} 没有可完成路线：${result.error ?? "Fuzz 无候选"}`,
        );
      const best = result.candidates[0];
      if (best.final_state?.dead)
        throw new Error(
          `${technique.id}/${variant.id}/${module.id} 最佳路线以死亡结束`,
        );
      console.log(
        `✓ ${technique.id}/${variant.id}/${module.id}: ${result.candidates.length} routes, final X=${best.final_state?.pos?.x ?? "n/a"}`,
      );
      validated += 1;
    }
    if (
      variant.training.finish.trigger.bounds.width <= 0 ||
      variant.training.finish.trigger.bounds.height <= 0
    )
      throw new Error(
        `${technique.id}/${variant.id} 终点 trigger 尺寸必须为正数`,
      );
    if (triggerIds.has(variant.training.finish.trigger.id))
      throw new Error(
        `${technique.id}/${variant.id} 终点 trigger id 与模块重复`,
      );
  }
}
console.log(
  `Validated ${validated} map-owned training modules with celeste-wasm`,
);
