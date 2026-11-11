"""Gera rótulos sintéticos com gabarito para testar o Scanner do Comprix.

Não usa imagens de terceiros: tudo é desenhado aqui, com valores conhecidos.
Saída: PNGs + gabarito.json (o que o OCR/código de barras deve devolver).
"""
import json
import os

from PIL import Image, ImageDraw, ImageFont

OUT = os.path.dirname(os.path.abspath(__file__))
FONT = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
FONT_B = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"

# ---- EAN-13 (padrão GS1) ----
L = ["0001101", "0011001", "0010011", "0111101", "0100011",
     "0110001", "0101111", "0111011", "0110111", "0001011"]
G = ["0100111", "0110011", "0011011", "0100001", "0011101",
     "0111001", "0000101", "0010001", "0001001", "0010111"]
R = ["1110010", "1100110", "1101100", "1000010", "1011100",
     "1001110", "1010000", "1000100", "1110100", "1110010"]
PARIDADE = ["LLLLLL", "LLGLGG", "LLGGLG", "LLGGGL", "LGLLGG",
            "LGGLLG", "LGGGLL", "LGLGLG", "LGLGGL", "LGGLGL"]


def dv_ean13(d12: str) -> str:
    s = sum(int(c) * (1 if i % 2 == 0 else 3) for i, c in enumerate(d12))
    return d12 + str((10 - s % 10) % 10)


def bits_ean13(code: str) -> str:
    assert len(code) == 13 and code == dv_ean13(code[:12]), code
    par = PARIDADE[int(code[0])]
    bits = "101"
    for i, d in enumerate(code[1:7]):
        bits += L[int(d)] if par[i] == "L" else G[int(d)]
    bits += "01010"
    for d in code[7:]:
        bits += R[int(d)]
    return bits + "101"


def desenhar_ean13(code: str, path: str):
    bits = bits_ean13(code)
    mod = 4
    altura = 160
    margem = 20 * mod
    w = len(bits) * mod + 2 * margem
    h = altura + 100
    img = Image.new("RGB", (w, h), "white")
    d = ImageDraw.Draw(img)
    for i, b in enumerate(bits):
        if b == "1":
            x = margem + i * mod
            d.rectangle([x, 20, x + mod - 1, 20 + altura], fill="black")
    f = ImageFont.truetype(FONT, 36)
    d.text((margem, 20 + altura + 8),
           code[0] + "  " + code[1:7] + "  " + code[7:], font=f, fill="black")
    img.save(path)


# ---- Tabela nutricional (modelo ANVISA, valores conhecidos) ----
def desenhar_tabela(path):
    W, H = 900, 1000
    img = Image.new("RGB", (W, H), "white")
    d = ImageDraw.Draw(img)
    fb = ImageFont.truetype(FONT_B, 44)
    f = ImageFont.truetype(FONT, 34)
    fs = ImageFont.truetype(FONT, 28)
    d.rectangle([30, 30, W - 30, H - 30], outline="black", width=4)
    d.text((60, 60), "INFORMAÇÃO NUTRICIONAL", font=fb, fill="black")
    d.text((60, 130), "Porções por embalagem: 10", font=f, fill="black")
    d.text((60, 175), "Porção: 30 g (1 unidade)", font=f, fill="black")
    d.rectangle([50, 230, W - 50, 240], fill="black")
    cols = [560, 690, 820]
    d.text((cols[0] - 10, 250), "30 g", font=fs, fill="black")
    d.text((cols[1] - 10, 250), "100 g", font=fs, fill="black")
    d.text((cols[2] - 10, 250), "%VD*", font=fs, fill="black")
    linhas = [
        ("Valor energético (kcal)", "120", "400", "6%"),
        ("Carboidratos (g)", "22", "73", "7%"),
        ("Açúcares totais (g)", "9", "30", "10%"),
        ("Açúcares adicionados (g)", "8", "27", "11%"),
        ("Proteínas (g)", "3", "10", "4%"),
        ("Gorduras totais (g)", "2,5", "8,3", "4%"),
        ("Gorduras saturadas (g)", "0,8", "2,7", "4%"),
        ("Gorduras trans (g)", "0", "0", "**"),
        ("Fibras alimentares (g)", "2", "7", "8%"),
        ("Sódio (mg)", "150", "500", "6%"),
    ]
    y = 300
    for nome, a, b, c in linhas:
        d.text((60, y), nome, font=f, fill="black")
        d.text((cols[0] - 10, y), a, font=f, fill="black")
        d.text((cols[1] - 10, y), b, font=f, fill="black")
        d.text((cols[2] - 10, y), c, font=f, fill="black")
        d.line([50, y + 48, W - 50, y + 48], fill="black", width=2)
        y += 62
    d.text((60, H - 80), "*Percentual de valores diários fornecidos pela porção.",
           font=fs, fill="black")
    img.save(path)


