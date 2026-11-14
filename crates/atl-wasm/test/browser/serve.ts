// Serves the smoke page, the wasm package, and a mock Jira from one origin.
const root = new URL("../../", import.meta.url)
const types: Record<string, string> = {
  html: "text/html",
  js: "text/javascript",
  wasm: "application/wasm",
}
Deno.serve({ port: Number(Deno.args[0] ?? 8765), hostname: "127.0.0.1" }, async (req) => {
  const { pathname } = new URL(req.url)
  if (pathname === "/rest/api/3/search/jql") {
    return Response.json({
      issues: [
        {
          key: "WEB-7",
          fields: {
            summary: "s",
            status: { name: "To Do", statusCategory: { key: "new" } },
            issuetype: { name: "Bug" },
          },
        },
      ],
    })
  }
  const file = pathname === "/" ? "test/browser/index.html" : pathname.slice(1)
  try {
    const body = await Deno.readFile(new URL(file, root))
    return new Response(body, {
      headers: { "content-type": types[file.split(".").pop() ?? ""] ?? "application/octet-stream" },
    })
  } catch {
    return new Response("not found", { status: 404 })
  }
})
