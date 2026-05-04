import { type ChildProcess, spawn } from "child_process";
import { mkdirSync, writeFileSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";
import { mkdtempSync } from "fs";

export interface ServerInstance {
  process: ChildProcess;
  baseUrl: string;
  launchUrl: string;
  projectDir: string;
  cleanup: () => void;
}

export function createDemoProject(entityCount = 5): string {
  const dir = mkdtempSync(join(tmpdir(), "kn-e2e-"));
  const metaDir = join(dir, "metadata");
  mkdirSync(metaDir);

  const entities = Array.from({ length: entityCount }, (_, i) => ({
    id: `ent_${String(i)}`,
    name: `entity_${String(i)}`,
    domain: `dom_${String(i % 3)}`,
    description: `Test entity ${String(i)}`,
    attributes: [
      {
        id: `attr_${String(i)}_id`,
        name: "id",
        logical_type: "integer",
        description: "Primary key",
      },
    ],
  }));

  const relationships = Array.from(
    { length: Math.min(entityCount - 1, 10) },
    (_, i) => ({
      id: `rel_${String(i)}`,
      from_entity: `entity_${String(i)}`,
      to_entity: `entity_${String(i + 1)}`,
    }),
  );

  const domains = [
    { id: "dom_0", name: "dom_0" },
    { id: "dom_1", name: "dom_1" },
    { id: "dom_2", name: "dom_2" },
  ];

  const openQuestions = [
    {
      id: "oq_1",
      question: "Should entity_0 have a created_at field?",
      context: "Audit requirement",
      entity: "entity_0",
      priority: "high",
    },
  ];

  const yaml = `version: "1.0"
project:
  name: e2e-test
  owner: test-team
domains:
${domains.map((d) => `  - id: ${d.id}\n    name: ${d.name}`).join("\n")}
entities:
${entities
    .map(
      (e) => `  - id: ${e.id}
    name: ${e.name}
    domain: ${e.domain}
    description: "${e.description}"
    attributes:
${e.attributes
        .map(
          (a) => `      - id: ${a.id}
        name: ${a.name}
        logical_type: ${a.logical_type}
        description: "${a.description}"`,
        )
        .join("\n")}`,
    )
    .join("\n")}
relationships:
${relationships
    .map(
      (r) => `  - id: ${r.id}
    from_entity: ${r.from_entity}
    to_entity: ${r.to_entity}`,
    )
    .join("\n")}
open_questions:
${openQuestions
    .map(
      (q) => `  - id: ${q.id}
    question: "${q.question}"
    context: "${q.context}"
    entity: ${q.entity}
    priority: ${q.priority}`,
    )
    .join("\n")}
`;

  writeFileSync(join(metaDir, "project.yml"), yaml);
  writeFileSync(join(dir, "know-now.yml"), "version: '1.0'\n");

  return dir;
}

export async function startServer(
  projectDir: string,
  port = 0,
): Promise<ServerInstance> {
  return new Promise((resolve, reject) => {
    const proc = spawn(
      "cargo",
      [
        "run",
        "--bin",
        "know-now",
        "--",
        "--project",
        projectDir,
        "serve",
        "--port",
        String(port),
      ],
      {
        stdio: ["pipe", "pipe", "pipe"],
        env: { ...process.env },
      },
    );

    let stderr = "";
    const timeout = setTimeout(() => {
      proc.kill();
      reject(new Error(`Server start timeout. stderr: ${stderr}`));
    }, 60_000);

    proc.stderr?.on("data", (data: Buffer) => {
      stderr += data.toString();
      const match = /Listening on (http:\/\/[^\s]+)/.exec(stderr);
      if (match?.[1]) {
        clearTimeout(timeout);
        const baseUrl = match[1];
        const launchMatch = /Launch URL: (http:\/\/[^\s]+)/.exec(stderr);
        const launchUrl = launchMatch?.[1] ?? `${baseUrl}/__open`;
        resolve({
          process: proc,
          baseUrl,
          launchUrl,
          projectDir,
          cleanup: () => {
            proc.kill("SIGTERM");
          },
        });
      }
    });

    proc.on("error", (err) => {
      clearTimeout(timeout);
      reject(err);
    });

    proc.on("exit", (code) => {
      clearTimeout(timeout);
      if (code !== null && code !== 0) {
        reject(new Error(`Server exited with code ${String(code)}. stderr: ${stderr}`));
      }
    });
  });
}

export function createXssPayloads(): Record<string, string> {
  return {
    script: '<script>alert("xss")</script>',
    imgOnError: '<img src=x onerror="alert(1)">',
    jsUrl: "javascript:alert(1)",
    svgOnload: '<svg onload="alert(1)">',
    eventHandler: '" onfocus="alert(1)" autofocus="',
  };
}