# ---- Selo "ALTO EM" (layout oficial: lupa + texto e caixas pretas) ----
def caixa_preta(d, x, y, texto, fonte):
    bb = d.multiline_textbbox((0, 0), texto, font=fonte, align="center", spacing=4)
    tw, th = bb[2] - bb[0], bb[3] - bb[1]
    w, h = tw + 36, th + 24
    d.rounded_rectangle([x, y, x + w, y + h], radius=8, fill="black")
    d.multiline_text((x + 18, y + 12), texto, font=fonte, fill="white",
                     align="center", spacing=4)
    return w


def desenhar_selo(path):
    W, H = 900, 380
    img = Image.new("RGB", (W, H), "white")
    d = ImageDraw.Draw(img)
    fb = ImageFont.truetype(FONT_B, 30)
    fa = ImageFont.truetype(FONT_B, 26)
    d.rounded_rectangle([20, 20, W - 20, H - 20], radius=18, outline="black", width=4)
    d.ellipse([60, 60, 110, 110], outline="black", width=6)
    d.line([104, 104, 130, 130], fill="black", width=8)
    d.rounded_rectangle([130, 62, 300, 110], radius=10, outline="black", width=3)
    d.text((140, 72), "ALTO EM", font=fa, fill="black")
    x = 60
    for t in ["AÇÚCAR\nADICIONADO", "GORDURA\nSATURADA", "SÓDIO"]:
        w = caixa_preta(d, x, 160, t, fb)
        x += w + 24
    img.save(path)


# ---- Etiqueta de preço (gôndola) ----
def desenhar_preco(path):
    W, H = 800, 420
    img = Image.new("RGB", (W, H), (255, 240, 120))
    d = ImageDraw.Draw(img)
    d.text((40, 30), "Arroz Tipo 1 5 kg", font=ImageFont.truetype(FONT_B, 48), fill="black")
    d.text((40, 110), "Marca Exemplo", font=ImageFont.truetype(FONT, 36), fill="black")
    d.text((40, 220), "R$ 24,90", font=ImageFont.truetype(FONT_B, 110), fill="black")
    d.text((40, 360), "7891000100103", font=ImageFont.truetype(FONT, 40), fill="black")
    img.save(path)


if __name__ == "__main__":
    casos = {}
    eans = [dv_ean13("789100010010"), dv_ean13("789600440001"),
            dv_ean13("789490001151")]
    for c in eans:
        desenhar_ean13(c, os.path.join(OUT, f"ean_{c}.png"))
    casos["ean"] = {"arquivos": [f"ean_{c}.png" for c in eans], "codigos": eans}
    desenhar_tabela(os.path.join(OUT, "tabela_nutricional.png"))
    casos["tabela"] = {"arquivo": "tabela_nutricional.png",
                       "esperado": {"porcao": "30 g", "energia_kcal": 120,
                                    "sodio_mg": 150, "acucares_adicionados_g": 8,
                                    "gorduras_saturadas_g": 0.8}}
    desenhar_selo(os.path.join(OUT, "selo_alto_em.png"))
    casos["selo"] = {"arquivo": "selo_alto_em.png",
                     "esperado": ["ALTO EM", "AÇÚCAR ADICIONADO", "GORDURA SATURADA", "SÓDIO"]}
    desenhar_preco(os.path.join(OUT, "etiqueta_preco.png"))
    casos["preco"] = {"arquivo": "etiqueta_preco.png",
                      "esperado": {"preco": "24,90", "codigo": "7891000100103"}}
    with open(os.path.join(OUT, "gabarito.json"), "w", encoding="utf-8") as f:
        json.dump(casos, f, ensure_ascii=False, indent=2)
    print("ok", sorted(os.listdir(OUT)))
