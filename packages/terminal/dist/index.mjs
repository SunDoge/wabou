import { createElement, effect, ref, setProp } from "@wabou/solid-renderer";
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
			a: props.command,
			o: props.args ? JSON.stringify(props.args) : void 0,
			i: props.cwd,
			n: props.fontFamily,
			s: props.fontSize,
			h: props.lineHeight,
			r: props.selectionBackground,
			d: props.selectionForeground,
			l: props.inheritTheme ? "true" : void 0,
			u: props.allowClipboardRead ? "true" : void 0,
			c: props.syncWindowTitle ? "true" : void 0,
			w: props.onTerminalExit,
			m: props.onTerminalProgress,
			f: props.onTerminalNotification,
			y: props.onTerminalTitleChange,
			g: props.onTerminalCwdChange,
			p: props.onTerminalSelectionChange,
			b: props.onTerminalBell
		};
	}, ({ e, t, a, o, i, n, s, h, r, d, l, u, c, w, m, f, y, g, p, b }, _p$) => {
		e !== _p$?.e && setProp(_el$, "class", e, _p$?.e);
		t !== _p$?.t && setProp(_el$, "style", t, _p$?.t);
		a !== _p$?.a && setProp(_el$, "command", a, _p$?.a);
		o !== _p$?.o && setProp(_el$, "args", o, _p$?.o);
		i !== _p$?.i && setProp(_el$, "cwd", i, _p$?.i);
		n !== _p$?.n && setProp(_el$, "font-family", n, _p$?.n);
		s !== _p$?.s && setProp(_el$, "font-size", s, _p$?.s);
		h !== _p$?.h && setProp(_el$, "line-height", h, _p$?.h);
		r !== _p$?.r && setProp(_el$, "selection-background", r, _p$?.r);
		d !== _p$?.d && setProp(_el$, "selection-foreground", d, _p$?.d);
		l !== _p$?.l && setProp(_el$, "inherit-theme", l, _p$?.l);
		u !== _p$?.u && setProp(_el$, "allow-clipboard-read", u, _p$?.u);
		c !== _p$?.c && setProp(_el$, "sync-window-title", c, _p$?.c);
		w !== _p$?.w && setProp(_el$, "onTerminalExit", w, _p$?.w);
		m !== _p$?.m && setProp(_el$, "onTerminalProgress", m, _p$?.m);
		f !== _p$?.f && setProp(_el$, "onTerminalNotification", f, _p$?.f);
		y !== _p$?.y && setProp(_el$, "onTerminalTitleChange", y, _p$?.y);
		g !== _p$?.g && setProp(_el$, "onTerminalCwdChange", g, _p$?.g);
		p !== _p$?.p && setProp(_el$, "onTerminalSelectionChange", p, _p$?.p);
		b !== _p$?.b && setProp(_el$, "onTerminalBell", b, _p$?.b);
	});
	return _el$;
}
//#endregion
export { Terminal };

//# sourceMappingURL=index.mjs.map