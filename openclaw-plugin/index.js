const DEFAULT_BASE_URL = "http://127.0.0.1:8000";
const DEFAULT_NAMESPACE = "default";
function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function getPluginConfig(api) {
  const pluginConfig = asObject(api.pluginConfig);
  return {
    baseUrl: typeof pluginConfig.baseUrl === "string" && pluginConfig.baseUrl.trim() ? pluginConfig.baseUrl.trim().replace(/\/$/, "") : DEFAULT_BASE_URL,
    namespace: typeof pluginConfig.namespace === "string" && pluginConfig.namespace.trim() ? pluginConfig.namespace.trim() : DEFAULT_NAMESPACE,
    enabledAgents: Array.isArray(pluginConfig.enabledAgents) ? pluginConfig.enabledAgents.filter((value) => typeof value === "string" && value.trim()).map((value) => value.trim()) : []
  };
}

function shouldHandleAgent(config, ctx) {
  if (!ctx?.agentId) return true;
  if (config.enabledAgents.length === 0) return true;
  return config.enabledAgents.includes(ctx.agentId);
}

function isLikelySqlitePrompt(prompt) {
  const lowered = String(prompt ?? "").toLowerCase();
  return lowered.includes("sqlite") || lowered.includes(".db") || lowered.includes("schema") || lowered.includes("query") || lowered.includes("sql");
}

function extractPromptResource(prompt) {
  const text = String(prompt ?? "");
  const dbMatch = text.match(/([~/.A-Za-z0-9_-]+\.db)\b/);
  if (dbMatch) return `sqlite:${dbMatch[1]}`;

  const lowered = text.toLowerCase();
  if ((lowered.includes("btc_directional") || lowered.includes("btc-directional")) && lowered.includes("dry")) {
    return "sqlite:~/drachma/bot_btc_directional_dry.db";
  }
  if ((lowered.includes("btc_directional") || lowered.includes("btc-directional")) && lowered.includes("prod")) {
    return "sqlite:~/drachma/bot_btc_directional_prod.db";
  }

  return null;
}

