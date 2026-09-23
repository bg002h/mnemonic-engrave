"""Apply a plan's blocks, task by task, to a git archive of the baseline; after
each task run the plan's generator commands, and compare the tree with the
staged commit for that task. Exit non-zero on any mismatch."""
import os, re, shutil, subprocess, sys
T='/scratch/code/shibboleth/.tmp'
plan=sys.argv[1]; X=f'{T}/cc1b-extracted'; S=f'{T}/cc1b-staged'; MIRROR=f'{T}/cc1b-engrave-mirror'
env=dict(os.environ, PATH=os.path.expanduser('~/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin')+':'+os.environ['PATH'], CARGO_TARGET_DIR=f'{T}/cc1b-target')
shutil.rmtree(X, ignore_errors=True); os.makedirs(X)
subprocess.run(f'git -C /scratch/code/shibboleth/descriptor-mnemonic archive d269c556 | tar -x -C {X}', shell=True, check=True)
subprocess.run(['git','init','-q'],cwd=X,check=True)
lines=open(plan).read().split('\n')
anchor=re.compile(r'^(Create|Replace|Modify) `([^`]+)`:$')
task=None; i=0; ops=[]
while i < len(lines):
    tm=re.match(r'^### Task (\d+):', lines[i])
    if tm: task=int(tm.group(1)); ops.append(('task',task)); i+=1; continue
    m=anchor.match(lines[i])
    if m:
        j=i+1
        while not lines[j].startswith('```'): j+=1
        f=re.match(r'^(`{3,})(\w*)$', lines[j]); fence=f.group(1); k=j+1; body=[]
        while lines[k]!=fence: body.append(lines[k]); k+=1
        ops.append((m.group(1), m.group(2), '\n'.join(body)+'\n')); i=k+1; continue
    i+=1
staged={}
for l in subprocess.run(['git','log','--format=%H %s'],cwd=S,capture_output=True,text=True).stdout.split('\n'):
    if l: h,s=l.split(' ',1); staged[s]=h
def check(t):
    ref=f'{T}/cc1b-ref'; shutil.rmtree(ref, ignore_errors=True); os.makedirs(ref)
    subprocess.run(f'git -C {S} archive {staged[t]} | tar -x -C {ref}', shell=True, check=True)
    r=subprocess.run(['diff','-r','-q','-x','.git','-x','target',ref,X],capture_output=True,text=True)
    print(f'  {t}: ' + ('IDENTICAL to staged' if not r.stdout else 'DIFFERS\n'+r.stdout)); return not r.stdout
def after(t):
    if t==5: subprocess.run([f'{X}/scripts/vendor-coord-evidence.sh', MIRROR],cwd=X,env=env,check=True)
    if t==6:
        subprocess.run(['cargo','update','--workspace','--offline'],cwd=X,env=env,check=True,capture_output=True)
        subprocess.run(['cargo','xtask','verdicts'],cwd=X,env=env,check=True)
    if t==9: subprocess.run(['cargo','update','--workspace','--offline'],cwd=X,env=env,check=True,capture_output=True)
ok=True; cur=None
for op in ops+[('task',None)]:
    if op[0]=='task':
        if cur is not None and cur>0: after(cur); ok&=check(f'T{cur}')
        cur=op[1]; continue
    verb,path,body=op; full=os.path.join(X,path)
    if verb in ('Create','Replace'):
        os.makedirs(os.path.dirname(full),exist_ok=True); open(full,'w').write(body)
        if path.endswith('.sh'): os.chmod(full,0o755)
    else:
        r=subprocess.run(['git','apply','--whitespace=nowarn','-'],cwd=X,input=body,text=True,capture_output=True)
        if r.returncode: print('APPLY FAILED',path,r.stderr); ok=False
sys.exit(0 if ok else 1)
