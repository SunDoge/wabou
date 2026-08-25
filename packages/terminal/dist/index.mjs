import { createElement, effect, ref, setProp } from "@wabou/core/renderer";
//#region src/index.tsx
/** Typed Solid wrapper around the Rust `terminal` widget. */
function Terminal(props) {
	var _el$ = createElement("terminal");
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
			l: props.selectionBackground,
			u: props.selectionForeground,
			c: props.inheritTheme ? "true" : void 0,
			w: props.allowClipboardRead ? "true" : void 0,
			m: props.syncWindowTitle ? "true" : void 0,
			f: props.onTerminalExit,
			y: props.onTerminalProgress,
			g: props.onTerminalNotification,
			p: props.onTerminalTitleChange,
			b: props.onTerminalCwdChange,
			T: props.onTerminalSelectionChange,
			A: props.onTerminalBell
		};
	}, ({ e, t, a, o, i, n, s, h, r, d, l, u, c, w, m, f, y, g, p, b, T, A }, _p$) => {
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
		l !== _p$?.l && setProp(_el$, "selection-background", l, _p$?.l);
		u !== _p$?.u && setProp(_el$, "selection-foreground", u, _p$?.u);
		c !== _p$?.c && setProp(_el$, "inherit-theme", c, _p$?.c);
		w !== _p$?.w && setProp(_el$, "allow-clipboard-read", w, _p$?.w);
		m !== _p$?.m && setProp(_el$, "sync-window-title", m, _p$?.m);
		f !== _p$?.f && setProp(_el$, "onTerminalExit", f, _p$?.f);
		y !== _p$?.y && setProp(_el$, "onTerminalProgress", y, _p$?.y);
		g !== _p$?.g && setProp(_el$, "onTerminalNotification", g, _p$?.g);
		p !== _p$?.p && setProp(_el$, "onTerminalTitleChange", p, _p$?.p);
		b !== _p$?.b && setProp(_el$, "onTerminalCwdChange", b, _p$?.b);
		T !== _p$?.T && setProp(_el$, "onTerminalSelectionChange", T, _p$?.T);
		A !== _p$?.A && setProp(_el$, "onTerminalBell", A, _p$?.A);
	});
	return _el$;
}
//#endregion
export { Terminal };

//# sourceMappingURL=index.mjs.map