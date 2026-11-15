// Writes dist/index.mjs and dist/index.d.mts: an ES module entry point that
// re-exports the CommonJS build (tsc output; a bundler would inline the
// build machine's __dirname, which the addon loader needs at run time). One copy of the code runs for both `import`
// and `require`, so `instanceof AtlError` works whichever one the caller uses.
import { writeFileSync } from "node:fs"
import { createRequire } from "node:module"

const require = createRequire(import.meta.url)
const names = Object.keys(require("../dist/index.js")).filter((name) => name !== "__esModule")
if (!names.includes("Client")) {
  throw new Error(
    `dist/index.js exports ${names.join(", ")}; build it with \`bun run build:ts\` first`,
  )
}
writeFileSync(
  "dist/index.mjs",
  `import cjs from "./index.js"\n\nexport const { ${names.join(", ")} } = cjs\nexport default cjs\n`,
)
writeFileSync("dist/index.d.mts", `export * from "./index.js"\n`)
