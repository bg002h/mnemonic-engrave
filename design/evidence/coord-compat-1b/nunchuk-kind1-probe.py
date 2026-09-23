import json,hashlib,re
c=json.load(open('/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/tests/fixtures/liana/cases.json'))[0]
B58='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
def b58c(b):
    b=b+hashlib.sha256(hashlib.sha256(b).digest()).digest()[:4]
    n=int.from_bytes(b,'big');s=''
    while n: n,r=divmod(n,58); s=B58[r]+s
    return '1'*(len(b)-len(b.lstrip(b'\0')))+s
def b58d(s):
    n=0
    for ch in s: n=n*58+B58.index(ch)
    b=n.to_bytes(82,'big'); return b[:-4]
NUMS='0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0'
def unsp(pks): return b58c(bytes.fromhex('0488b21e')+b'\0'*9+hashlib.sha256(b''.join(pks)).digest()+bytes.fromhex(NUMS))
d=c['descriptor_with_checksum'].split('#')[0]
ik=c['expected_xpub']
keys=re.findall(r"\[[0-9a-f]{8}/[^\]]*\](xpub[1-9A-HJ-NP-Za-km-z]+)/<0;1>/\*",d)
full=re.findall(r"(\[[0-9a-f]{8}/[^\]]*\]xpub[1-9A-HJ-NP-Za-km-z]+/<0;1>/\*)",d)
pk=lambda x: b58d(x)[-33:]
pks=[pk(k) for k in keys]
assert unsp(pks)==ik, 'liana recipe mismatch'
rows=[]
rows.append(('k1-liana-unsorted', d))                      # kind 1 as md renders it; keys unsorted
rows.append(('pr1746-nunchuk-form', d.replace(ik, unsp(sorted(set(pks))))))  # Nunchuk's own form, same leaves
# reorder the four key expressions so leaf pubkeys ascend in descriptor order
order=sorted(range(4),key=lambda i:pks[i])
t=d.replace(ik,'@IK@')
for i,f in enumerate(full): t=t.replace(f,'@K%d@'%i)
for pos,src in enumerate(order): t=t.replace('@K%d@'%pos, full[src])
spk=[pks[i] for i in order]; assert spk==sorted(spk)
rows.append(('k1-liana-sorted', t.replace('@IK@', unsp(spk))))
print('sorted==pr1746 for sorted row:', unsp(spk)==unsp(sorted(set(spk))))
with open('recon-1b-nunchuk-probe.tsv','w') as f:
    for n,x in rows: f.write(n+'\t'+x+'\n')
