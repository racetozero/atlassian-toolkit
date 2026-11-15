// Deno loads the same Node-API addon through its npm/Node compatibility
// layer, through the ES module entry point.
import { Client, markup, tasks } from "../dist/index.mjs"

Deno.test("the addon loads in Deno", () => {
  if (markup.markdownToWiki("**hi**") !== "*hi*") throw new Error("markup failed")
  if (!tasks().some((t: { name: string }) => t.name === "jira_issue_list"))
    throw new Error("tasks missing")
  const client = Client.withConnections({})
  if (typeof client.jira.issue.list !== "function") throw new Error("namespaces missing")
})
