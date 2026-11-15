// Prepares the npm release: one package per native platform in npm/<platform>/
// (atlassian-toolkit-<platform>, with os/cpu/libc so npm installs only the
// matching one) and package.json pointing at them as optionalDependencies.
// The release workflow runs this after `bun run build:ts`, then publishes each
// npm/<platform> directory and this one.
//
//   bun scripts/platform-packages.mjs [--platforms darwin-arm64,linux-x64]
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { join } from "node:path"

const ALL = ["linux-x64", "linux-arm64", "darwin-x64", "darwin-arm64", "win32-x64", "win32-arm64"]
const flag = process.argv.indexOf("--platforms")
const platforms = flag === -1 ? ALL : process.argv[flag + 1].split(",")
for (const platform of platforms) {
  if (!ALL.includes(platform)) {
    throw new Error(`--platforms ${platform}: use any of ${ALL.join(", ")}, separated by commas`)
  }
  if (!existsSync(join("native", platform, "atl.node"))) {
    throw new Error(
      `native/${platform}/atl.node is missing; build it with \`bun run build:native\` on that platform (the release workflow downloads every build-node-sdk artifact into native/)`,
    )
  }
}

const main = JSON.parse(readFileSync("package.json", "utf8"))
const optional = {}
for (const platform of platforms) {
  const [os, cpu] = platform.split("-")
  const name = `${main.name}-${platform}`
  const dir = join("npm", platform)
  mkdirSync(dir, { recursive: true })
  copyFileSync(join("native", platform, "atl.node"), join(dir, "atl.node"))
  if (existsSync("LICENSE")) copyFileSync("LICENSE", join(dir, "LICENSE"))
  writeFileSync(
    join(dir, "README.md"),
    `# ${name}\n\nThe ${platform} native addon of [${main.name}](https://www.npmjs.com/package/${main.name}). Install ${main.name}; npm adds this package on ${platform}.\n`,
  )
  const pkg = {
    name,
    version: main.version,
    description: `${platform} native addon of ${main.name}`,
    homepage: main.homepage,
    license: main.license,
    repository: main.repository,
    os: [os],
    cpu: [cpu],
    ...(os === "linux" ? { libc: ["glibc"] } : {}),
    main: "atl.node",
    files: ["atl.node", "README.md", "LICENSE"],
    engines: main.engines,
  }
  writeFileSync(join(dir, "package.json"), `${JSON.stringify(pkg, null, 2)}\n`)
  optional[name] = main.version
}
main.optionalDependencies = optional
main.files = main.files.filter((entry) => entry !== "native")
writeFileSync("package.json", `${JSON.stringify(main, null, 2)}\n`)
console.log(`prepared ${platforms.map((p) => `${main.name}-${p}`).join(", ")}`)
