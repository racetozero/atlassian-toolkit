// Deno loads the same Node-API addon through its npm/Node compatibility layer.
import { createRequire } from "node:module"
const require = createRequire(import.meta.url)
const { markup, tasks } = require("../dist/index.js")

Deno.test("the addon loads in Deno", () => {
  if (markup.markdownToWiki("**hi**") !== "*hi*") throw new Error("markup failed")
  if (!tasks().some((t: { name: string }) => t.name === "jira_issue_list"))
    throw new Error("tasks missing")
})