async function postJson(baseUrl, path, payload, logger) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 1500);
  try {
    const response = await fetch(`${baseUrl}${path}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(payload),
      signal: controller.signal
    });
    if (!response.ok) {
      const body = await response.text().catch(() => "");
      throw new Error(`HTTP ${response.status}${body ? `: ${body}` : ""}`);
    }
    return await response.json();
  } catch (error) {
    logger?.warn?.(`[iterum-auto] ${path} failed: ${error instanceof Error ? error.message : String(error)}`);
    return null;
  } finally {
    clearTimeout(timeout);
  }
}

function getBashCommand(params) {
  if (typeof params?.command === "string") return params.command;
  if (typeof params?.cmd === "string") return params.cmd;
  return null;
}

function isSqliteShellCommand(command) {
  return /\bsqlite3\b/.test(command);
}

function isSqliteIntrospectionCommand(command) {
  return /\.(?:tables|schema)\b/.test(command) || /\bPRAGMA\s+table_info\b/i.test(command);
}

function isExecLikeTool(toolName) {
  return toolName === "exec" || toolName === "bash";
}

function tokenizeShell(command) {
  return String(command).match(/"[^"]*"|'[^']*'|\S+/g) ?? [];
}

function extractDirectSqliteDbToken(command) {
  const tokens = tokenizeShell(command);
  const sqliteIndex = tokens.findIndex((token) => token === "sqlite3" || token.endsWith("/sqlite3"));
  if (sqliteIndex === -1) return null;
  for (let index = sqliteIndex + 1; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (!token) continue;
    if (token.startsWith("-")) {
      if (token === "-cmd" || token === "-separator" || token === "-newline" || token === "-nullvalue") {
        index += 1;
      }
      continue;
    }
    return token;
  }
  return null;
}

function normalizeDbToken(token) {
  return token?.replace(/^['"]|['"]$/g, "") ?? null;
}

function parseSqliteCommand(command) {
  const directToken = extractDirectSqliteDbToken(command);
  if (directToken) {
    return {
      mode: "direct",
      dbToken: directToken,
      normalizedDbToken: normalizeDbToken(directToken)
    };
  }

  const sshMatch = String(command).match(/^(.*?\bssh\b(?:\s+-\S+|\s+\S+)+)\s+(["'])(sqlite3[\s\S]*)\2$/);
  if (!sshMatch) return null;

  const sshPrefix = sshMatch[1];
  const quote = sshMatch[2];
  const innerCommand = sshMatch[3];
  const innerDbToken = extractDirectSqliteDbToken(innerCommand);
  if (!innerDbToken) return null;

  return {
    mode: "ssh",
    sshPrefix,
    quote,
    innerCommand,
    dbToken: innerDbToken,
    normalizedDbToken: normalizeDbToken(innerDbToken)
  };
}

function resourceIdForSqlite(command) {
  const parsed = parseSqliteCommand(command);
  if (!parsed?.normalizedDbToken) return null;
  return `sqlite:${parsed.normalizedDbToken}`;
}


function extractText(value) {
  if (value == null) return "";
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.map(extractText).filter(Boolean).join("\n");
  if (typeof value === "object") {
    if (typeof value.text === "string") return value.text;
    if (typeof value.stdout === "string" || typeof value.stderr === "string") {
      return [value.stdout, value.stderr].filter(Boolean).join("\n");
    }
    if (Array.isArray(value.content)) return extractText(value.content);
    return Object.values(value).map(extractText).filter(Boolean).join("\n");
  }
  return String(value);
}

function extractSchemaTable(command) {
  const match = String(command).match(/\.schema\s+([A-Za-z_][A-Za-z0-9_]*)/);
  if (match) return match[1];

  const pragmaMatch = String(command).match(/\bPRAGMA\s+table_info\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)/i);
  return pragmaMatch ? pragmaMatch[1] : null;
}

function buildSilentGuidance(resourceId, promptContext) {
  const parts = [
    "Internal note for tool use: do not mention skills, Iterum, memory retrieval, prompt injection, or whether prior context exists unless the user explicitly asks.",
    "Use any remembered context silently to improve investigation quality."
  ];
  if (resourceId) {
    parts.push(`Target resource: ${resourceId}.`);
  }
  if (promptContext) {
    parts.push(`Remembered operational context:\n${promptContext}`);
  }
  return parts.join("\n\n");
}

export default {
  id: "iterum-auto",
  name: "Iterum Auto",
  description: "Automatic Iterum prompt injection and sqlite fact learning hooks",
  configSchema: {
    type: "object",
    additionalProperties: false,
    properties: {
      baseUrl: { type: "string" },
      namespace: { type: "string" },
      enabledAgents: { type: "array", items: { type: "string" } }
    }
  },
  register(api) {
    api.on("before_prompt_build", async (event, ctx) => {
      const config = getPluginConfig(api);
      if (!shouldHandleAgent(config, ctx)) return;
      if (!isLikelySqlitePrompt(event.prompt)) return;

      const resourceId = extractPromptResource(event.prompt);
      let promptContext = "";

      if (resourceId) {
        const retrieved = await postJson(config.baseUrl, "/v1/context/retrieve", {
          namespace: config.namespace,
          tool_name: "sqlite3",
          resource_id: resourceId,
          task_type: "analysis",
          query: event.prompt
        }, api.logger);
        promptContext = typeof retrieved?.prompt_context === "string" ? retrieved.prompt_context.trim() : "";
      }

      return { prependContext: buildSilentGuidance(resourceId, promptContext) };
    }, { name: "iterum-auto-before-prompt" });

    api.on("after_tool_call", async (event, ctx) => {
      const config = getPluginConfig(api);
      if (!shouldHandleAgent(config, ctx)) return;
      if (!isExecLikeTool(event.toolName)) return;

      const command = getBashCommand(event.params);
      if (!command || !isSqliteShellCommand(command)) return;

      const resourceId = resourceIdForSqlite(command);
      if (!resourceId) return;

      const text = extractText(event.result).trim();
      if (!text) return;

      if (/\b\.tables\b/.test(command)) {
        await postJson(config.baseUrl, "/v1/context/facts", {
          namespace: config.namespace,
          tool_name: "sqlite3",
          resource_id: resourceId,
          topic: "tables",
          title: `Known tables for ${resourceId}`,
          content: text,
          confidence: 0.95
        }, api.logger);
      }

      const schemaTable = extractSchemaTable(command);
      if (schemaTable) {
        await postJson(config.baseUrl, "/v1/context/facts", {
          namespace: config.namespace,
          tool_name: "sqlite3",
          resource_id: resourceId,
          topic: `schema:${schemaTable}`,
          title: `Schema for ${schemaTable}`,
          content: text,
          confidence: 0.95
        }, api.logger);
      }

      if (/no such column|unknown column/i.test(text)) {
        await postJson(config.baseUrl, "/v1/context/observations", {
          namespace: config.namespace,
          tool_name: "sqlite3",
          resource_id: resourceId,
          topic: "query_failure",
          content: text,
          confidence: 0.8,
          ttl_seconds: 86400
        }, api.logger);
      }
    }, { name: "iterum-auto-after-tool" });
  }
};
