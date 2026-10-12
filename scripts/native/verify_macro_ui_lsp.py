#!/usr/bin/env python3
"""Actual rust-analyzer JSON-RPC field completion/definition for actor declarations."""
import json,os,pathlib,queue,subprocess,sys,threading,time
from dev import ROOT, ENV, cargo_tool
CARGO = cargo_tool()

def main():
    os.chdir(ROOT);cargo=CARGO
    large="--large" in sys.argv
    braces="--braces" in sys.argv
    scenarios="--scenarios" in sys.argv
    rsx="--rsx" in sys.argv
    external=pathlib.Path(sys.argv[sys.argv.index("--project")+1]).resolve() if "--project" in sys.argv else None
    tags="--tags" in sys.argv or external is not None
    fixture="external-tags" if external else "tag_helpers" if tags else "rsx_ui" if rsx else "large_ui_scenarios" if scenarios else "large_ui" if large else "macro_ui"
    build=[cargo,'test','--lib','--manifest-path',str(external/'Cargo.toml'),'--target-dir',str(ROOT/'target'),'--no-run'] if external else [cargo,'test','-p','blockui-test','--test',fixture,'--no-run']
    subprocess.run(build,env=ENV,check=True)
    binary=subprocess.check_output(['rustup','which','rust-analyzer'],env=ENV,text=True).strip()
    messages=queue.Queue();ROOT.joinpath('reports').mkdir(exist_ok=True)
    with ROOT.joinpath(f'reports/{fixture}-lsp.stderr').open('w') as stderr:
        process=subprocess.Popen([binary],env=ENV,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr)
        def reader():
            while True:
                headers={}
                while True:
                    line=process.stdout.readline()
                    if not line: messages.put(None);return
                    if line==b'\r\n': break
                    key,value=line.decode().split(':',1);headers[key.lower()]=value.strip()
                messages.put(json.loads(process.stdout.read(int(headers['content-length']))))
        threading.Thread(target=reader,daemon=True).start();identifier=0
        def send(method,params=None,request=False):
            nonlocal identifier
            payload=dict(jsonrpc='2.0',method=method,params=params)
            if request: identifier+=1;payload['id']=identifier
            data=json.dumps(payload).encode();process.stdin.write(f'Content-Length: {len(data)}\r\n\r\n'.encode()+data);process.stdin.flush();return identifier
        def response(identity,timeout=45):
            deadline=time.monotonic()+timeout
            while time.monotonic()<deadline:
                value=messages.get(timeout=max(.01,deadline-time.monotonic()))
                if value is None: raise RuntimeError(ROOT.joinpath(f'reports/{fixture}-lsp.stderr').read_text())
                if value.get('id')==identity and 'method' not in value:
                    if 'error' in value: raise RuntimeError(value['error'])
                    return value.get('result')
                if 'id' in value and 'method' in value:
                    result=[] if value['method']=='workspace/configuration' else None
                    data=json.dumps(dict(jsonrpc='2.0',id=value['id'],result=result)).encode();process.stdin.write(f'Content-Length: {len(data)}\r\n\r\n'.encode()+data);process.stdin.flush()
            raise TimeoutError('rust-analyzer request')
        try:
            response(send('initialize',dict(processId=os.getpid(),rootUri=(external or ROOT).as_uri(),capabilities={},initializationOptions={'cargo':{'targetDir':str(ROOT/'target')},'procMacro':{'enable':True}}),True))
            send('initialized',{})
            path=external/'src/lib.rs' if external else ROOT/f'native/blockui-test/tests/{fixture}.rs';uri=path.as_uri();original=path.read_text()
            if braces:
                original=original.replace('ui!(Column::default().children({','ui! { Column::default().children({').replace('        }))','        }) }')
            report={}
            cases=[
                ('receiver','self.section("Helper title")','self.','ui!(self.',len('ui!(self.'),'section'),
                ('live_state','self.count.get().to_string()','self.count.','live!(self.count.',len('live!(self.count.'),'get'),
                ('constructor','Text::new("Static caption")','Text::','ui!(Text::',len('ui!(Text::'),'new'),
                ('parent','Column::default()','Column::','ui!(Column::',len('ui!(Column::'),'default'),
                ('builder','Text::new("Static caption")','Text::new("Static caption").','ui!(Text::new("Static caption").',len('ui!(Text::new("Static caption").'),'style'),
            ]
            if large:
                cases=[
                    ('receiver','self.section("Helper title")','self.','',0,'section'),
                    ('live_state','self.count.get().to_string()','self.count.','',0,'get'),
                    ('constructor','Text::new(caption_text)','Text::','',0,'new'),
                    ('parent','Column::default()','Column::','',0,'default'),
                    ('builder','Text::new(caption_text)','Text::new(caption_text).','',0,'style'),
                    ('local','Text::new(caption_text)','Text::new(caption_te)','',0,'caption_text'),
                ]
            if scenarios:
                cases=[
                    ('style_signal','self.left.get() as f32','self.left.','',0,'get'),
                    ('list_renderer','this.row(&key)','this.','',0,'row'),
                    ('list_method','.each(self.live_each(','.ea','',0,'each'),
                    ('component','self.left.render()','self.left.','',0,'render'),
                    ('async_boundary','.boundary(async_fallback)','.bou','',0,'boundary'),
                    ('fallible_builder','Text::new(label()?)','Text::new(label()?).','',0,'style'),
                ]
            if rsx:
                cases=[
                    ('signal','self.count.get().to_string()','self.count.','',0,'get'),
                    ('event','self.increment_action()','self.incre','',0,'increment_action'),
                    ('branch_receiver','self.details()','self.','',0,'details'),
                    ('rust_child','{footer()}','{Text::}','',-1,'new'),
                    ('rust_child_prefix','{footer()}','{Text::ne}','',-1,'new'),
                    ('rust_child_builder','{footer()}','{Text::new("Probe").}','',-1,'style'),
                    ('bare_prop','gap={8.0_f32}','ga','',0,'gap'),
                    ('tag','<Column gap','<Col gap','',-4,'Column'),
                    ('closing_tag','</Column>','</Col>','',-1,'Column'),
                    ('prop','gap={8.0_f32}','ga={8.0_f32}','',-10,'gap'),
                    ('custom_prop','title="Custom component"','tit="Custom component"','',-19,'title'),
                    ('style_prop','style={ViewStyle','sty={ViewStyle','',-11,'style'),
                    ('missing_close','</Column>','','',0,'get'),
                ]
            if tags:
                cases=[
                    ('receiver','self.section("Helper title")','self.','',0,'section'),
                    ('live_state','self.count.get().to_string()','self.count.','',0,'get'),
                    ('constructor','Text::new(title)','Text::','',0,'new'),
                    ('parent','Column::default()','Column::','',0,'default'),
                    ('builder','Text::new(title)','Text::new(title).','',0,'style'),
                    ('event','self.increment_action()','self.incre','',0,'increment_action'),
                    ('style_field','Text::new(title)','Text::new(title).style(ViewStyle { wid })','',-3,'width'),
                ]
            if external and "--legacy-tags" not in sys.argv:
                cases=[
                    ('receiver','self.details()','self.','',0,'details'),
                    ('live_state','self.count.get()','self.count.','',0,'get'),
                    ('constructor','Text::new("Details")','Text::','',0,'new'),
                    ('parent','Column::default()','Column::','',0,'default'),
                    ('builder','Text::new("Details")','Text::new("Details").','',0,'style'),
                    ('event','self.increment_action()','self.incre','',0,'increment_action'),
                    ('style_field','Text::new("Details")','Text::new("Details").style(ViewStyle { wid })','',-3,'width'),
                ]
            for version,(name,old,new,needle,offset,required) in enumerate(cases,1):
                text=original.replace(old,new,1)
                cursor=original.index(old)+len(new)
                if name=='local':cursor-=1
                if rsx or tags:
                    cursor+=offset
                    if name=='missing_close':
                        text=text.replace('self.count.get().to_string()', 'self.count.',1)
                        cursor=text.index('self.count.')+len('self.count.')
                send('textDocument/didOpen',dict(textDocument=dict(uri=uri,languageId='rust',version=version,text=text)))
                prefix=text[:cursor]
                position=dict(line=prefix.count('\n'),character=len(prefix.rsplit('\n',1)[-1]))
                labels=[];deadline=time.monotonic()+(15 if rsx else 45)
                while time.monotonic()<deadline:
                    try:
                        result=response(send('textDocument/completion',dict(textDocument=dict(uri=uri),position=position,context=dict(triggerKind=1)),True))
                    except RuntimeError as error:
                        if isinstance(error.args[0],dict) and error.args[0].get('code')==-32801:continue
                        raise
                    items=result.get('items',[]) if isinstance(result,dict) else result or []
                    labels=[item['label'] for item in items]
                    if any(label.split('(')[0]==required for label in labels):break
                    time.sleep(.1)
                supported=any(label.split('(')[0]==required for label in labels)
                report[name]=dict(supported=supported,completion=labels,position=position,source_before_cursor=prefix[-100:])
                print(name, 'PASS' if supported else 'FAIL',flush=True)
                send('textDocument/didClose',dict(textDocument=dict(uri=uri)))
            if large or rsx or tags:
                send('textDocument/didOpen',dict(textDocument=dict(uri=uri,languageId='rust',version=100,text=original)))
                navigation=[('signal_navigation','self.count.get()' if external and '--legacy-tags' not in sys.argv else 'self.count.get().to_string()','count: State<i64>')]
                if large:navigation.insert(0,('local_navigation','Text::new(caption_text)','let caption_text'))
                if rsx:navigation.append(('tag_navigation','<CustomPanel title','struct CustomPanel'))
                for name,needle,declaration in navigation:
                    offset=len('Text::new(') if name.startswith('local') else 1 if name=='tag_navigation' else len('self.')
                    prefix=original[:original.index(needle)+offset]
                    position=dict(line=prefix.count('\n'),character=len(prefix.rsplit('\n',1)[-1]))
                    expected=original[:original.index(declaration)].count('\n')
                    deadline=time.monotonic()+45
                    while True:
                        try:
                            result=response(send('textDocument/definition',dict(textDocument=dict(uri=uri),position=position),True))
                        except RuntimeError as error:
                            if isinstance(error.args[0],dict) and error.args[0].get('code')==-32801 and time.monotonic()<deadline:
                                time.sleep(.1)
                                continue
                            raise
                        locations=result if isinstance(result,list) else [result]
                        supported=any(loc and (loc.get('uri') or loc.get('targetUri'))==uri and
                            (loc.get('range') or loc.get('targetSelectionRange'))['start']['line']==expected for loc in locations)
                        if supported or time.monotonic()>=deadline:break
                        time.sleep(.1)
                    report[name]=dict(supported=supported,definition=result)
                    print(name,'PASS' if supported else 'FAIL',flush=True)
                send('textDocument/didClose',dict(textDocument=dict(uri=uri)))
            ROOT.joinpath('reports/external-tag-ui-lsp.json' if external else 'reports/tag-ui-lsp.json' if tags else 'reports/rsx-ui-lsp.json' if rsx else 'reports/large-ui-scenarios-lsp.json' if scenarios else ('reports/large-ui-braces-lsp.json' if braces else 'reports/large-ui-lsp.json') if large else 'reports/macro-ui-lsp.json').write_text(json.dumps(report,indent=2)+'\n')
            response(send('shutdown',None,True));send('exit',None);process.wait(timeout=5)
            if rsx:
                assert all(case['supported'] for name,case in report.items() if name not in ['rust_child','style_prop']),report
            else:
                assert all(case['supported'] for case in report.values()),report
        finally:
            if process.poll() is None:process.terminate();process.wait(timeout=5)
if __name__=='__main__':main()
