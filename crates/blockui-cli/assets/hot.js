// Injected only by `blockui dev --web`; static releases never load this file.
const panel=document.createElement('aside');
panel.id='blockui-dev-status';panel.setAttribute('role','status');
panel.style.cssText='position:fixed;bottom:12px;right:12px;max-width:min(600px,90vw);padding:10px 14px;background:#18181b;color:white;border-radius:6px;font:13px/1.5 monospace;z-index:2147483647;white-space:pre-wrap';
panel.hidden=true;document.body.append(panel);
const message=(text,error=false)=>{panel.hidden=!text;panel.textContent=text;panel.setAttribute('role',error?'alert':'status');};
const storageKey=`blockui:hot:${location.pathname}`;
let renderer;
for(let attempt=0;attempt<300 && !renderer;attempt++) {
  renderer=window.blockui?.replaceApp ? window.blockui : null;
  if(!renderer) await new Promise(resolve=>setTimeout(resolve,100));
}
if(!renderer) message('BlockUI hot reload requires window.blockui = mount(app, root).',true);
else {
  try {
    const saved=sessionStorage.getItem(storageKey);
    if(saved) {renderer.replaceApp(renderer.app.constructor.restore(saved));sessionStorage.removeItem(storageKey);}
  } catch(error) {message(`State restore failed: ${error}`,true);}
  let latest=null, applied=0, busy=false, retry=0, loads=0, rendererVersion=document.querySelector('script[src="/__blockui/client.js"]')?.dataset.renderer || null;
  let loadedGeneration=0, loadedModule=null;
  function restart() {
    try {sessionStorage.setItem(storageKey,renderer.app.checkpoint());location.reload();}
    catch(error) {message(`Cannot restart yet: ${error}`,true);}
  }
  function limit(text) {
    message(text,true);
    const button=document.createElement('button');button.textContent='Restart preserving state';button.onclick=restart;
    panel.append(document.createElement('br'),button);
  }
  async function apply() {
    if(busy || !latest || latest.error || latest.building || latest.generation===applied) return;
    busy=true;const status=latest;let candidate=null;
    try {
      if(rendererVersion!==null && rendererVersion!==status.renderer) {
        limit('Browser renderer changed. Restart to load the new adapter.');return;
      }
      rendererVersion=status.renderer;
      if(loadedGeneration!==status.generation) {
        if(loads>=20) {limit('20 Wasm generations loaded. Restart to release the browser module cache.');return;}
        const base=`/__blockui/generations/${status.generation}/${status.module}`;
        loads++;
        loadedModule=await import(`${base}.js`);
        await loadedModule.default({module_or_path:`${base}_bg.wasm`});
        loadedGeneration=status.generation;
      }
      if(latest.generation!==status.generation || latest.error || latest.building) return;
      const type=loadedModule[renderer.app.constructor.name];
      if(!type?.restore) throw new Error('Application export changed or lacks state restore');
      // Read state after asynchronous loading, so edits during compilation survive.
      candidate=type.restore(renderer.app.checkpoint());
      renderer.replaceApp(candidate);candidate=null;applied=status.generation;
      for(const link of document.querySelectorAll('link[rel=stylesheet]')) {
        const url=new URL(link.href);if(url.origin!==location.origin) continue;
        url.pathname=url.pathname.replace(/^\/__blockui\/generations\/\d+\//,`/__blockui/generations/${applied}/`);
        url.searchParams.set('blockui-generation',String(applied));link.href=url.href;
      }
      message('');window.dispatchEvent(new CustomEvent('blockui-reload',{detail:{generation:applied}}));
    } catch(error) {
      message(`Hot reload kept the running application: ${error}`,true);
      if(String(error).includes('Web hot reload waits')) retry=setTimeout(apply,1000);
    } finally {
      candidate?.free();busy=false;
      if(latest && latest.generation!==status.generation) queueMicrotask(apply);
    }
  }
  const events=new EventSource('/__blockui/events');
  events.onmessage=event=>{
    latest=JSON.parse(event.data);clearTimeout(retry);
    if(latest.error) message(`Build failed; running application preserved.\n${latest.error}`,true);
    else if(latest.building) message('Building Web update…');
    else apply();
  };
  window.addEventListener('pagehide',()=>{events.close();clearTimeout(retry);},{once:true});
}
