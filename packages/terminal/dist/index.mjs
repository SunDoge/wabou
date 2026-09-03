import { createElement, effect, ref, setProp } from "@wabou/core/renderer";
//#region src/index.tsx
/** Typed Solid wrapper around the Rust `terminal` widget. */
function Terminal(props) {
	var _el$ = createElement("terminal", {
		projectionBoundary: true,
		role: "textbox"
	});
	var _ref$ = props.ref;
	typeof _ref$ === "function" || Array.isArray(_ref$) ? ref(() => {
		return _ref$;
	}, _el$) : props.ref = _el$;
	effect(() => {
		return {
			e: props.class,
			t: props.style,
			a: props.focusOrder ?? 0,
			o: props["aria-label"],
			i: props.command,
			n: props.args ? JSON.stringify(props.args) : void 0,
			s: props.cwd,
			h: props.fontFamily,
			r: props.fontSize,
			d: props.lineHeight,
			l: props.cursorBlink === void 0 ? void 0 : String(props.cursorBlink),
			u: props.selectionBackground,
			c: props.selectionForeground,
			w: props.inheritTheme ? "true" : void 0,
			m: props.allowClipboardRead ? "true" : void 0,
			f: props.syncWindowTitle ? "true" : void 0,
			y: props.onTerminalExit,
			g: props.onTerminalProgress,
			p: props.onTerminalNotification,
			b: props.onTerminalTitleChange,
			T: props.onTerminalCwdChange,
			A: props.onTerminalSelectionChange,
			O: props.onTerminalBell
		};
	}, ({ e, t, a, o, i, n, s, h, r, d, l, u, c, w, m, f, y, g, p, b, T, A, O }, _p$) => {
		e !== _p$?.e && setProp(_el$, "class", e, _p$?.e);
		t !== _p$?.t && setProp(_el$, "style", t, _p$?.t);
		a !== _p$?.a && setProp(_el$, "focusOrder", a, _p$?.a);
		o !== _p$?.o && setProp(_el$, "aria-label", o, _p$?.o);
		i !== _p$?.i && setProp(_el$, "command", i, _p$?.i);
		n !== _p$?.n && setProp(_el$, "args", n, _p$?.n);
		s !== _p$?.s && setProp(_el$, "cwd", s, _p$?.s);
		h !== _p$?.h && setProp(_el$, "font-family", h, _p$?.h);
		r !== _p$?.r && setProp(_el$, "font-size", r, _p$?.r);
		d !== _p$?.d && setProp(_el$, "line-height", d, _p$?.d);
		l !== _p$?.l && setProp(_el$, "cursor-blink", l, _p$?.l);
		u !== _p$?.u && setProp(_el$, "selection-background", u, _p$?.u);
		c !== _p$?.c && setProp(_el$, "selection-foreground", c, _p$?.c);
		w !== _p$?.w && setProp(_el$, "inherit-theme", w, _p$?.w);
		m !== _p$?.m && setProp(_el$, "allow-clipboard-read", m, _p$?.m);
		f !== _p$?.f && setProp(_el$, "sync-window-title", f, _p$?.f);
		y !== _p$?.y && setProp(_el$, "onTerminalExit", y, _p$?.y);
		g !== _p$?.g && setProp(_el$, "onTerminalProgress", g, _p$?.g);
		p !== _p$?.p && setProp(_el$, "onTerminalNotification", p, _p$?.p);
		b !== _p$?.b && setProp(_el$, "onTerminalTitleChange", b, _p$?.b);
		T !== _p$?.T && setProp(_el$, "onTerminalCwdChange", T, _p$?.T);
		A !== _p$?.A && setProp(_el$, "onTerminalSelectionChange", A, _p$?.A);
		O !== _p$?.O && setProp(_el$, "onTerminalBell", O, _p$?.O);
	});
	return _el$;
}
//#endregion
export { Terminal };

//# sourceMappingURL=index.mjs.map