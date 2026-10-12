// Physical side/alignment with collision flip and viewport clamping.
export function popoverOrigin(a, size, viewport, placement) {
  const [w,h]=size, [vw,vh]=viewport;
  let side=Math.floor(placement/3);
  const align=placement%3, room=[vh-24-a[3],a[1]-24,vw-24-a[2],a[0]-24];
  const opposite=side^1, needed=side<2 ? h : w;
  if (room[side]<needed && room[opposite]>room[side]) side=opposite;
  const cross=(start,end,length)=>align===1 ? (start+end-length)/2 : align===2 ? end-length : start;
  let [x,y]=side===0 ? [cross(a[0],a[2],w),a[3]+8]
    : side===1 ? [cross(a[0],a[2],w),a[1]-8-h]
    : side===2 ? [a[2]+8,cross(a[1],a[3],h)] : [a[0]-8-w,cross(a[1],a[3],h)];
  const mx=Math.min(16,vw/2), my=Math.min(16,vh/2);
  return [Math.max(mx,Math.min(x,vw-16-w)),Math.max(my,Math.min(y,vh-16-h))];
}
let nextMount = 0;
// Keyed DOM backend. State and dependency tracking remain in Rust/Wasm.
export function mount(app, root) {
  const prefix = `blockui-${++nextMount}-`;
  const elements = new Map();
  const previous = new Map();
  const tags = {1:'div',2:'span',3:'button',4:'input',5:'div',7:'textarea',8:'div',9:'label',10:'input',11:'progress',12:'svg',13:'dialog',14:'select'};
  let writes = 0;
  let frame = 0, motionFrame = 0, disposed = false;
  let tooltipTimer = 0, tooltipPointer = null, tooltipVisible = null;
  function layoutTooltips() {
    for (const panel of root.querySelectorAll('[data-semantic="21"]:not([hidden])')) {
      const anchor = elements.get(panel.dataset.tooltipAnchor);
      if (!anchor) continue;
      const a=anchor.getBoundingClientRect(), box=panel.getBoundingClientRect();
      const [x,y]=popoverOrigin([a.left,a.top,a.right,a.bottom],[box.width,box.height],[innerWidth,innerHeight],Number(panel.dataset.overlayPosition));
      panel.style.left=`${x}px`; panel.style.top=`${y}px`;
    }
  }
  function tooltipSync(command=0) {
    if (disposed || !app.tooltip_input) return;
    clearTimeout(tooltipTimer); tooltipTimer=0;
    const trigger = target => {
      const control=target?.closest?.('[data-key][aria-describedby]');
      return control && root.contains(control) && !control.closest('[hidden]') && !control.disabled ? control.dataset.key : '';
    };
    const content=tooltipPointer?.closest?.('[data-semantic="21"]');
    if (document.hidden || !document.hasFocus()) command=2;
    const state=JSON.parse(app.tooltip_input(trigger(tooltipPointer),trigger(document.activeElement),content && root.contains(content) ? content.dataset.key : '',performance.now(),command));
    tooltipVisible=state.visible;
    for (const panel of root.querySelectorAll('[data-semantic="21"]')) panel.hidden=panel.dataset.key!==tooltipVisible;
    layoutTooltips();
    if (state.deadline!==null) tooltipTimer=setTimeout(()=>tooltipSync(),Math.max(0,state.deadline-performance.now()));
  }
  function tooltipPointerChanged(event) {
    if (event.pointerType==='touch') return;
    tooltipPointer=event.type==='pointerout' ? event.relatedTarget : event.target;
    tooltipSync();
  }
  function tooltipFocusChanged() { queueMicrotask(()=>{ if (!disposed) tooltipSync(); }); }
  function tooltipKey(event) {
    if (event.key==='Escape' && !event.repeat && !event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey && (tooltipVisible || tooltipTimer)) {
      tooltipSync(1); event.preventDefault(); event.stopImmediatePropagation();
    }
  }
  function tooltipActivate(event) {
    const previousPointer=tooltipPointer;
    tooltipPointer=event.target;
    tooltipSync(1);
    if (event.pointerType==='touch') tooltipPointer=previousPointer;
  }
  function tooltipBlur() { tooltipPointer=null; tooltipSync(2); }
  root.addEventListener('pointerover',tooltipPointerChanged);
  root.addEventListener('pointerout',tooltipPointerChanged);
  root.addEventListener('focusin',tooltipFocusChanged);
  root.addEventListener('focusout',tooltipFocusChanged);
  root.addEventListener('keydown',tooltipKey,true);
  document.addEventListener('pointerdown',tooltipActivate,true);
  window.addEventListener('blur',tooltipBlur);
  window.addEventListener('focus',tooltipFocusChanged);
  document.addEventListener('visibilitychange',tooltipBlur);
  let toastReturnFocus = null;
  const menubarStops=new Map();
  function syncMenubarStops() {
    const bars=[...root.querySelectorAll('[data-semantic="35"]')];
    const live=new Set(bars.map(bar=>bar.dataset.key));
    for (const key of menubarStops.keys()) if (!live.has(key)) menubarStops.delete(key);
    for (const bar of bars) {
      const triggers=[...bar.querySelectorAll('[data-semantic="27"]')].filter(trigger=>trigger.parentElement?.parentElement===bar);
      const selected=triggers.find(trigger=>!trigger.disabled && trigger.dataset.key===menubarStops.get(bar.dataset.key)) ?? triggers.find(trigger=>!trigger.disabled);
      if (selected) menubarStops.set(bar.dataset.key,selected.dataset.key);
      for (const trigger of triggers) trigger.tabIndex=trigger===selected ? 0 : -1;
    }
  }

  function rememberFocus(event) {
    if (!event.target.closest?.('[data-semantic="15"]')) toastReturnFocus = event.target;
    let trigger=event.target.closest?.('[data-semantic="27"]');
    if (!trigger) {
      const panel=event.target.closest?.('[data-semantic="26"]');
      if (panel) trigger=document.getElementById(menuRoot(panel).getAttribute('aria-labelledby'));
    }
    const bar=trigger?.parentElement?.parentElement;
    if (bar?.dataset.semantic==='35') {
      menubarStops.set(bar.dataset.key,trigger.dataset.key);
      syncMenubarStops();
    }

  }
  root.addEventListener('focusin', rememberFocus);
  let toastLayoutQueued = false;
  function layoutToasts() {
    const viewports = [...root.querySelectorAll('[data-semantic="15"]')].filter(el=>el.childElementCount);
    const remaining = Array(6).fill(0), occupied = Array(6).fill(0);
    for (const viewport of viewports) remaining[Number(viewport.dataset.toastPosition ?? 0)]++;
    for (const viewport of viewports) {
      const position = Number(viewport.dataset.toastPosition ?? 0), count = remaining[position]--;
      const available = Math.max(0,innerHeight-32-occupied[position]-12*(count-1))/count;
      for (const [name,value] of [['--toast-offset',occupied[position]],['--toast-available',available]]) {
        const text = `${value}px`;
        if (viewport.style.getPropertyValue(name)!==text) viewport.style.setProperty(name,text);
      }
      occupied[position] += viewport.getBoundingClientRect().height + 12;
    }
  }
  let contextAnchor=null;
  function layoutPopovers() {
    for (const panel of root.querySelectorAll(':is([data-semantic="16"],[data-semantic="26"]):not([hidden])')) {
      const anchor = document.getElementById(panel.getAttribute('aria-labelledby'));
      if (!anchor) continue;
      const a = anchor.getBoundingClientRect(), box = panel.getBoundingClientRect();
      const [x,y]=contextAnchor?.key===anchor.dataset.key
        ? JSON.parse(app.context_origin(...contextAnchor.point,box.width,box.height,innerWidth,innerHeight))
        : popoverOrigin([a.left,a.top,a.right,a.bottom],[box.width,box.height],[innerWidth,innerHeight],Number(panel.dataset.overlayPosition??0));
      for (const [name,value] of [['left',x],['top',y]]) {
        const text = `${value}px`;
        if (panel.style[name]!==text) panel.style[name]=text;
      }
    }
  }
  let menuPointer=null, menuHoverTimer=0;
  function menuHoverReset() {
    clearTimeout(menuHoverTimer); menuHoverTimer=0; menuPointer=null;
    if (!disposed) app.menu_hover?.('{"reset":true}');
  }
  function menuHoverSync() {
    clearTimeout(menuHoverTimer); menuHoverTimer=0;
    if (disposed || !menuPointer || !app.menu_hover || document.hidden || !document.hasFocus()) return;
    const [x,y]=menuPointer;
    const hit=document.elementFromPoint(x,y)?.closest('[data-key]');
    if (hit && root.contains(hit) && !hit.disabled && hit.dataset.semantic==='27'
        && hit.parentElement?.parentElement?.dataset.semantic==='35') {
      const decision=JSON.parse(app.menubar_input(hit.dataset.key,'hover'));
      if (decision.open) {
        app.menu_hover('{"reset":true}');
        hit.focus({preventScroll:true});
        app.event(decision.open,'click',''); update();
        return;
      }
    }
    const target=hit && root.contains(hit) && !hit.disabled && !hit.closest('[hidden]')
      && [28,29,30,34].includes(Number(hit.dataset.semantic)) ? hit : null;
    const panels=[...root.querySelectorAll('[data-semantic="26"]:not([hidden])')];
    const panel=panels.reverse().find(panel=>document.getElementById(panel.getAttribute('aria-labelledby'))?.dataset.semantic==='34');
    const anchor=panel && document.getElementById(panel.getAttribute('aria-labelledby'));
    const rect=el=>{const r=el.getBoundingClientRect();return [r.left,r.top,r.right,r.bottom];};
    const result=JSON.parse(app.menu_hover(JSON.stringify({point:[x,y],hovered:target?.dataset.key ?? null,now:performance.now(),branch:anchor ? {key:anchor.dataset.key,anchor:rect(anchor),panel:rect(panel)} : null})));
    if (result.focus) {
      const control=elements.get(result.focus);
      if (control) {
        if (panel && control!==anchor && control.closest('[data-semantic="26"]')===anchor.closest('[data-semantic="26"]')) dismissPopover(panel);
        control.focus({preventScroll:true});
      }
    }
    if (result.open) { app.event(result.open,'click',''); update(); }
    if (result.deadline!==null) menuHoverTimer=setTimeout(menuHoverSync,Math.max(0,result.deadline-performance.now()));
  }
  function menuPointerMoved(event) {
    if (event.pointerType==='touch' || event.buttons) { menuHoverReset(); return; }
    menuPointer=[event.clientX,event.clientY]; menuHoverSync();
  }
  document.addEventListener('pointermove',menuPointerMoved);
  document.addEventListener('pointerdown',menuHoverReset,true);
  let openingMenuLast = null;
  function menuFocus(panel, command, text='') {
    const key=app.menu_input(panel.dataset.key, document.activeElement?.dataset.key ?? '', command, text, performance.now());
    const target=elements.get(key) ?? panel;
    target.focus({preventScroll:true});
    const box=target.getBoundingClientRect(), port=panel.getBoundingClientRect();
    if (box.top<port.top) panel.scrollTop-=port.top-box.top;
    else if (box.bottom>port.bottom) panel.scrollTop+=box.bottom-port.bottom;
  }
  function menuRoot(panel) {
    let parent;
    while ((parent=panel.parentElement?.closest('[data-semantic="26"]'))) panel=parent;
    return panel;
  }
  function menuBlur() {
    menuHoverReset();
    const panels=[...root.querySelectorAll('[data-semantic="26"]:not([hidden])')]
      .filter(panel=>!panel.parentElement.closest('[data-semantic="26"]'));
    for (const panel of panels) if (panel.isConnected && !panel.hidden) dismissPopover(panel);
  }
  window.addEventListener('blur',menuBlur);
  document.addEventListener('visibilitychange',menuBlur);
  function dismissPopover(panel) {
    app.event(panel.dataset.key,'dismiss-popover','');
    update();
  }
  function outsidePopover(event) {
    const panels = [...root.querySelectorAll(':is([data-semantic="16"],[data-semantic="26"]):not([hidden])')];
    const panel = panels.at(-1);
    if (!panel) return;
    const anchor = document.getElementById(panel.getAttribute('aria-labelledby'));
    if (!panel.contains(event.target) && !anchor?.contains(event.target)) {
      const owner=panel.dataset.semantic==='26' ? menuRoot(panel) : panel;
      const rootAnchor=document.getElementById(owner.getAttribute('aria-labelledby'));
      if (!rootAnchor?.contains(event.target)) dismissPopover(owner.contains(event.target) ? panel : owner);
    }
  }
  function popoverFocusChanged(event) {
    // Focusout runs before the browser installs the next active element. Capture
    // the intended destination; BODY may only be an intermediate focus state.
    const target = event.relatedTarget;
    if (target) queueMicrotask(()=>{ if (!disposed) outsidePopover({target}); });
  }
  document.addEventListener('pointerdown',outsidePopover,true);
  root.addEventListener('focusout',popoverFocusChanged);
  window.addEventListener('scroll',queueToastLayout,true);
  function queueToastLayout() {
    if (disposed || toastLayoutQueued) return;
    toastLayoutQueued = true;
    queueMicrotask(()=>{ toastLayoutQueued=false; if (!disposed) { layoutToasts(); layoutPopovers(); layoutTooltips(); queueToastActivity(); } });
  }
  const toastObserver = new ResizeObserver(queueToastLayout);
  window.addEventListener('resize', queueToastLayout);
  let toastActivityQueued = false;
  function queueToastActivity() {
    if (disposed || toastActivityQueued) return;
    toastActivityQueued = true;
    queueMicrotask(() => {
      toastActivityQueued = false;
      if (disposed) return;
      let changed = false;
      const blocked = document.hidden || !document.hasFocus() || !!root.querySelector('dialog[open]');
      for (const viewport of root.querySelectorAll('[data-semantic="15"]')) {
        const paused = blocked || viewport.matches(':hover') || viewport.contains(document.activeElement);
        if (viewport.dataset.toastPaused === String(paused)) continue;
        app.event(viewport.dataset.key, 'toast-activity', paused ? '1' : '0');
        viewport.dataset.toastPaused = String(paused);
        changed = true;
      }
      if (changed) update();
    });
  }
  const toastActivityEvents = ['pointerover','pointerout','focusin','focusout'];
  toastActivityEvents.forEach(event => root.addEventListener(event, queueToastActivity));
  window.addEventListener('blur', queueToastActivity);
  window.addEventListener('focus', queueToastActivity);
  document.addEventListener('visibilitychange', queueToastActivity);
  const motionPreference = matchMedia('(prefers-reduced-motion: reduce)');
  app.motion_reduced?.(motionPreference.matches);
  function requestMotionFrame() {
    if (!disposed && !motionFrame && app.motion_active?.()) motionFrame = requestAnimationFrame(paintMotion);
  }
  function paintMotion(now) {
    motionFrame = 0;
    if (disposed) return;
    for (const [key, value] of JSON.parse(app.motion_frame(now))) {
      elements.get(key)?.querySelector('input[role="switch"]')?.style.setProperty('--switch-position', String(value));
    }
    requestMotionFrame();
  }
  function motionPreferenceChanged(event) { app.motion_reduced?.(event.matches); requestMotionFrame(); }
  motionPreference.addEventListener('change', motionPreferenceChanged);
  const sizes = new Map();
  const observer = new ResizeObserver(entries => {
    for (const {target:el} of entries) {
      if (!el.isConnected || !el.dataset.viewport) continue;
      const size = JSON.stringify([el.clientWidth, el.clientHeight]);
      if (sizes.get(el.dataset.key) === size) continue;
      sizes.set(el.dataset.key, size);
      app.event(el.dataset.key, 'viewport', size);
    }
    update();
  });
  function update() {
    const nodes = JSON.parse(app.snapshot());
    const focusedControl = root.contains(document.activeElement) ? document.activeElement : null;
    const focusedControlKey = focusedControl?.dataset.key;

    if (contextAnchor && !nodes.some(node=>node.key===contextAnchor.key && [36,37].includes(node.semantics.role) && node.semantics.expanded===2)) contextAnchor=null;

    const focusedDisclosures = [];
    for (let el = document.activeElement; el && root.contains(el); el = el.parentElement) {
      if (el.dataset.semantic === '12') focusedDisclosures.push(el.dataset.key);
    }
    const popoverFocusPath=[];
    for (let panel=document.activeElement?.closest?.(':is([data-semantic="16"],[data-semantic="26"])'); panel; panel=panel.parentElement?.closest(':is([data-semantic="16"],[data-semantic="26"])')) {
      popoverFocusPath.push({panel,anchor:panel.getAttribute('aria-labelledby')});
    }
    const openingPopovers = nodes.filter(n=>[16,26].includes(n.semantics.role) && !n.semantics.hidden && (!previous.has(n.key) || JSON.parse(previous.get(n.key)).semantics.hidden));
    const focusedToast = document.activeElement?.closest?.('[data-semantic="15"]') ? document.activeElement : null;
    const focusedPanel = root.contains(document.activeElement) ? document.activeElement.closest('[role="tabpanel"]')?.dataset.key : null;
    const live = new Set(nodes.map(n => n.key));
    for (const [key, el] of elements) if (!live.has(key)) { app.motion_remove?.(key); observer.unobserve(el); toastObserver.unobserve(el); sizes.delete(key); if (el.tagName === 'DIALOG' && el.open) el.close(); el.remove(); elements.delete(key); previous.delete(key); }
    for (const n of nodes) {
      let el = elements.get(n.key);
      const tag = tags[n.kind];
      if (!tag) throw new Error(`Unsupported browser node kind ${n.kind}`);
      if (el && el.tagName.toLowerCase() !== tag) { app.motion_remove?.(n.key); observer.unobserve(el); toastObserver.unobserve(el); el.remove(); elements.delete(n.key); previous.delete(n.key); el = null; }
      if (!el) {
        el = n.kind === 12 ? document.createElementNS('http://www.w3.org/2000/svg', tag) : document.createElement(tag); el.dataset.key = n.key; el.dataset.kind = n.kind; el.id = prefix + n.key;
        if (n.kind === 9) { const control = document.createElement('input'); control.type = 'checkbox'; el.append(control, document.createElement('span')); }
        if (n.kind === 10) el.type = 'range';
        if (n.kind === 3) el.type = 'button';
        elements.set(n.key, el);
      }
      if ([15,16,17,21,26,27].includes(n.semantics.role) || n.semantics.describedBy !== '0' && n.semantics.describedBy) toastObserver.observe(el); else toastObserver.unobserve(el);
      if (previous.get(n.key) !== JSON.stringify(n)) {
        patch(el, n); previous.set(n.key, JSON.stringify(n)); writes++;
      }
      const parent = elements.get(n.parent) ?? root;
      if (el.parentNode !== parent) parent.append(el);
      if (n.viewport && !el.dataset.viewport) { el.dataset.viewport = 'true'; observer.observe(el); }

    }
    for (const n of nodes) {
      const el = elements.get(n.key);
      if (n.kind === 5 && n.contentWidth) for (const child of el.children) child.style.minWidth = `${n.contentWidth}px`;
      if (n.gridRows) { el.setAttribute('role','grid'); el.setAttribute('aria-rowcount',n.gridRows); el.setAttribute('aria-colcount',n.gridColumns); el.dataset.columns = n.gridColumns; }
      else if (el.parentElement?.getAttribute('role') === 'grid' && n.kind === 8) el.setAttribute('role','row');
      else if (el.parentElement?.getAttribute('role') === 'row' && n.kind === 3) {
        el.setAttribute('role','gridcell'); el.dataset.index = n.index;
        el.dataset.selected = n.text.startsWith('✓');
        el.setAttribute('aria-selected',el.dataset.selected);
        el.textContent = n.text.replace(/^✓\s*/, '');
      }
    }
    syncMenubarStops();
    // Move only elements whose sibling position changed; preserve input focus/selection.
    const children = new Map();
    for (const n of nodes) { const parent = elements.get(n.parent) ?? root; if (!children.has(parent)) children.set(parent, []); children.get(parent).push(elements.get(n.key)); }
    for (const [parent, desired] of children) desired.forEach((el, i) => { if (parent.children[i] !== el) parent.insertBefore(el, parent.children[i] ?? null); });
    if (focusedPanel && nodes.find(node=>node.key===focusedPanel)?.semantics?.hidden) {
      const owner = nodes.find(node=>node.semantics?.role===4 && node.semantics.controls===focusedPanel)?.parent;
      const next = nodes.find(node=>node.parent===owner && node.semantics?.role===4 && node.semantics.tabStop && !node.disabled);
      if (next) elements.get(next.key)?.focus();
    }
    for (const key of focusedDisclosures) {
      const panel = nodes.find(node=>node.key === key);
      if (panel?.semantics.hidden) {
        const trigger = nodes.find(node=>node.key === panel.semantics.labelledBy);
        if (trigger && !trigger.disabled) elements.get(trigger.key)?.focus();
        break;
      }
    }
    if (focusedToast && !focusedToast.isConnected) {
      if (toastReturnFocus?.isConnected && !toastReturnFocus.disabled) toastReturnFocus.focus();
      else root.querySelector('button:enabled,input:enabled,textarea:enabled,select:enabled')?.focus();
    }
    for (const el of elements.values()) if (el.tagName === 'DIALOG' && !el.open) el.showModal();
    layoutToasts();
    layoutPopovers();
    tooltipSync();
    for (const {panel,anchor} of popoverFocusPath) {
      if (!panel.isConnected || panel.hidden) {
        const trigger=document.getElementById(anchor);
        if (trigger?.isConnected && !trigger.disabled && !trigger.closest('[hidden]')) { trigger.focus(); break; }
      }
    }
    for (const panel of openingPopovers) {
      const el = elements.get(panel.key);
      if (panel.semantics.role===26) {
        menuFocus(el, openingMenuLast===panel.key ? 'open-last' : 'open');
        openingMenuLast=null;
      } else (el.querySelector('button:enabled,input:enabled,textarea:enabled,select:enabled') ?? el).focus({preventScroll:true});
    }
    const focusedMenu=document.activeElement?.closest?.('[data-semantic="26"]:not([hidden])');
    if (focusedMenu) menuFocus(focusedMenu,'reconcile');
    if (focusedControlKey && focusedControl.disabled &&
        (document.activeElement===focusedControl || document.activeElement===document.body)) {
      const next=app.navigation_focus?.(focusedControlKey);
      if (next) elements.get(next)?.focus();
    }
    queueToastActivity();
  }
  function patch(el, n) {
    el.dataset.tooltipAnchor = n.tooltipAnchor ?? '0';
    el.dataset.toastPaused = String(!!n.toastPaused);
    el.dataset.overlayPosition = String(n.overlayPosition ?? 0);
    el.dataset.toastPosition = String(n.toastPosition ?? 0);
    el.dataset.variant = n.variant;
    el.dataset.scroll = n.scroll;
    el.dataset.tone = n.tone; el.dataset.surface = n.surface;
    const style = el.style;
    style.gap = `${n.gap}px`; style.padding = `${n.padding}px`;
    style.width = n.width ? `${n.width}px` : '';
    style.flex = [24,25].includes(n.semantics?.role) ? (n.width ? '0 1 auto' : '1 1 0px') : (n.width ? '0 0 auto' : '');
    if (n.contentWidth && el.firstElementChild) el.firstElementChild.style.minWidth = `${n.contentWidth}px`;
    style.maxWidth = n.maxWidth ? `${n.maxWidth}px` : '';
    style.fontSize = n.fontSize ? `${n.fontSize}px` : '';
    style.borderRadius = n.radius ? `${n.radius}px` : '';
    style.height = n.height && n.kind !== 2 ? `${n.height}px` : '';
    const semantic = n.semantics ?? {};
    el.dataset.semantic = semantic.role ?? 0;
    const roles = {1:'heading',2:'navigation',3:'tablist',4:'tab',5:'tabpanel',6:'radiogroup',7:'radio',9:'alert',10:'status',11:'group',12:'region',14:'group',15:'region',16:'dialog',18:'separator',19:'separator',21:'tooltip',22:'table',23:'row',24:'columnheader',25:'cell',26:'menu',28:'menuitem',29:'menuitemcheckbox',30:'menuitemradio',31:'group',34:'menuitem',35:'menubar',37:'group'};
    const role = semantic.menubarTrigger ? 'menuitem' : roles[semantic.role];
    if (role) el.setAttribute('role',role); else el.removeAttribute('role');
    el.hidden = semantic.role===21 ? el.dataset.key!==tooltipVisible : !!semantic.hidden;
    if ([9,10].includes(semantic.role)) el.setAttribute('aria-atomic','true'); else el.removeAttribute('aria-atomic');
    for (const [name,value] of Object.entries({
      'aria-sort': semantic.role===24 ? ({1:'ascending',2:'descending',3:'other'}[semantic.sortDirection] ?? null) : null,
      'aria-orientation': semantic.role===18 ? 'horizontal' : semantic.role===19 ? 'vertical' : null,
      'aria-modal': semantic.role===16 ? false : null,
      'aria-haspopup': semantic.role===17 ? 'dialog' : [27,34,36,37].includes(semantic.role) ? 'menu' : null,
      'aria-live': semantic.role === 15 ? 'polite' : null,
      'aria-level': semantic.role === 1 ? semantic.level : null,
      'aria-selected': semantic.role === 4 ? semantic.selected : null,
      'aria-current': semantic.current === 1 ? 'page' : null,
      'aria-checked': [7,29,30].includes(semantic.role) ? semantic.selected : null,
      'aria-expanded': semantic.expanded ? semantic.expanded === 2 : null,
      'aria-disabled': semantic.locked || (semantic.role===37 && n.disabled) ? true : null,
      'aria-controls': semantic.controls && semantic.controls !== '0' ? prefix + semantic.controls : null,
      'aria-describedby': semantic.describedBy && semantic.describedBy !== '0' ? prefix + semantic.describedBy : null,
      'aria-labelledby': semantic.role!==37 && semantic.labelledBy && semantic.labelledBy !== '0' ? prefix + semantic.labelledBy : null,
    })) { if (value !== null) el.setAttribute(name,String(value)); else el.removeAttribute(name); }
    if (semantic.role===37) el.tabIndex=n.disabled ? -1 : 0;
    else if ([4,5,7].includes(semantic.role)) el.tabIndex = semantic.tabStop ? 0 : -1;
    else if ([16,26,28,29,30,34].includes(semantic.role)) el.tabIndex = -1;
    else el.removeAttribute('tabindex');
    const label = semantic.role === 15 ? 'Notifications' : n.label || ([10,13].includes(n.kind) || [3,6,29,30,34].includes(semantic.role) ? n.text : '');
    if (label && !el.hasAttribute('aria-labelledby')) el.setAttribute('aria-label', label); else el.removeAttribute('aria-label');
    if ([3,4,7,10,14].includes(n.kind)) el.disabled = n.disabled;
    if ([4,7].includes(n.kind)) { el.readOnly = n.readonly; if (el.value !== n.text) el.value = n.text; }
    if ([2,3].includes(n.kind)) el.textContent = n.text;
    if (n.kind === 9) {
      for (const attribute of ['aria-describedby','aria-labelledby']) {
        const value=el.getAttribute(attribute);
        if (value) el.firstElementChild.setAttribute(attribute,value);
        else el.firstElementChild.removeAttribute(attribute);
      }
      el.firstElementChild.disabled = n.disabled; el.firstElementChild.checked = n.checked;
      if (semantic.role === 8) { el.firstElementChild.setAttribute('role','switch'); el.firstElementChild.setAttribute('aria-checked',String(n.checked)); }
      else { el.firstElementChild.removeAttribute('role'); el.firstElementChild.removeAttribute('aria-checked'); }
      el.lastElementChild.textContent = n.text.replace(/^[☐☑]\s*/, '');
      if (semantic.role === 8 && app.motion_target) {
        const position = app.motion_target(n.key, !!n.checked, performance.now(), !!n.disabled);
        el.firstElementChild.style.setProperty('--switch-position', String(position));
        requestMotionFrame();
      } else {
        app.motion_remove?.(n.key);
        el.firstElementChild.style.removeProperty('--switch-position');
      }
    }
    if (n.kind === 10) { [el.min, el.max, el.step] = n.range.map(String); el.value = n.value; }
    if (n.kind === 14) {
      const signature = JSON.stringify(n.options);
      if (el.dataset.options !== signature) {
        el.replaceChildren(...n.options.map(o => { const option = document.createElement('option'); option.value = o.value; option.textContent = o.label; option.disabled = !o.enabled; return option; }));
        el.dataset.options = signature;
      }
      el.value = n.value;
    }
    if (n.kind === 11) { el.max = 1; el.value = Number(n.text); }
    if (n.kind === 12) {
      el.setAttribute('role', 'img'); el.setAttribute('aria-label', n.canvas.label);
      el.replaceChildren(...n.canvas.circles.map(c => {
        const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        const color = rgba => rgba === null ? 'none' : `#${rgba.toString(16).padStart(8,'0')}`;
        for (const [key,value] of Object.entries({cx:c.x,cy:c.y,r:c.radius,fill:color(c.fill),stroke:color(c.stroke),'stroke-width':c.strokeWidth})) circle.setAttribute(key,value);
        return circle;
      }));
    }
  }
  function openContext(trigger, point) {
    menuHoverReset();
    const ownPanel=trigger.getAttribute('aria-controls');
    for (const panel of [...root.querySelectorAll(':is([data-semantic="16"],[data-semantic="26"]):not([hidden])')]) {
      if (!panel.closest('[hidden]') && !panel.parentElement?.closest('[data-semantic="26"]') && panel.id!==ownPanel) dismissPopover(panel);
    }
    contextAnchor=point ? {key:trigger.dataset.key,point} : null;
    trigger.focus({preventScroll:true});
    app.event(trigger.dataset.key,'click',''); update();
    const panel=document.getElementById(ownPanel);
    if (panel && !panel.hidden) menuFocus(panel,'open');
  }
  function dispatch(e) {
    if (e.type==='keydown') menuHoverReset();
    if (e.type==='click') tooltipSync(1);
    let el = e.target.closest('[data-key]'); if (!el || !root.contains(el)) return;
    let contextTarget=null;
    for (let candidate=el; candidate && root.contains(candidate); candidate=candidate.parentElement) {
      if (['16','26'].includes(candidate.dataset.semantic)) break;
      if (['36','37'].includes(candidate.dataset.semantic)) { contextTarget=candidate; break; }
    }
    if (contextTarget && !contextTarget.disabled && contextTarget.getAttribute('aria-disabled')!=='true') {
      if (e.type==='contextmenu') {
        e.preventDefault(); openContext(contextTarget,[e.clientX,e.clientY]); return;
      }
      if (e.type==='keydown' && !e.isComposing && !e.altKey && !e.ctrlKey && !e.metaKey
          && ((e.key==='ContextMenu' && !e.shiftKey) || (e.key==='F10' && e.shiftKey))) {
        e.preventDefault(); if (!e.repeat) openContext(contextTarget,null); return;
      }
      if (e.type==='click' && el===contextTarget && el.dataset.semantic==='36') contextAnchor=null;
    }
    if (e.type==='keydown' && !e.isComposing && !e.altKey && !e.ctrlKey && !e.metaKey) {
      if (!e.shiftKey && ['27','36'].includes(el.dataset.semantic) && ['ArrowDown','ArrowUp'].includes(e.key)) {
        e.preventDefault();
        if (!e.repeat) {
          const panel=elements.get(el.getAttribute('aria-controls')?.slice(prefix.length));
          if (el.getAttribute('aria-expanded')==='true') menuFocus(panel,e.key==='ArrowUp' ? 'open-last' : 'open');
          else {
            openingMenuLast=e.key==='ArrowUp' ? panel?.dataset.key : null;
            app.event(el.dataset.key,'click',''); update();
          }
        }
        return;
      }
      if (!e.shiftKey && el.dataset.semantic==='34' && e.key==='ArrowRight') {
        e.preventDefault();
        if (!e.repeat) {
          if (el.getAttribute('aria-expanded')==='true') menuFocus(document.getElementById(el.getAttribute('aria-controls')),'open');
          else { app.event(el.dataset.key,'click',''); update(); }
        }
        return;
      }
      const panel=el.closest('[data-semantic="26"]:not([hidden])');
      if (panel && !e.shiftKey && e.key==='ArrowLeft' && document.getElementById(panel.getAttribute('aria-labelledby'))?.dataset.semantic==='34') {
        e.preventDefault(); if (!e.repeat) dismissPopover(panel); return;
      }
      const rootTrigger=el.dataset.semantic==='27' ? el : panel ? document.getElementById(menuRoot(panel).getAttribute('aria-labelledby')) : null;
      if (!e.shiftKey && rootTrigger?.parentElement?.parentElement?.dataset.semantic==='35'
          && (['ArrowLeft','ArrowRight'].includes(e.key) || (el===rootTrigger && ['Home','End'].includes(e.key)))) {
        e.preventDefault();
        if (!e.repeat) {
          const command={ArrowLeft:'previous',ArrowRight:'next',Home:'first',End:'last'}[e.key];
          const decision=JSON.parse(app.menubar_input(rootTrigger.dataset.key,command));
          const target=elements.get(decision.focus);
          target?.focus({preventScroll:true});
          if (decision.open) {
            const trigger=elements.get(decision.open);
            if (trigger.getAttribute('aria-expanded')==='true') menuFocus(document.getElementById(trigger.getAttribute('aria-controls')),'open');
            else { app.event(decision.open,'click',''); update(); }
          }
        }
        return;
      }
      if (panel && e.key==='Tab') {
        e.preventDefault();
        const owner=menuRoot(panel);
        const anchor=document.getElementById(owner.getAttribute('aria-labelledby'));
        const choices=[...root.querySelectorAll('button:enabled,input:enabled,textarea:enabled,select:enabled,[tabindex="0"]')]
          .filter(control=>!control.closest('[hidden],[data-semantic="26"]') && control.tabIndex>=0 && control.getClientRects().length);
        const next=choices[choices.indexOf(anchor)+(e.shiftKey ? -1 : 1)];
        dismissPopover(owner);
        (next ?? anchor)?.focus();
        return;
      }
      if (panel && !e.shiftKey) {
        const command={ArrowDown:'next',ArrowUp:'previous',Home:'first',End:'last'}[e.key];
        if (command || (e.key!==' ' && [...e.key].length===1)) {
          e.preventDefault(); menuFocus(panel,command ?? 'text',command ? '' : e.key); return;
        }
      }
    }
    if (e.type === 'keydown' && !e.repeat && !e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey
        && !root.querySelector('dialog[open]')) {
      let target;
      if (e.key === 'F8') {
        target = [...root.querySelectorAll('[data-semantic="15"] button:not(:disabled)')]
          .find(control=>control.getClientRects().length && !control.closest('[hidden]'));
      } else if (e.key === 'Escape' && el.closest('[data-semantic="15"]')) {
        target = toastReturnFocus?.isConnected && root.contains(toastReturnFocus)
          && !toastReturnFocus.disabled && !toastReturnFocus.closest('[hidden]') ? toastReturnFocus : null;
      }
      if (target) {
        e.preventDefault();
        target.focus({preventScroll:true});
        // Scroll only the floating plane, never the authored page ancestors.
        const viewport = target.closest('[data-semantic="15"]');
        if (viewport) {
          const box = target.getBoundingClientRect(), port = viewport.getBoundingClientRect();
          if (box.top < port.top) viewport.scrollTop -= port.top - box.top;
          else if (box.bottom > port.bottom) viewport.scrollTop += box.bottom - port.bottom;
        }
        return;
      }
    }
    if (e.type==='keydown' && e.key==='Escape' && !e.repeat && !e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey) {
      const panel = el.closest(':is([data-semantic="16"],[data-semantic="26"]):not([hidden])');
      if (panel) { e.preventDefault(); dismissPopover(panel); return; }
    }
    // Escape dismisses the active modal even when its focused child is an editor.
    // Outside a modal, editors retain their ordinary cancel command.
    if (e.type === 'keydown' && e.key === 'Escape') el = el.closest('dialog[open]') ?? el;
    const kind = Number(el.dataset.kind);
    if (e.type === 'keydown' && !e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey
        && el.dataset.semantic === '13' && ['ArrowUp','ArrowDown','Home','End'].includes(e.key)) {
      const owner = el.closest('[data-semantic="11"]');
      const choices = [...(owner?.querySelectorAll('[data-semantic="13"]') ?? [])]
        .filter(child=>!child.disabled && child.closest('[data-semantic="11"]')===owner);
      const index = choices.indexOf(el);
      if (index >= 0) {
        e.preventDefault();
        choices[e.key==='Home' ? 0 : e.key==='End' ? choices.length-1 : (index + (e.key==='ArrowUp' ? choices.length-1 : 1)) % choices.length].focus();
        return;
      }
    }
    if (e.type === 'keydown' && !e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey && ['tab','radio'].includes(el.getAttribute('role'))) {
      const role = el.getAttribute('role');
      const backward = ['ArrowLeft','ArrowUp'].includes(e.key), forward = ['ArrowRight','ArrowDown'].includes(e.key);
      if ((backward || forward || ['Home','End'].includes(e.key)) && !(role === 'tab' && ['ArrowUp','ArrowDown'].includes(e.key))) {
        const choices = [...el.parentElement.children].filter(child=>child.getAttribute('role')===role && !child.disabled);
        const index = choices.indexOf(el);
        if (index >= 0) {
          const next = choices[e.key === 'Home' ? 0 : e.key === 'End' ? choices.length-1 : (index + (backward ? choices.length-1 : 1)) % choices.length];
          e.preventDefault();
          app.event(next.dataset.key,'click',''); update();
          elements.get(next.dataset.key)?.focus();
          return;
        }
      }
    }
    let action, value = '';
    if (e.type === 'click' && [3,9].includes(kind)) { if (kind === 9 && e.target.tagName !== 'INPUT') return; action = 'click'; }
    if (e.type === 'input' && [4,7].includes(kind)) { action = 'input'; value = el.value; }
    if (e.type === 'input' && kind === 10) { action = 'slider'; value = el.value; }
    if (e.type === 'change' && kind === 14) { action = 'select'; value = el.value; }
    if (e.type === 'keydown' && kind === 4 && ['Enter','Escape'].includes(e.key)) { action = e.key === 'Enter' ? 'submit' : 'cancel'; e.preventDefault(); }
    if (kind === 12 && ['click','contextmenu'].includes(e.type)) {
      const rect = el.getBoundingClientRect(); action = e.type === 'click' ? 'canvas' : 'secondary';
      value = JSON.stringify([e.clientX-rect.left, e.clientY-rect.top]); e.preventDefault();
    }
    if (kind === 13 && (e.type === 'cancel' || (e.type === 'keydown' && e.key === 'Escape'))) { action = 'dismiss'; e.preventDefault(); }
    if (e.type === 'scroll' && kind === 5 && el.dataset.scroll === 'true') { action = 'scroll'; value = JSON.stringify([el.scrollLeft,el.scrollTop]); }
    if (!action) return;
    const popoverOwner = el.closest(':is([data-semantic="16"],[data-semantic="26"])');
    const popoverAnchor = popoverOwner?.getAttribute('aria-labelledby');
    try {
      app.event(el.dataset.key, action, value); update();
      if (e.type==='click' && popoverOwner && (!popoverOwner.isConnected || popoverOwner.hidden)) {
        document.getElementById(popoverAnchor)?.focus();
      }
    }
    catch (error) { root.dispatchEvent(new CustomEvent('blockui-error', {detail: String(error)})); throw error; }
  }
  const events = ['click','input','change','keydown','contextmenu','cancel','scroll'];
  events.forEach(event => root.addEventListener(event, dispatch, true));
  update();
  function poll() {
    if (disposed) return;
    if (app.poll?.()) update();
    frame = requestAnimationFrame(poll);
  }
  if (app.poll) frame = requestAnimationFrame(poll);
  function replaceApp(next) {
    if (disposed) throw new Error('Cannot replace an unmounted application');
    const nodes=JSON.parse(next.snapshot());
    if (nodes.some(node=>!tags[node.kind])) throw new Error('Unsupported candidate browser recipe');
    const focused=document.activeElement;
    const focusKey=focused?.closest?.('[data-key]')?.dataset.key;
    const selection=focused && typeof focused.selectionStart==='number' ? [focused.selectionStart,focused.selectionEnd,focused.selectionDirection] : null;
    const scroll=[window.scrollX,window.scrollY];
    const old=app;
    clearTimeout(menuHoverTimer); menuPointer=null;
    clearTimeout(tooltipTimer); tooltipPointer=null; tooltipVisible=null;
    cancelAnimationFrame(motionFrame); motionFrame=0;
    app=next;
    try { previous.clear(); update(); }
    catch(error) { app=old; previous.clear(); update(); throw error; }
    const retained=elements.get(focusKey);
    if (retained && !retained.disabled && !retained.closest('[hidden]')) {
      const control=retained.tagName==='LABEL' ? retained.querySelector('input') : retained;
      control?.focus({preventScroll:true});
      if (selection && control?.setSelectionRange) control.setSelectionRange(...selection);
    }
    window.scrollTo(...scroll);
    old.free();
    return true;
  }
  return { get app() { return app; }, replaceApp, update, get writes() { return writes; }, unmount() { disposed = true; clearTimeout(tooltipTimer); tooltipTimer=0; tooltipPointer=null; tooltipVisible=null;
      root.removeEventListener('pointerover',tooltipPointerChanged); root.removeEventListener('pointerout',tooltipPointerChanged);
      root.removeEventListener('focusin',tooltipFocusChanged); root.removeEventListener('focusout',tooltipFocusChanged);
      root.removeEventListener('keydown',tooltipKey,true); document.removeEventListener('pointerdown',tooltipActivate,true);
      window.removeEventListener('blur',tooltipBlur); window.removeEventListener('blur',menuBlur); document.removeEventListener('visibilitychange',menuBlur);
      clearTimeout(menuHoverTimer); menuPointer=null;
      document.removeEventListener('pointermove',menuPointerMoved);
      document.removeEventListener('pointerdown',menuHoverReset,true); window.removeEventListener('focus',tooltipFocusChanged);
      document.removeEventListener('visibilitychange',tooltipBlur);
      cancelAnimationFrame(frame); cancelAnimationFrame(motionFrame); motionPreference.removeEventListener('change', motionPreferenceChanged); observer.disconnect(); toastObserver.disconnect(); document.removeEventListener('pointerdown',outsidePopover,true); root.removeEventListener('focusout',popoverFocusChanged); window.removeEventListener('scroll',queueToastLayout,true); window.removeEventListener('resize', queueToastLayout); root.removeEventListener('focusin',rememberFocus); toastActivityEvents.forEach(event => root.removeEventListener(event, queueToastActivity)); window.removeEventListener('blur', queueToastActivity); window.removeEventListener('focus', queueToastActivity); document.removeEventListener('visibilitychange', queueToastActivity); toastReturnFocus = null; events.forEach(event => root.removeEventListener(event, dispatch, true)); elements.clear(); previous.clear(); menubarStops.clear(); contextAnchor=null; root.replaceChildren(); app.free(); } };
}
