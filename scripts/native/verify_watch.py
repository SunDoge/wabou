#!/usr/bin/env python3
"""Real live Cargo watcher: dependency addition, retry, target rename and removal."""
import argparse,json,os,pathlib,queue,shutil,statistics,subprocess,tempfile,threading,time
from dev import ROOT,ENV,cargo_tool,configuration
from storage import disk

def main():
    global ROOT
    parser=argparse.ArgumentParser(description=__doc__)
    modes=parser.add_mutually_exclusive_group()
    modes.add_argument('--async-join',action='store_true',help='exercise independent host timer inputs joined through a record memo')
    modes.add_argument('--async-value',action='store_true',help='exercise host timer completion into durable async result state')
    modes.add_argument('--style-patch',action='store_true',help='exercise ordered static and live style patches through actual dylib reloads')
    modes.add_argument('--live-each-scoped',action='store_true',help='exercise nested row scopes through actual dylib reloads')
    modes.add_argument('--live-each',action='store_true',help='exercise keyed live row order through actual dylib reloads')
    modes.add_argument('--choose',action='store_true',help='exercise memoized content/fallback branches through actual dylib reloads')
    modes.add_argument('--memo-when',action='store_true',help='exercise memoized branch preservation through actual dylib reloads')
    modes.add_argument('--live-subtree',action='store_true',help='exercise conditional typed live subtrees through actual dylib reloads')
    modes.add_argument('--typed-live',action='store_true',help='exercise typed text and style observers through actual dylib reloads')
    modes.add_argument('--rsx',action='store_true',help='exercise rstml RSX through actual dylib replacements')
    modes.add_argument('--large-ui',action='store_true',help='exercise single outer UI macros through actual dylib replacements')
    modes.add_argument('--trailing-ui',action='store_true',help='exercise automatic app UI blocks through actual dylib reloads')
    modes.add_argument('--tree-view',action='store_true',help='exercise struct-literal child arrays through actual dylib reloads')
    modes.add_argument('--live-text',action='store_true',help='exercise ordinary receiver text callbacks through actual dylib reloads')
    modes.add_argument('--suffix-text',action='store_true',help='exercise shared suffix bindings through actual reloads')
    modes.add_argument('--macro-errors',action='store_true',help='also verify actor macro failures and repaired dylib replacements')
    parser.add_argument('--wabou-native',action='store_true',help='build the plugin through a renamed wabou-native facade dependency')
    parser.add_argument('--idle-samples',type=int,default=0,help='also measure saves after the native watcher returns to idle polling')
    parser.add_argument('--idle-seconds',type=float,default=2.25,help='pause before each idle save; must exceed the two-second active polling window')
    parser.add_argument('--idle-cpu-seconds',type=float,default=0,help='optional additional pure-idle CPU sampling window after the active polling period')
    parser.add_argument("--sdk-root",type=pathlib.Path,help="verify implementation crates migrated into another checkout")
    parser.add_argument("--migration",action="store_true",help="also reject a compiled migration after a state write and prove rollback")
    args=parser.parse_args()
    if args.sdk_root:
        ROOT=args.sdk_root.resolve()
        import dev
        dev.ROOT=ROOT
    if args.migration and not args.trailing_ui:
        parser.error("--migration requires --trailing-ui")
    if args.large_ui or args.rsx: args.trailing_ui=True
    if args.async_join: args.async_value=True
    if args.idle_samples<0 or not args.idle_seconds>2 or not args.idle_cpu_seconds>=0:
        parser.error('idle samples/CPU seconds must be nonnegative and idle seconds must exceed 2')
    os.chdir(ROOT);cargo=cargo_tool()
    subprocess.run([cargo,'build','-p','blockui-runtime','--features','native'],env=ENV,check=True)
    with tempfile.TemporaryDirectory(prefix='identity-watch-') as temporary:
        root=pathlib.Path(temporary);app=root/'app';dep=root/'dependency';app.joinpath('src').mkdir(parents=True)
        manifest=f'''[package]
name = "watch-probe"
version = "0.1.0"
edition = "2024"
[lib]
crate-type = ["cdylib"]
[dependencies]
blockui = {{ path = {json.dumps(str(ROOT/'crates/blockui'))} }}
[workspace]
[profile.dev]
debug = 0
'''
        if args.macro_errors or args.live_text or args.tree_view or args.trailing_ui or args.typed_live or args.live_subtree or args.memo_when or args.choose or args.live_each or args.live_each_scoped or args.style_patch or args.async_value:
            manifest=manifest.replace('blockui = { path = ', 'blockui = { features = ["macros"], path = ')
        if args.rsx:
            manifest=manifest.replace('features = ["macros"]', 'features = ["macros", "rsx"]')
        source='''use blockui::*;
const COUNT: State<i64> = State::<i64>::new("count", 0);
const NOTES: State<String> = State::<String>::new("notes");
struct Probe;
impl Probe { const STEP: Action = Action::new("increment", |cx, _| COUNT.update(cx, |n| n.checked_add(1).ok_or(Error::Overflow))); }
impl Actor for Probe {
const STATE: &'static [StateSpec] = &[COUNT.spec(), NOTES.spec()];
const ACTIONS: &'static [Action] = &[Self::STEP];
fn view(_: &Cx<'_>) -> Result<View> { Ok({ let __blockui_tree = ::blockui::Ui::new(); {   { let _scope_0 = __blockui_tree.scope(Column { key: "root".into(), ..default() });
__blockui_tree.add(Text { key: "label".into(), text: bind!(|cx| Ok(COUNT.get(cx)?.to_string())) });
__blockui_tree.add(TextInput { read_only: false, key: "input".into(), ..TextInput::new(NOTES) });
}   } __blockui_tree.finish() }) }
}
export_actor!(Probe);
'''
        if args.suffix_text:
            old='text: bind!(|cx| Ok(COUNT.get(cx)?.to_string()))'
            assert source.count(old)==1
            source=source.replace(old,'text: COUNT.text_with_suffix(" items")')
        if args.live_text or args.tree_view:
            source='''use blockui::*;
#[actor(name = "watch-probe")]
struct Probe {
    #[state(key = "count")] count: State<i64>,
    #[state(key = "notes")] notes: State<String>,
}
#[actor_impl(render)]
impl Probe {
    #[action(bound, name = "increment")]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| n.checked_add(1).ok_or(Error::Overflow))
    }
}
impl Render for ProbeBound<'_, '_> {
    fn render(&self) -> View {
        Column::default().key("root").children(vec![
            Text::new(self.live_text(|this| this.count.get().to_string())).key("label").into_view(),
            TextInput::new(&self.notes).key("input").into_view(),
        ])
    }
}
export_actor!(Probe);
'''
        if args.tree_view:
            start=source.index('    fn render(&self) -> View {')
            end=source.index('\n}\nexport_actor!(Probe);',start)
            source=source[:start]+'''    #[blockui::view(tree)]
    fn render(&self) -> View {
        Column {
            key: "root".into(),
            children: [
                Text::new(live!(self.count.get().to_string())).key("label"),
                TextInput::new(&self.notes).key("input"),
            ],
            ..default()
        }
    }'''+source[end:]
        if args.trailing_ui or args.typed_live or args.live_subtree or args.memo_when or args.choose or args.live_each or args.live_each_scoped or args.style_patch or args.async_value:
            source='''use blockui::*;
#[app(name = "watch-probe")]
struct Probe {
    #[state(key = "count")] count: State<i64>,
    #[state(key = "notes")] notes: State<String>,
}
#[app_impl]
impl Probe {
    #[action(name = "increment")]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| n.checked_add(1).ok_or(Error::Overflow))
    }
    fn view(&self) -> View {
        #[ui(Column { key: "root".into(), ..default() })]
        {
            #[ui]
            Text::new(live!(self.count.get().to_string())).key("label");
            #[ui]
            TextInput::new(&self.notes).key("input");
        }
    }
}
export_actor!(Probe);
'''
        if args.trailing_ui:
            source=source.replace('    fn view(&self) -> View {', '    fn label(&self) -> View {\n        #[ui(Row::default())] {\n            #[ui] Text::new(live!(self.count.get().to_string())).key("label");\n        }\n    }\n    fn view(&self) -> View {',1)
            source=source.replace('            Text::new(live!(self.count.get().to_string())).key("label");', '            self.label();',1)
        if args.large_ui:
            source = """use blockui::*;
#[app(name = "watch-probe")]
struct Probe {
    #[state(key = "count")] count: State<i64>,
    #[state(key = "notes")] notes: State<String>,
}
#[app_impl]
impl Probe {
    #[action(name = "increment")]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| n.checked_add(1).ok_or(Error::Overflow))
    }
    fn label(&self) -> View {
        ui! {
            Row::default().children({
                Text::new(live!(self.count.get().to_string())).key("label");
            })
        }
    }
    fn view(&self) -> View {
        ui! {
            Column::default().key("root").children({
                self.label();
                TextInput::new(&self.notes).key("input");
            })
        }
    }
}
export_actor!(Probe);
"""
        if args.rsx:
            source = """use blockui::*;
#[app(name = "watch-probe")]
struct Probe {
    #[state(key = "count")] count: State<i64>,
    #[state(key = "notes")] notes: State<String>,
}
#[app_impl]
impl Probe {
    #[action(name = "increment")]
    fn increment(&self) -> Result<()> {
        self.count.update(|n| n.checked_add(1).ok_or(Error::Overflow))
    }
    fn label(&self) -> View {
        rsx! { <Row><Text text={live!(self.count.get().to_string())} key="label" /></Row> }
    }
    fn view(&self) -> View {
        rsx! {
            <Column key="root">
                {self.label()}
                {TextInput::new(&self.notes).key("input")}
            </Column>
        }
    }
}
export_actor!(Probe);
"""
        if args.typed_live:
            source=source.replace('Text::new(live!(self.count.get().to_string())).key("label");',
                'Text::new(live!(self.count.get().to_string())).style(live!(ViewStyle { width: 100.0 + self.count.get() as f32, ..default() })).key("label");')
        if args.live_subtree or args.memo_when or args.choose or args.live_each or args.live_each_scoped:
            source=source.replace('    fn view(&self) -> View {',
                '    #[view(block)]\n    fn details(&self) -> View {\n        #[ui(Column::default())] {\n            #[ui] Text::new("Details");\n            #[ui] TextInput::new(&self.notes).key("input");\n        }\n    }\n    fn view(&self) -> View {',1)
            source=source.replace('TextInput::new(&self.notes).key("input");\n        }\n    }\n}\nexport_actor!',
                'live!(if self.count.get() % 2 == 1 { self.details() } else { View::empty() }).mount("details");\n        }\n    }\n}\nexport_actor!',1)
        if args.memo_when:
            source=source.replace('    #[view(block)]', '    #[computed]\n    fn visible(&self) -> bool { self.count.get() > 0 }\n    #[view(block)]',1)
            source=source.replace('live!(if self.count.get() % 2 == 1 { self.details() } else { View::empty() })',
                'self.visible_computed().when(live!(self.details()))',1)
        if args.choose:
            source=source.replace('    #[view(block)]', '    #[computed]\n    fn visible(&self) -> bool { self.count.get() % 2 == 1 }\n    #[view(block)]',1)
            source=source.replace('live!(if self.count.get() % 2 == 1 { self.details() } else { View::empty() })',
                'self.visible_computed().choose(live!(self.details()), live!(Text::new("Fallback")))',1)
        if args.live_each or args.live_each_scoped:
            start=source.index('    #[view(block)]')
            end=source.index('    fn view(&self) -> View {',start)
            source=source[:start]+'    #[computed]\n    fn keys(&self) -> Vec<String> {\n        if self.count.get() % 2 == 1 { vec!["a".into(), "b".into()] } else { vec!["b".into(), "a".into()] }\n    }\n    #[view(block)]\n    fn row(&self, key: &str) -> View {\n        #[ui(Column::default())] {\n            #[ui] Text::new(key).key("name");\n            #[ui] TextInput::new(&self.notes).key("input");\n        }\n    }\n'+source[end:]
            source=source.replace('live!(if self.count.get() % 2 == 1 { self.details() } else { View::empty() }).mount("details")',
                'self.keys_computed().each(self.live_each(|this, key| this.row(&key))).mount("rows")',1)
        if args.live_each_scoped:
            source=source.replace('.each(self.live_each(', '.each_scoped(self.live_each(',1)
            source=source.replace('Text::new(key).key("name");', 'Text::new(key).key("name");\n            #[ui] Text::new(self.count.get().to_string()).key("structural");',1)
        if args.style_patch:
            source=source.replace('Text::new(live!(self.count.get().to_string())).key("label");',
                'Text::new(live!(self.count.get().to_string())).style((ViewStyle { width: 777.0, text_tone: TextTone::Accent, ..default() }, live!(StylePatch { width: Some(100.0 + self.count.get() as f32), ..default() }), StylePatch { max_width: Some(800.0), ..default() })).key("label");')
        if args.async_value:
            source=source.replace('use blockui::*;', 'use blockui::*;\n#[derive(Record, Default, Clone)]\n#[record(name = "watch.async.data")]\nstruct Data { text: String }',1)
            source=source.replace('    #[state(key = "notes")] notes: State<String>,', '    #[state(key = "notes")] notes: State<String>,\n    value: State<AsyncValue<Data>>,\n    job: Task,',1)
            source=source.replace('self.count.update(|n| n.checked_add(1).ok_or(Error::Overflow))', 'self.count.update(|n| n.checked_add(1).ok_or(Error::Overflow))?;\n        let mut value = self.value.get()?;\n        let token = value.begin()?;\n        self.value.set(&value)?;\n        self.job.cancel()?;\n        self.job.delay(std::time::Duration::from_millis(10), self.complete_action(), token)',1)
            source=source.replace('    fn view(&self) -> View {', '    #[action]\n    fn complete(&self, token: i64) -> Result<()> {\n        self.value.update(|value| { value.resolve(token, Data { text: token.to_string() }); Ok(()) })\n    }\n    fn view(&self) -> View {',1)
            source=source.replace('            TextInput::new(&self.notes).key("input");', '            TextInput::new(&self.notes).key("input");\n            #[ui] Text::new(live!({ let value = self.value.get().unwrap(); value.latest().map(|data| data.text.clone()).unwrap_or_default() })).key("async.result");\n            #[ui] Text::new(live!(self.value.get().unwrap().generation().to_string())).key("async.generation");',1)
        if args.async_value:
            source=source.replace('    fn view(&self) -> View {', '    #[computed(record)]\n    fn derived(&self) -> Result<AsyncResult<Data>> { Ok(self.value.get()?.into_read()) }\n    #[computed(record)]\n    fn forwarded(&self) -> Result<AsyncResult<Data>> { Ok(self.derived()?.map(|data| Data { text: data.text })) }\n    fn view(&self) -> View {',1)
            source=source.replace('            TextInput::new(&self.notes).key("input");', '            TextInput::new(&self.notes).key("input");\n            #[ui] live!({ let value = self.forwarded().unwrap(); value.map(|data| Text::new(format!("Async boundary {}", data.text))) }).boundary(|_| Text::new("Async unavailable")).mount("async.boundary");',1)
        if args.async_join:
            source=source.replace('    job: Task,', '    job: Task,\n    other: State<AsyncValue<Data>>,\n    other_job: Task,',1)
            old='self.job.delay(std::time::Duration::from_millis(10), self.complete_action(), token)'
            assert old in source
            source=source.replace(old, old+'?;\n        let mut other = self.other.get()?;\n        let other_token = other.begin()?;\n        self.other.set(&other)?;\n        self.other_job.cancel()?;\n        self.other_job.delay(std::time::Duration::from_millis(30), self.complete_other_action(), other_token)',1)
            source=source.replace('    fn view(&self) -> View {', '    #[action]\n    fn complete_other(&self, token: i64) -> Result<()> {\n        self.other.update(|value| { value.resolve(token, Data { text: token.to_string() }); Ok(()) })\n    }\n    fn view(&self) -> View {',1)
            old='self.derived()?.map(|data| Data { text: data.text })'
            assert old in source
            source=source.replace(old, 'self.derived()?.join(self.other.get()?.into_read()).map(|(data, other)| Data { text: format!("{}:{}", data.text, other.text) })',1)
        if args.wabou_native:
            manifest=manifest.replace('blockui = {', 'native = { package = "wabou-native",')
            manifest=manifest.replace(json.dumps(str(ROOT/'crates/blockui')), json.dumps(str(ROOT/'crates/wabou-native')))
            source=source.replace('blockui::', 'native::')
        app.joinpath('Cargo.toml').write_text(manifest);app.joinpath('src/lib.rs').write_text(source)
        # A broken auxiliary binary must not participate in plugin-only reloads.
        app.joinpath('src/main.rs').write_text('compile_error!("auxiliary binary must not block UI reload");\nfn main() {}\n')
        config=configuration(cargo,app/'Cargo.toml','watch-probe','')
        subprocess.run([cargo,'build','--lib','--manifest-path',str(app/'Cargo.toml'),'--target-dir',str(ROOT/'target')],env=ENV,check=True)
        wrapper=root/'cargo-wrapper'; fail_marker=root/'fail-build'; cargo_calls=root/'cargo-calls.jsonl'
        wrapper.write_text('#!/usr/bin/env python3\nimport json,os,pathlib,sys\n'
            + 'with open(' + repr(str(cargo_calls)) + ', "a") as log: log.write(json.dumps(sys.argv[1:])+"\\n")\n'
            + 'if len(sys.argv)>1 and sys.argv[1]=="rustc" and pathlib.Path(' + repr(str(fail_marker)) + ').exists():\n'
            + ' print("wrapper: 编译器暂时不可用", flush=True)\n sys.exit(19)\n'
            + 'os.execv(' + repr(shutil.which(cargo) or cargo) + ', [' + repr(shutil.which(cargo) or cargo) + ', *sys.argv[1:]])\n')
        wrapper.chmod(0o755)
        log=root/'watch.err';lines=queue.Queue();events=[];timings={}
        with log.open('w') as stderr:
            process=subprocess.Popen([str(ROOT/'target/debug/blockui-runtime'),'watch-probe',config['BLOCKUI_DEV_ARTIFACT']],env=dict(dict(ENV,**config),BLOCKUI_CARGO=str(wrapper)),stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
            def collect():
                for line in process.stdout: lines.put(line.rstrip())
                lines.put(None)
            threading.Thread(target=collect,daemon=True).start()
            def receive(prefix,timeout=15):
                deadline=time.monotonic()+timeout
                while time.monotonic()<deadline:
                    try: line=lines.get(timeout=max(.01,deadline-time.monotonic()))
                    except queue.Empty: raise TimeoutError(log.read_text())
                    if line is None: raise RuntimeError(log.read_text())
                    if line.startswith('EVENT '):
                        event=json.loads(line[6:]);events.append(event)
                        if prefix=='EVENT': return event
                        raise AssertionError(f'unexpected build: {event}')
                    if line.startswith(prefix): return json.loads(line.split(' ',1)[1]) if prefix=='STATE' else line
                raise TimeoutError(log.read_text())
            last_scope_counts={}
            def command(value):
                process.stdin.write(value+'\n');process.stdin.flush();state=receive('STATE')
                if args.suffix_text: assert state['label']==str(state['count'])+' items',state
                if args.live_text or args.tree_view or args.trailing_ui or args.typed_live or args.live_subtree or args.memo_when or args.choose or args.live_each or args.live_each_scoped or args.style_patch or args.async_value: assert state['label']==str(state['count']),state
                if args.live_subtree: assert state['details_visible']==(state['count']%2==1),state
                if args.live_each or args.live_each_scoped: assert state['row_order']==(['a','b'] if state['count']%2==1 else ['b','a']),state
                if args.choose: assert state['fallback_visible']==(state['count']%2==0),state
                if args.memo_when:
                    assert state['details_visible'],state
                    generation=(state['implementation'],state['sequence'])
                    assert state['details_evaluations']==last_scope_counts.setdefault(generation,state['details_evaluations']),state
                if args.style_patch:
                    assert state['label_width']==100.0+state['count'],state
                    assert state['label_tone']==2 and state['label_max_width']==800.0,state
                if args.typed_live: assert state['label_width']==100.0+state['count'],state
                if args.async_value and value=='step':
                    deadline=time.monotonic()+5
                    while state['async_text']!=state['async_generation'] or state['async_boundary']!='Async boundary '+state['async_generation']+(':'+state['async_generation'] if args.async_join else ''):
                        assert time.monotonic()<deadline,state
                        time.sleep(.02)
                        state=command('status')
                return state
            try:
                receive('READY');initial=command('status');assert initial['count']==1
                assert command('step')['count']==2
                if args.typed_live:
                    captured=source.replace('fn view(&self) -> View {', 'fn view(&self) -> View { let caption = String::from("Count");')
                    captured=captured.replace('width: 100.0 + self.count.get() as f32', 'width: caption.len() as f32')
                    assert captured != source
                    app.joinpath('src/lib.rs').write_text(captured)
                    event=receive('EVENT')
                    assert event['error'] and event['implementation']==initial['implementation'],event
                    assert 'captur' in event['error'].lower(),event
                    assert command('status')['count']==2
                if args.live_text:
                    captured=source.replace('fn render(&self) -> View {', 'fn render(&self) -> View { let caption = String::from("Count");')
                    captured=captured.replace('this.count.get().to_string()', 'format!("{caption}: {}", this.count.get())')
                    assert captured != source
                    app.joinpath('src/lib.rs').write_text(captured)
                    event=receive('EVENT')
                    assert event['error'] and event['implementation']==initial['implementation'],event
                    assert 'captur' in event['error'].lower(),event
                    assert command('status')['count']==2
                fail_marker.write_text('fail')
                app.joinpath('src/lib.rs').write_text(source+'\n// wrapper diagnostic probe\n')
                event=receive('EVENT')
                assert event['error'] and event['implementation']==initial['implementation'],event
                assert 'wrapper: 编译器暂时不可用' in event['error'] and '19' in event['error'],event
                assert command('status')['count']==2
                fail_marker.unlink()
                app.joinpath('src/lib.rs').write_text(source+'\nfn broken() { let missing: = ; }\n')
                event=receive('EVENT');assert event['error'] and event['implementation']==initial['implementation']
                assert 'Cargo build failed (' in event['error'],event
                state=command('step');assert state['count']==3 and state['implementation']==initial['implementation']
                app.joinpath('Cargo.toml').write_text(manifest+'\n[broken')
                event=receive('EVENT');assert event['error'] and event['implementation']==initial['implementation']
                assert 'Cargo metadata failed (' in event['error'],event
                state=command('step');assert state['count']==4 and state['implementation']==initial['implementation']
                added=manifest.replace('[workspace]',f'local-step = {{ path = {json.dumps(str(dep))} }}\n[workspace]')
                app.joinpath('Cargo.toml').write_text(added);app.joinpath('src/lib.rs').write_text(source.replace('checked_add(1)','checked_add(local_step::VALUE)'))
                event=receive('EVENT');assert event['error'] and event['implementation']==initial['implementation']
                assert 'Cargo metadata failed (' in event['error'],event
                state=command('step');assert state['count']==5 and state['implementation']==initial['implementation']
                dep.joinpath('src').mkdir(parents=True);dep.joinpath('Cargo.toml').write_text('[package]\nname="local-step"\nversion="0.1.0"\nedition="2024"\n')
                dep.joinpath('src/lib.rs').write_text('pub const VALUE: i64 = 2;\n')
                start=time.monotonic();event=receive('EVENT');assert event['error'] is None,event;timings['new_dependency_ms']=round((time.monotonic()-start)*1000,1)
                assert command('step')['count']==7
                renamed=added.replace('[lib]','[lib]\nname="renamed_probe"')
                app.joinpath('Cargo.toml').write_text(renamed);event=receive('EVENT');assert event['error'] is None
                dep.joinpath('src/lib.rs').write_text('pub const VALUE: i64 = 5;\n');start=time.monotonic();event=receive('EVENT');assert event['error'] is None;timings['dependency_edit_ms']=round((time.monotonic()-start)*1000,1)
                assert command('step')['count']==12
                # Source outside metadata roots is reached only through this link.
                external=root/'external';external.mkdir()
                first=external/'first.rs';first.write_text('pub const VALUE: i64 = 6;\n')
                linked=dep/'src/lib.rs';linked.unlink();linked.symlink_to(first)
                event=receive('EVENT');assert event['error'] is None
                assert command('step')['count']==18
                first.write_text('pub const VALUE: i64 = 7;\n')
                event=receive('EVENT');assert event['error'] is None
                assert command('step')['count']==25
                second=external/'second.rs';second.write_text('pub const VALUE: i64 = 8;\n')
                replacement=dep/'src/replacement';replacement.symlink_to(second);replacement.replace(linked)
                event=receive('EVENT');assert event['error'] is None
                assert command('step')['count']==33
                app.joinpath('Cargo.toml').write_text(manifest.replace('[lib]','[lib]\nname="renamed_probe"'));app.joinpath('src/lib.rs').write_text(source)
                event=receive('EVENT');assert event['error'] is None
                assert command('step')['count']==34
                sequence=event['sequence'];dep.joinpath('src/lib.rs').write_text('pub const VALUE: i64 = 999;\n')
                time.sleep(.7)
                state=command('status');assert state['sequence']==sequence and state['count']==34
                # A module absent at watcher startup must become visible later.
                before_module = state['implementation']
                modular = 'mod behavior;\n' + source.replace('checked_add(1)', 'checked_add(behavior::VALUE)')
                app.joinpath('src/lib.rs').write_text(modular)
                event=receive('EVENT');assert event['error'] and event['implementation']==before_module
                assert command('step')['count']==35  # Last accepted implementation remains live.
                behavior=app/'src/behavior/mod.rs';behavior.parent.mkdir()
                behavior.write_text('pub const VALUE: i64 = 3;\n')
                event=receive('EVENT');assert event['error'] is None
                assert command('step')['count']==38
                module_implementation=event['implementation']
                assert module_implementation!=before_module
                behavior.unlink()
                event=receive('EVENT');assert event['error'] and event['implementation']==module_implementation
                assert command('step')['count']==41  # Deletion cannot discard state/code.
                behavior.write_text('pub const VALUE: i64 = 4;\n')
                event=receive('EVENT');assert event['error'] is None
                assert command('step')['count']==45
                app.joinpath('src/lib.rs').write_text(source)
                event=receive('EVENT');assert event['error'] is None
                assert command('step')['count']==46
                macro_recovery=[]
                expected_count=46
                if args.macro_errors:
                    for role, output, body in [('scope','View','View::empty()'), ('computed','i64','1'), ('action','Result<()>','Ok(())'), ('multiple','View','View::empty()')]:
                        annotation='scope' if role=='multiple' else role
                        extra = (
                            '\n#[actor] struct MacroProbe { value: State<i64> }\n'
                            '#[actor_impl] impl MacroProbe {\n'
                            "fn view(&self, cx: &Cx<'_>) -> View { View::empty() }\n"
                            f'#[{annotation}] fn broken(&self) -> {output} {{ {body} }}\n'
                            "#[scope] fn working(&self, cx: &Cx<'_>) -> View { View::empty() }\n}\n"
                        )
                        if role=='multiple':
                            extra=extra.replace('#[scope] fn working', '#[computed] fn another(&self) -> i64 { 1 }\n#[action] fn third(&self) -> Result<()> { Ok(()) }\n#[scope] fn working')
                        accepted=command('status')['implementation']
                        app.joinpath('src/lib.rs').write_text(source+extra)
                        event=receive('EVENT')
                        assert event['error'] and event['implementation']==accepted,event
                        required=['scope','computed','action'] if role=='multiple' else [role]
                        assert all(category+' requires' in event['error'] for category in required),event
                        expected_count+=1
                        state=command('step')
                        assert state['implementation']==accepted and state['count']==expected_count,state
                        fixed=extra.replace('(&self)', "(&self, cx: &Cx<'_>)")
                        repaired=source.replace('COUNT.get(cx)?.to_string()', f'format!("{role}: {{}}", COUNT.get(cx)?)')
                        app.joinpath('src/lib.rs').write_text(repaired+fixed)
                        event=receive('EVENT')
                        assert event['error'] is None and event['implementation']!=accepted,event
                        state=command('status')
                        assert state['count']==expected_count,state
                        snapshots=list(ROOT.glob(f'.runtime/actor-42-{process.pid}/impl-*'))
                        assert len(snapshots)==1,snapshots
                        macro_recovery.append(dict(role=role,failed_build_preserved_implementation=True,old_action_processed=True,repaired_dylib_accepted=True,state_retained=True,retained_generations=len(snapshots)))
                    app.joinpath('src/lib.rs').write_text(source)
                    event=receive('EVENT');assert event['error'] is None,event
                    assert command('status')['count']==expected_count
                # Measure from before the save through the accepted reload EVENT,
                # including detection/debounce, Cargo and the actor implementation swap.
                warm_samples=[]; idle_samples=[]; idle_cpu_samples=[]; runtime_samples=[]
                def process_cpu_seconds():
                    # Linux-only optional CPU evidence, never inferred on other hosts.
                    try:
                        fields=pathlib.Path(f'/proc/{process.pid}/stat').read_text().rsplit(') ',1)[1].split()
                        return (int(fields[11])+int(fields[12]))/os.sysconf('SC_CLK_TCK')
                    except (OSError,ValueError,IndexError,AttributeError):
                        return None
                target_path=(ROOT/'target').resolve()
                target_before=disk(target_path)
                previous=command('status')['implementation']
                calls_before_warm=len(cargo_calls.read_text().splitlines())
                warm_cpu_before=process_cpu_seconds()
                warm_cpu_after=None
                for step in range(2, 10+args.idle_samples):
                    if step>=10:
                        time.sleep(args.idle_seconds)
                        if args.idle_cpu_seconds:
                            cpu_before=process_cpu_seconds()
                            time.sleep(args.idle_cpu_seconds)
                            cpu_after=process_cpu_seconds()
                            if cpu_before is not None and cpu_after is not None:
                                idle_cpu_samples.append(round((cpu_after-cpu_before)*1000,1))
                    started=time.monotonic()
                    app.joinpath('src/lib.rs').write_text(source.replace('checked_add(1)',f'checked_add({step})'))
                    event=receive('EVENT');elapsed=(time.monotonic()-started)*1000
                    if step==9: warm_cpu_after=process_cpu_seconds()
                    assert event['error'] is None and event['implementation']!=previous,event
                    previous=event['implementation']
                    expected_count+=step
                    assert command('step')['count']==expected_count
                    snapshots=list(ROOT.glob(f'.runtime/actor-42-{process.pid}/impl-*'))
                    assert len(snapshots)==1,snapshots
                    maps=pathlib.Path(f'/proc/{process.pid}/maps').read_text()
                    mapped={line.split(maxsplit=5)[5] for line in maps.splitlines() if f'actor-42-{process.pid}/impl-' in line}
                    assert len(mapped)==1,mapped
                    (warm_samples if step<10 else idle_samples).append(round(elapsed,1))
                    runtime_samples.append(disk(ROOT/'.runtime')['allocated_bytes'])
                warm_calls=[json.loads(line) for line in cargo_calls.read_text().splitlines()[calls_before_warm:]]
                assert len(warm_calls)==8+args.idle_samples,warm_calls
                assert all(call[0]=='rustc' and '--lib' in call and '--crate-type' in call and call[call.index('--crate-type')+1]=='cdylib' for call in warm_calls),warm_calls
                timings['warm_cargo_operations']={'rustc':len(warm_calls),'metadata':0}
                if warm_cpu_before is not None and warm_cpu_after is not None:
                    timings['warm_process_cpu_ms']=round((warm_cpu_after-warm_cpu_before)*1000,1)
                    timings['warm_cpu_note']='Linux host process CPU ticks across warm edits; excludes Cargo/compiler child CPU and is limited by scheduler tick resolution'
                timings['warm_save_to_reload_ms']=warm_samples
                timings['warm_median_ms']=round(statistics.median(warm_samples),1)
                timings['warm_max_ms']=max(warm_samples)
                if idle_samples:
                    timings['idle_pause_seconds']=args.idle_seconds
                    timings['idle_save_to_reload_ms']=idle_samples
                    timings['idle_median_ms']=round(statistics.median(idle_samples),1)
                    timings['idle_max_ms']=max(idle_samples)
                    timings['idle_process_cpu_ms']=idle_cpu_samples
                    timings['idle_cpu_window_seconds']=args.idle_cpu_seconds
                    timings['idle_cpu_note']='Linux process CPU ticks after the active period, including all host work; limited by clock-tick resolution, not CPU usage of a running GUI'
                warm_disk=dict(target_before=target_before,target_after=disk(target_path),target_location=dict(path=str(target_path),shared=(ROOT/'target').is_symlink()),runtime_allocated_samples=runtime_samples,live_plugin_mappings=1,retained_plugin_snapshots=1)
                if args.migration:
                    before=command('status')
                    candidate=source.replace('    fn view(&self) -> View {',
                        "    fn migrate(&self, cx: &Cx<'_>, previous_schema: u64) -> Result<()> {\n"
                        '        let _ = previous_schema;\n'
                        '        State::<i64>::new("count", 0).set(cx, 999)?;\n'
                        '        Err(Error::Application("migration rollback probe".into()))\n'
                        '    }\n    fn view(&self) -> View {',1)
                    candidate=candidate.replace('#[state(key = "notes")] notes: State<String>,',
                        '#[state(key = "notes")] notes: State<String>,\n    #[state(key = "migration.probe")] migration_probe: State<i64>,',1)
                    assert candidate!=source
                    app.joinpath('src/lib.rs').write_text(candidate)
                    event=receive('EVENT')
                    assert event['error'] and 'migration rollback probe' in event['error'],event
                    assert event['implementation']==before['implementation'],event
                    state=command('status')
                    assert state['count']==before['count'],state
                    state=command('step')
                    assert state['implementation']==before['implementation'] and state['count']==before['count']+step,state
                    app.joinpath('src/lib.rs').write_text(source)
                    event=receive('EVENT');assert event['error'] is None,event
                    assert command('status')['count']==before['count']+step
                    snapshots=list(ROOT.glob(f'.runtime/actor-42-{process.pid}/impl-*'))
                    assert len(snapshots)==1,snapshots
                process.stdin.write('quit\n');process.stdin.flush();receive('STOPPED');process.wait(timeout=5);assert process.returncode==0,log.read_text()
            finally:
                if process.poll() is None:
                    process.stdin.write('quit\n');process.stdin.flush()
                    try: process.wait(timeout=5)
                    except subprocess.TimeoutExpired: process.kill();process.wait()
        report=dict(migration_write_rollback=args.migration,wabou_native_facade=args.wabou_native,rsx_blocks=args.rsx,large_ui_blocks=args.large_ui,independent_async_join=args.async_join,async_result_chain=args.async_value,record_computed_chain=args.async_value,host_async_value=args.async_value,ordered_style_patch=args.style_patch,scoped_live_each=args.live_each_scoped,keyed_live_each=args.live_each,memoized_choose_fallback=args.choose,memoized_when_branch=args.memo_when,conditional_live_subtree=args.live_subtree,typed_live_text_and_style=args.typed_live,typed_live_capture_rejected=args.typed_live,trailing_ui_blocks=args.trailing_ui,struct_literal_tree=args.tree_view,receiver_live_text=args.live_text,implicit_live_capture_rejected_without_replacing_actor=args.live_text,shared_suffix_binding=args.suffix_text,fixture_disk=disk(root),macro_error_recovery=macro_recovery,events=events,timings=timings,warm_disk=warm_disk,measurement_scope='headless real watcher save-to-accepted-reload; excludes native layout/render/presentation',diagnostics=log.read_text(),preserved_state=True,stdout_only_wrapper_error_preserves_actor_and_recovers=True,failed_commands_include_operation_and_status=True,auxiliary_binary_compile_error_does_not_block_plugin_reload=True,old_code_handles_messages_after_source_manifest_and_dependency_errors=True,new_dependency=True,removed_dependency_ignored=True,symlink_content_and_retarget=True,target_rename=True,missing_path_retried=True,new_module_creation_deletion_recovery=True)
    report['fixture_removed']=not root.exists()
    assert report['fixture_removed']
    assert not list(ROOT.glob('.runtime/actor-*'))
    assert not list(ROOT.glob('.runtime/watch-log-*'))
    ROOT.joinpath('reports').mkdir(exist_ok=True);ROOT.joinpath(('reports/wabou-native-async-watcher.json' if args.async_value else 'reports/wabou-native-watcher.json') if args.wabou_native else 'reports/watcher.json').write_text(json.dumps(report,indent=2)+'\n')
    print('PASS real watcher detects new/deleted/restored modules while preserving state and accepted code, follows linked source edits/retargeting, adds/edits/removes local dependencies, retries missing paths, updates renamed cdylib, executes old code after source/manifest/dependency build errors, retains state through recovery and removes logs')
    print(json.dumps(timings))
if __name__=='__main__':main()
