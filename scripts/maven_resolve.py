"""Resolve dependências Maven (compile/runtime, transitivas) de Google Maven + Maven Central.
Saída: lista de jars (AAR -> classes.jar extraído) em jars/. Uso: maven_resolve.py out_dir coord...
"""
import os, re, sys, zipfile, urllib.request, urllib.error, xml.etree.ElementTree as ET
REPOS = ["https://dl.google.com/dl/android/maven2/", "https://repo1.maven.org/maven2/"]
CACHE = {}
def fetch(rel):
    if rel in CACHE: return CACHE[rel]
    for r in REPOS:
        try:
            with urllib.request.urlopen(r + rel, timeout=120) as f:
                data = f.read(); CACHE[rel] = (r, data); return CACHE[rel]
        except urllib.error.HTTPError:
            continue
    CACHE[rel] = None; return None
def strip(t): return t.split('}', 1)[1] if t.startswith('{') else t
def kids(el, name): return [c for c in el if strip(c.tag) == name]
def text(el, name):
    for c in kids(el, name): return (c.text or '').strip()
    return None
def pom_of(g, a, v):
    rel = f"{g.replace('.', '/')}/{a}/{v}/{a}-{v}.pom"
    got = fetch(rel)
    if not got: return None, None
    return ET.fromstring(got[1]), got[0]
def props_and_managed(root, parent_chain):
    props = {}
    for p in parent_chain + [root]:
        if p is None: continue
        pe = kids(p, 'properties')
        if pe:
            for c in pe[0]: props[strip(c.tag)] = (c.text or '').strip()
    return props
def subst(s, props):
    if s is None: return None
    for _ in range(5):
        s = re.sub(r'\$\{([^}]+)\}', lambda m: props.get(m.group(1), m.group(0)), s)
    return s
def load(g, a, v):
    """Carrega POM com pais; devolve (props, managed dict (g:a)->version, deps list)."""
    chain = []; root, _ = pom_of(g, a, v)
    if root is None: return None
    cur = root
    while True:
        par = kids(cur, 'parent')
        if not par: break
        pg, pa, pv = text(par[0], 'groupId'), text(par[0], 'artifactId'), text(par[0], 'version')
        pr, _ = pom_of(pg, pa, pv)
        if pr is None: break
        chain.append(pr); cur = pr
    chain_all = chain  # nearest parent first
    props = {}
    for p in reversed(chain_all + [root]):
        pe = kids(p, 'properties')
        if pe:
            for c in pe[0]: props[strip(c.tag)] = (c.text or '').strip()
    props.setdefault('project.version', v); props.setdefault('project.groupId', g)
    managed = {}
    for p in reversed(chain_all + [root]):
        dm = kids(p, 'dependencyManagement')
        if dm:
            for d in kids(dm[0], 'dependencies'):
                for dep in kids(d, 'dependency'):
                    k = subst(text(dep, 'groupId'), props) + ':' + subst(text(dep, 'artifactId'), props)
                    managed[k] = subst(text(dep, 'version'), props)
    deps = []
    ds = kids(root, 'dependencies')
    if ds:
        for dep in kids(ds[0], 'dependency'):
            gg = subst(text(dep, 'groupId'), props); aa = subst(text(dep, 'artifactId'), props)
            vv = subst(text(dep, 'version'), props) or managed.get(gg + ':' + aa)
            sc = text(dep, 'scope') or 'compile'; opt = text(dep, 'optional') == 'true'
            if sc in ('compile', 'runtime') and not opt:
                deps.append((gg, aa, vv, dep))
    return deps
def main():
    out = sys.argv[1]; os.makedirs(out, exist_ok=True)
    seen = {}; queue = []
    for c in sys.argv[2:]:
        g, a, v = c.split(':'); queue.append((g, a, v))
    while queue:
        g, a, v = queue.pop(0)
        k = f"{g}:{a}"
        if k in seen: continue
        deps = load(g, a, v)
        if deps is None:
            print('MISSING', g, a, v); seen[k] = None; continue
        seen[k] = v
        for gg, aa, vv, _ in deps:
            if vv is None or vv.startswith('${'):
                print('UNRESOLVED', gg, aa, vv); continue
            if vv.startswith('['): vv = vv.strip('[]')
            if f"{gg}:{aa}" not in seen: queue.append((gg, aa, vv))
    # download
    for k, v in seen.items():
        if v is None: continue
        g, a = k.split(':')
        for ext in ('aar', 'jar'):
            rel = f"{g.replace('.', '/')}/{a}/{v}/{a}-{v}.{ext}"
            got = fetch(rel)
            if got:
                path = os.path.join(out, f"{a}-{v}.{ext}")
                with open(path, 'wb') as f: f.write(got[1])
                if ext == 'aar':
                    jar = path[:-4] + '.jar'
                    with zipfile.ZipFile(path) as z:
                        if 'classes.jar' in z.namelist():
                            with open(jar, 'wb') as f: f.write(z.read('classes.jar'))
                break
    print('RESOLVED', len(seen))
main()
