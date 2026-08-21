import { parseLayoutSnapshot } from "./layout.mjs";
import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
//#region src/layout-node.ts
function layoutCommandArgs(options) {
	const args = [
		"layout",
		options.app,
		"--out",
		options.out
	];
	if (options.width !== void 0) args.push("--width", String(options.width));
	if (options.height !== void 0) args.push("--height", String(options.height));
	if (options.scaleFactor !== void 0) args.push("--scale-factor", String(options.scaleFactor));
	if (options.mode !== void 0) args.push("--mode", options.mode);
	if (options.skipBuild) args.push("--skip-build");
	if (options.waitMs !== void 0) args.push("--wait-ms", String(options.waitMs));
	return args;
}
async function renderAppLayout(options) {
	const command = options.command ?? ["wabou"];
	if (command.length === 0) throw new Error("layout command must not be empty");
	await new Promise((resolve, reject) => {
		const child = spawn(command[0], [...command.slice(1), ...layoutCommandArgs(options)], { stdio: [
			"ignore",
			"inherit",
			"inherit"
		] });
		child.once("error", reject);
		child.once("exit", (code, signal) => {
			if (code === 0) resolve();
			else reject(/* @__PURE__ */ new Error(`layout command failed ${signal ? `with signal ${signal}` : `with exit status ${code}`}`));
		});
	});
	return parseLayoutSnapshot(JSON.parse(await readFile(options.out, "utf8")));
}
//#endregion
export { layoutCommandArgs, renderAppLayout };

//# sourceMappingURL=layout-node.mjs.map