// The ES module entry point: named imports work, and they are the same
// objects as the CommonJS ones, so `instanceof` agrees across both.
import assert from "node:assert/strict"
import { createRequire } from "node:module"
import test from "node:test"
import sdk, { AtlError, Client, markup, tasks } from "atlassian-toolkit"

const require = createRequire(import.meta.url)

test("named ESM imports are the CommonJS exports", () => {
  const cjs = require("atlassian-toolkit")
  assert.equal(Client, cjs.Client)
  assert.equal(AtlError, cjs.AtlError)
  assert.equal(sdk.Client, Client)
  assert.equal(markup.markdownToWiki("**hi**"), "*hi*")
  assert.ok(tasks().length > 100)
})

test("errors from an ESM import are AtlError instances", async () => {
  const client = Client.withConnections({})
  await assert.rejects(client.jira.issue.view({ key: "A-1" }), (err) => {
    assert.ok(err instanceof AtlError)
    assert.equal(err.code, "auth")
    return true
  })
})

test("the build keeps __dirname a run-time value", async () => {
  // A bundler can inline the build machine's directory, and then the addon
  // loader looks for native/ on that path on every user's machine.
  const { readFile } = await import("node:fs/promises")
  for (const file of ["index.js", "common.js", "tasks.js"]) {
    const text = await readFile(new URL(`../dist/${file}`, import.meta.url), "utf8")
    assert.doesNotMatch(text, /__dirname\s*=\s*["'`]/, file)
  }
})
