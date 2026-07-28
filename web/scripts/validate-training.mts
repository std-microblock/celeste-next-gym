import { readFile } from 'node:fs/promises'
import { decode, encode } from '@msgpack/msgpack'
import init, { cache_simulation_map_msgpack, fuzz_search_cached_map_msgpack } from '../src/wasm/celeste_wasm.js'
import { trainingCatalog } from '../src/training/catalog.ts'
import { trainingEntryInput } from '../src/training/session.ts'

await init({ module_or_path: await readFile(new URL('../src/wasm/celeste_wasm_bg.wasm', import.meta.url)) })

let validated = 0
for (const technique of trainingCatalog) {
  for (const variant of technique.variants) {
    const entry = trainingEntryInput(variant.document)
    if (!entry) throw new Error(`${technique.id}/${variant.id} entry.input_id 未指向可验证输入`)
    if (entry.at !== 0) throw new Error(`${technique.id}/${variant.id} 入口输入不在本地 F0`)
    cache_simulation_map_msgpack(encode(variant.map))
    const result = decode(fuzz_search_cached_map_msgpack(encode(variant.initial), JSON.stringify(variant.validationFuzz ?? variant.document.fuzz))) as {
      candidates?: Array<{ final_state?: { dead?: boolean; pos?: { x: number }; speed?: { x: number } } }>
      error?: string
    }
    if (!result.candidates?.length) throw new Error(`${technique.id}/${variant.id} 没有可完成路线：${result.error ?? 'Fuzz 无候选'}`)
    const best = result.candidates[0]
    if (best.final_state?.dead) throw new Error(`${technique.id}/${variant.id} 最佳路线以死亡结束`)
    console.log(`✓ ${technique.id}/${variant.id}: ${result.candidates.length} routes, final X=${best.final_state?.pos?.x ?? 'n/a'}`)
    validated += 1
  }
}
console.log(`Validated ${validated} training variants with celeste-wasm`)
