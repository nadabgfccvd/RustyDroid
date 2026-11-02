# Fontes do snapshot (rd-render)

- `DejaVuSans.ttf` — DejaVu Sans 2.37 (pacote Debian `fonts-dejavu-core`,
  SHA-256 `57f73e11f51999432bf7ab22ce55b6f945d5eca1bf824404cfa9ec2e3718c84e`).
  Usada pelo canvas tiny-skia + rustybuzz do `render_snapshot` (M4).
- `DejaVu-LICENSE.txt` — texto de licença e copyright original (Bitstream Vera
  + licença pública de DejaVu), redistribuído junto com a fonte.

Motivo: o snapshot precisa ser determinístico e não depender de fontes do
sistema. Não é a fonte do Android (Roboto); a fidelidade visual continua fora
de escopo (ver ADR 019).
