import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const text = readFileSync("design-system/linguaray/handoff-manifest.md", "utf-8");
const UUID_RE = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g;
const STRICT_UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

function extractSection(label) {
  const match = text.match(new RegExp(`## ${label}\\n([\\s\\S]*?)(?=\\n## |$)`));
  assert.ok(match, `区段 "${label}" 不存在`);
  return match[1];
}

function extractUUIDs(section) {
  return [...section.matchAll(UUID_RE)].map((match) => match[0]);
}

const teamSection = extractSection("Penpot File & Team");
const teamLineIDs = [...teamSection.matchAll(/Team ID:\s*([0-9a-f-]{36})/g)].map(
  (match) => match[1],
);
const fileLineIDs = [...teamSection.matchAll(/File ID:\s*([0-9a-f-]{36})/g)].map(
  (match) => match[1],
);
assert.strictEqual(teamLineIDs.length, 1, `Team ID 行应为 1 个，实际 ${teamLineIDs.length}`);
assert.strictEqual(fileLineIDs.length, 1, `File ID 行应为 1 个，实际 ${fileLineIDs.length}`);
assert.match(teamLineIDs[0], STRICT_UUID, "Team ID 格式不合法");
assert.match(fileLineIDs[0], STRICT_UUID, "File ID 格式不合法");
assert.notStrictEqual(fileLineIDs[0], teamLineIDs[0], "File ID 不得等于 Team ID");

const pageUUIDs = extractUUIDs(extractSection("Penpot 页面"));
assert.strictEqual(pageUUIDs.length, 8, `页面 UUID 应为 8，实际 ${pageUUIDs.length}`);
assert.strictEqual(new Set(pageUUIDs).size, 8, "页面 UUID 有重复");
pageUUIDs.forEach((id) => assert.match(id, STRICT_UUID, `页面 UUID 格式不合法: ${id}`));

const surfaceUUIDs = extractUUIDs(extractSection("16 Surface"));
assert.strictEqual(surfaceUUIDs.length, 16, `Surface UUID 应为 16，实际 ${surfaceUUIDs.length}`);
assert.strictEqual(new Set(surfaceUUIDs).size, 16, "Surface UUID 有重复");
surfaceUUIDs.forEach((id) => assert.match(id, STRICT_UUID, `Surface UUID 格式不合法: ${id}`));

const componentUUIDs = extractUUIDs(extractSection("18 Component"));
assert.strictEqual(componentUUIDs.length, 18, `Component UUID 应为 18，实际 ${componentUUIDs.length}`);
assert.strictEqual(new Set(componentUUIDs).size, 18, "Component UUID 有重复");
componentUUIDs.forEach((id) => assert.match(id, STRICT_UUID, `Component UUID 格式不合法: ${id}`));

const nodeUUIDs = [...surfaceUUIDs, ...componentUUIDs];
assert.strictEqual(nodeUUIDs.length, 34);
assert.strictEqual(new Set(nodeUUIDs).size, 34, "34 个 Node ID 有全局重复");

const allUUIDs = [teamLineIDs[0], fileLineIDs[0], ...pageUUIDs, ...nodeUUIDs];
assert.strictEqual(new Set(allUUIDs).size, allUUIDs.length, "manifest 全局 ID 有重复");

assert.doesNotMatch(text, /TBD-S|TBD-C|NODE_ID_REQUIRED/gi, "存在 Node ID 占位符");

const tokenSection = extractSection("Token 集合");
assert.ok(tokenSection.includes("Core: 97"), "Core 数量必须为 97");
assert.ok(tokenSection.includes("Light: 28"), "Light 数量必须为 28");
assert.ok(tokenSection.includes("Dark: 28"), "Dark 数量必须为 28");
assert.doesNotMatch(tokenSection, /~\d+|约\d+/, "Token 数量不得用近似值");

console.log(
  `R0-3 manifest: 结构验证通过（Page ${pageUUIDs.length} / Surface ${surfaceUUIDs.length} / Component ${componentUUIDs.length} / Node ${nodeUUIDs.length}）`,
);
