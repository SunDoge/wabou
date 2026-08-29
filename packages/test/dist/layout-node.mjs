import { assertNoLayoutDiagnostics, parseLayoutSnapshot, siblingCollisionDiagnostics, styleDiagnostics, textCollisionDiagnostics, visibleOverflowDiagnostics, visualQualityDiagnostics } from "./layout.mjs";
import { spawn } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
//#region src/layout-node.ts
function layoutCommandArgs(options) {
	const args = options.withHost ? [
		"render",
		options.app,
		"--out",
		`${options.out}.png`,
		"--snapshot",
		options.out,
		"--with-host"
	] : [
		"layout",
		options.app,
		"--out",
		options.out
	];
	if (options.withHost && options.batch !== void 0) throw new Error("host-backed layout does not support batch fixtures");
	if (options.batch !== void 0) args.push("--batch", options.batch);
	if (options.width !== void 0) args.push("--width", String(options.width));
	if (options.height !== void 0) args.push("--height", String(options.height));
	if (options.scaleFactor !== void 0) args.push("--scale-factor", String(options.scaleFactor));
	if (options.mode !== void 0) args.push("--mode", options.mode);
	if (options.skipBuild) args.push("--skip-build");
	if (options.waitMs !== void 0) args.push("--wait-ms", String(options.waitMs));
	return args;
}
/** Return the first Solid runtime diagnostic that makes a layout run invalid. */
function reactiveRuntimeDiagnostic(output) {
	return output.split(/\r?\n/).find((line) => line.includes("[STRICT_READ_UNTRACKED]") || line.includes("[REACTIVITY_HALTED]"))?.trim();
}
function parseLayoutFixtureReport(value) {
	if (typeof value !== "object" || value === null) throw new Error("Wabou layout fixture report must be an object");
	const raw = value;
	if (raw.version !== 1 || !Number.isFinite(raw.totalDurationMs) || raw.totalDurationMs < 0 || !Array.isArray(raw.cases)) throw new Error("invalid Wabou layout fixture report");
	return {
		version: 1,
		totalDurationMs: raw.totalDurationMs,
		cases: raw.cases.map((entry, index) => {
			if (typeof entry !== "object" || entry === null || typeof entry.id !== "string" || !Number.isFinite(entry.durationMs) || entry.durationMs < 0 || !("snapshot" in entry)) throw new Error(`invalid Wabou layout fixture result at index ${index}`);
			return {
				id: entry.id,
				durationMs: entry.durationMs,
				snapshot: parseLayoutSnapshot(entry.snapshot)
			};
		})
	};
}
async function validateLayoutFixtureReport(report, fixtures) {
	for (const result of report.cases) {
		const fixture = fixtures.find((entry) => entry.id === result.id);
		if (!fixture) throw new Error(`unexpected Wabou layout fixture result \`${result.id}\``);
		try {
			if (!fixture.allowStyleDiagnostics) assertNoLayoutDiagnostics(styleDiagnostics(result.snapshot));
			for (const check of fixture.checks ?? []) assertNoLayoutDiagnostics(check === "visible-overflow" ? visibleOverflowDiagnostics(result.snapshot) : check === "sibling-collision" ? siblingCollisionDiagnostics(result.snapshot) : check === "text-collision" ? textCollisionDiagnostics(result.snapshot) : visualQualityDiagnostics(result.snapshot));
			await fixture.assert?.(result.snapshot);
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			throw new Error(`layout fixture \`${result.id}\` failed: ${message}`, { cause: error });
		}
	}
}
/** Build one fixture bundle and evaluate every case in one QuickJS runtime. */
async function renderLayoutFixtures(options) {
	if (options.cases !== "all" && options.cases.length === 0) throw new Error("layout fixture batch must contain at least one case");
	const directory = await mkdtemp(join(tmpdir(), "wabou-layout-"));
	const manifest = join(directory, "manifest.json");
	const out = join(directory, "report.json");
	try {
		await writeFile(manifest, JSON.stringify(options.cases === "all" ? {
			version: 1,
			all: true
		} : {
			version: 1,
			cases: options.cases.map(({ id, width, height, scaleFactor, waitMs }) => ({
				id,
				width,
				height,
				scaleFactor,
				waitMs
			}))
		}), "utf8");
		await runLayoutCommand({
			app: options.app,
			out,
			batch: manifest,
			mode: options.mode,
			skipBuild: options.skipBuild,
			waitMs: options.waitMs,
			command: options.command
		});
		const report = parseLayoutFixtureReport(JSON.parse(await readFile(out, "utf8")));
		const fixtures = options.cases === "all" ? report.cases.map(({ id }) => ({
			id,
			checks: options.checks,
			...options.overrides?.[id]
		})) : options.cases;
		if (options.cases === "all" && options.overrides) {
			const discovered = new Set(report.cases.map(({ id }) => id));
			const unknown = Object.keys(options.overrides).filter((id) => !discovered.has(id));
			if (unknown.length > 0) throw new Error(`layout fixture overrides reference unknown fixtures: ${unknown.join(", ")}`);
		}
		await validateLayoutFixtureReport(report, fixtures);
		return report;
	} finally {
		await rm(directory, {
			recursive: true,
			force: true
		});
	}
}
async function renderAppLayout(options) {
	await runLayoutCommand(options);
	return parseLayoutSnapshot(JSON.parse(await readFile(options.out, "utf8")));
}
async function runLayoutCommand(options) {
	const command = options.command ?? ["wabou"];
	if (command.length === 0) throw new Error("layout command must not be empty");
	await new Promise((resolve, reject) => {
		let diagnostics = "";
		const child = spawn(command[0], [...command.slice(1), ...layoutCommandArgs(options)], {
			env: {
				...process.env,
				NODE_ENV: "development"
			},
			stdio: [
				"ignore",
				"pipe",
				"pipe"
			]
		});
		child.stdout.on("data", (chunk) => {
			const text = chunk.toString();
			diagnostics += text;
			process.stdout.write(text);
		});
		child.stderr.on("data", (chunk) => {
			const text = chunk.toString();
			diagnostics += text;
			process.stderr.write(text);
		});
		child.once("error", reject);
		child.once("exit", (code, signal) => {
			if (code !== 0) {
				reject(/* @__PURE__ */ new Error(`layout command failed ${signal ? `with signal ${signal}` : `with exit status ${code}`}`));
				return;
			}
			const reactiveDiagnostic = reactiveRuntimeDiagnostic(diagnostics);
			if (reactiveDiagnostic) {
				reject(/* @__PURE__ */ new Error(`layout command emitted a reactive runtime diagnostic: ${reactiveDiagnostic.trim()}`));
				return;
			}
			resolve();
		});
	});
}
//#endregion
export { layoutCommandArgs, parseLayoutFixtureReport, reactiveRuntimeDiagnostic, renderAppLayout, renderLayoutFixtures, validateLayoutFixtureReport };

//# sourceMappingURL=layout-node.mjs.map