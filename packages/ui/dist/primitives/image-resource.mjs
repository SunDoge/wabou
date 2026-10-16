import { useHost } from "@wabou/core";
import { createEffect, createSignal, onCleanup } from "solid-js";
//#region src/primitives/image-resource.ts
function host() {
	return useHost().imageResources;
}
/** Explicitly create a new resource from a host file. No identity deduplication occurs. */
function createFileImageResource(path) {
	return host().createFile({ path });
}
/** Explicitly create a new resource from an HTTP(S) response. */
function createNetworkImageResource(url) {
	return host().createNetwork({ url });
}
/** Deterministically release a resource. Images only borrow their handle. */
function releaseImageResource(handle) {
	return Promise.resolve(host().release(handle));
}
/**
* Create a resource owned by the current Solid owner. Source replacement and
* owner cleanup clear the borrowed handle before releasing the native resource.
*/
function createOwnedImageResource(request) {
	const [resource, setResource] = createSignal();
	const [loading, setLoading] = createSignal(false);
	const [error, setError] = createSignal();
	let owned;
	let revision = 0;
	const releaseOwned = () => {
		const previous = owned;
		owned = void 0;
		setResource(void 0);
		if (previous) releaseImageResource(previous.handle);
	};
	createEffect(request, (source) => {
		const currentRevision = ++revision;
		releaseOwned();
		setError(void 0);
		if (!source) {
			setLoading(false);
			return;
		}
		setLoading(true);
		(source.kind === "file" ? createFileImageResource(source.path) : createNetworkImageResource(source.url)).then((next) => {
			if (currentRevision !== revision) {
				releaseImageResource(next.handle);
				return;
			}
			owned = next;
			setResource(next);
			setLoading(false);
		}, (reason) => {
			if (currentRevision !== revision) return;
			setError(reason);
			setLoading(false);
		});
	});
	onCleanup(() => {
		revision++;
		releaseOwned();
	});
	return {
		resource,
		loading,
		error
	};
}
//#endregion
export { createFileImageResource, createNetworkImageResource, createOwnedImageResource, releaseImageResource };

//# sourceMappingURL=image-resource.mjs.map