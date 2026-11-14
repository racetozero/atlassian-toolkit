// Copies the cargo-built addon to native/<platform>-<arch>/atl.node.
import { copyFileSync, existsSync, mkdirSync } from "node:fs"
import { join } from "node:path"

const name =
  { win32: "atl_node.dll", darwin: "libatl_node.dylib" }[process.platform] ?? "libatl_node.so"
const source = join("target", "release", name)
if (!existsSync(source)) {
  throw new Error(`${source} is missing; run \`cargo build --release\` in crates/atl-node first`)
}
// The release workflow names each runner's platform (linux-x64, darwin-arm64, ...).
const destination = join(
  "native",
  process.env.ATL_NODE_PLATFORM ?? `${process.platform}-${process.arch}`,
)
mkdirSync(destination, { recursive: true })
copyFileSync(source, join(destination, "atl.node"))
