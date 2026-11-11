"""Valida os rotulos sinteticos contra fixtures/gabarito.json (fora do VM).
EAN: zbarimg. Selo, tabela e preco: RapidOCR. Comparacao sem acento e por palavra
(a ordem de leitura do OCR nao e garantida)."""
import json, re, subprocess, unicodedata
from rapidocr_onnxruntime import RapidOCR
F = "/home/user/comprix/fixtures/"
g = json.load(open(F + "gabarito.json"))
def sa(s): return "".join(c for c in unicodedata.normalize("NFD", s) if unicodedata.category(c) != "Mn").upper()
eng = RapidOCR()
def ocr(p):
    r, _ = eng(F + p); return [x[1] for x in (r or [])]
res = {}
# EAN
for arq, cod in zip(g["ean"]["arquivos"], g["ean"]["codigos"]):
    lido = subprocess.run(["zbarimg", "-q", "--raw", F + arq], capture_output=True, text=True).stdout.strip()
    res[cod] = lido == cod
# selo: por palavra
toks = set(sa(" ".join(ocr(g["selo"]["arquivo"]))).split())
res["selo"] = {p: all(w in toks for w in sa(p).split()) for p in g["selo"]["esperado"]}
# tabela
t = sa(" ".join(ocr(g["tabela"]["arquivo"])))
e = g["tabela"]["esperado"]
res["tabela"] = {
    "porcao_30g": bool(re.search(r"PORCAO:?\s*30\s*G", t)),
    "energia_120": bool(re.search(r"\b120\b", t)),
    "sodio_150": bool(re.search(r"\b150\b", t)),
    "saturadas_0,8": bool(re.search(r"\b0[,.]8\b", t)),
    "acucares_adicionados_8": "inconclusivo: OCR nao leu o valor 8 na linha do rotulo",
}
# preco
t = sa(" ".join(ocr(g["preco"]["arquivo"])))
res["preco"] = {"24,90": "24,90" in t, "codigo": g["preco"]["esperado"]["codigo"] in re.sub(r"\D", "", t)}
print(json.dumps(res, ensure_ascii=False, indent=1))
open(F.replace("fixtures/", "") + "validacao_fixtures.json", "w").write(json.dumps(res, ensure_ascii=False, indent=1))
