import { Image, Text, View } from "../primitives/view.mjs";
import { Center } from "../primitives/layout.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/avatar.tsx
/** Derive at most two stable initials without depending on Intl. */
function avatarInitials(name) {
	const words = name.trim().split(/\s+/).filter(Boolean);
	if (words.length === 0) return "";
	return (words.length === 1 ? [...words[0]].slice(0, 2).join("") : words.slice(0, 2).map((word) => [...word][0] ?? "").join("")).toUpperCase();
}
function Avatar(props) {
	const forwarded = omit(props, "image", "name", "alt", "fallback", "size", "class");
	const size = () => match(props.size ?? "default").with("sm", () => "w-8 h-8 text-xs").with("default", () => "w-10 h-10 text-sm").with("lg", () => "w-12 h-12 text-base").exhaustive();
	const fallback = () => props.fallback ?? avatarInitials(props.name ?? "");
	const label = () => props.alt ?? props.name ?? props.fallback ?? "Avatar";
	return createComponent$1(Center, mergeProps(forwarded, {
		role: "img",
		get "aria-label"() {
			return label();
		},
		get "class"() {
			return mergeClasses("flex-none overflow-hidden rounded-full bg-control border border-subtle", size(), props.class);
		},
		get children() {
			return memo(() => {
				return !!props.image;
			})() ? createComponent$1(Image, {
				"aria-hidden": "true",
				get resource() {
					return props.image;
				},
				class: "w-full h-full"
			}) : createComponent$1(Text, {
				"aria-hidden": "true",
				class: "font-medium text-secondary",
				get children() {
					return fallback();
				}
			});
		}
	}));
}
function AvatarGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "group";
		},
		get "class"() {
			return mergeClasses("flex items-center gap-1", props.class);
		}
	}));
}
function AvatarGroupCount(props) {
	const forwarded = omit(props, "class", "children");
	return createComponent$1(Center, mergeProps(forwarded, {
		get "class"() {
			return mergeClasses("w-10 h-10 flex-none rounded-full bg-control border border-subtle", props.class);
		},
		get children() {
			return createComponent$1(Text, {
				class: "text-xs font-medium text-muted",
				get children() {
					return props.children;
				}
			});
		}
	}));
}
//#endregion
export { Avatar, AvatarGroup, AvatarGroupCount, avatarInitials };

//# sourceMappingURL=avatar.mjs.map