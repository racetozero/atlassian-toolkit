#!/usr/bin/env node
// Size budget for the published WebAssembly package. Every Atlassian API
// catalog is compiled in, so the module is large; this catches growth.
import { strict as assert } from "node:assert"
import { readFile } from "node:fs/promises"
import { gzipSync } from "node:zlib"

const wasm = await readFile(new URL("../pkg/atl_wasm_bg.wasm", import.meta.url))
const loader = await readFile(new URL("../pkg/atl_wasm.js", import.meta.url))
const gzip = gzipSync(wasm, { level: 9 })

assert.ok(wasm.length <= 9_000_000, `WASM exceeds 9000000-byte budget: ${wasm.length}`)
assert.ok(gzip.length <= 1_700_000, `gzip WASM exceeds 1700000-byte budget: ${gzip.length}`)
assert.ok(loader.length <= 60_000, `JS loader exceeds 60000-byte budget: ${loader.length}`)
console.log(`atl-wasm size passed: raw=${wasm.length} gzip=${gzip.length} loader=${loader.length}`)
