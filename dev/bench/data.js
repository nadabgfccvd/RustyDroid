window.BENCHMARK_DATA = {
  "lastUpdate": 1793479327896,
  "repoUrl": "https://github.com/nadabgfccvd/RustyDroid",
  "entries": {
    "rd-dex parse tables": [
      {
        "commit": {
          "author": {
            "email": "117041552+nadabgfccvd@users.noreply.github.com",
            "name": "nadabgfccvd",
            "username": "nadabgfccvd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d8f4bd5007232235d255a32b645ac20647cf0d97",
          "message": "ci(m12): elf fuzz target, bench regression gate, release SBOM and provenance (#235 item 3) (#437)\n\n- fuzz.yml: nightly matrix runs the rd-ndk 'elf' target; ci.yml fuzz-build\n  compiles it on every PR.\n- bench.yml (new): push main + manual. rd-dex parse-table benches\n  (harness = false, libtest-format output, no new dependency) compared with\n  github-action-benchmark (pinned v1.22.2); fails at a 110% threshold (10%\n  regression). History lives on the gh-pages branch.\n- release.yml: CycloneDX SBOM (cargo-cyclonedx 0.5.9, pinned) inside the\n  payload; SLSA build provenance on the tarball via actions/attest-build-provenance\n  (pinned v4.2.2). Permissions limited to the build job.\n\nnightly-compat.yml and docs.yml already matched the spec.\n\nRefs #235\n\nCo-authored-by: xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-10-31T15:27:42-03:00",
          "tree_id": "1190f444f474246831c0b83699672649c1078e08",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/d8f4bd5007232235d255a32b645ac20647cf0d97"
        },
        "date": 1793471298402,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 10417,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 10413,
            "range": "± 36",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "117041552+nadabgfccvd@users.noreply.github.com",
            "name": "nadabgfccvd",
            "username": "nadabgfccvd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d0125b3a87794d22747ea27559db31e07b5d74bb",
          "message": "docs: status honesto M1/M4 + ADR 019 (renderer) (#438)\n\n- ROADMAP: regra de status; M1 e M4 viram ⚠️ parcial com auditoria\n  (fuzz 24h nunca executado; sem tiny-skia/shaping, ADR 019); M5 anota\n  que as provas usam APK sintético; texto do M4 descreve o atlas DejaVu real.\n- ADR 019: decisão do renderer pendente (portado de c5946e1 como 019,\n  já que 003 existe).\n- README/README.en: renderer descrito como rasterizador próprio;\n  tiny-skia como alvo da spec, pendente de ADR 019.\n- xtask: fase ⚠️ parcial reconhecida pelo parser (antes virava ⬜);\n  compat-matrix regenerado.\n\nCo-authored-by: xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-10-31T17:28:23-03:00",
          "tree_id": "359f39d18a40e28d5e02ba986ac8e91498cbf996",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/d0125b3a87794d22747ea27559db31e07b5d74bb"
        },
        "date": 1793478522206,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16854,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16751,
            "range": "± 51",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "117041552+nadabgfccvd@users.noreply.github.com",
            "name": "nadabgfccvd",
            "username": "nadabgfccvd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d0125b3a87794d22747ea27559db31e07b5d74bb",
          "message": "docs: status honesto M1/M4 + ADR 019 (renderer) (#438)\n\n- ROADMAP: regra de status; M1 e M4 viram ⚠️ parcial com auditoria\n  (fuzz 24h nunca executado; sem tiny-skia/shaping, ADR 019); M5 anota\n  que as provas usam APK sintético; texto do M4 descreve o atlas DejaVu real.\n- ADR 019: decisão do renderer pendente (portado de c5946e1 como 019,\n  já que 003 existe).\n- README/README.en: renderer descrito como rasterizador próprio;\n  tiny-skia como alvo da spec, pendente de ADR 019.\n- xtask: fase ⚠️ parcial reconhecida pelo parser (antes virava ⬜);\n  compat-matrix regenerado.\n\nCo-authored-by: xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-10-31T17:28:23-03:00",
          "tree_id": "359f39d18a40e28d5e02ba986ac8e91498cbf996",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/d0125b3a87794d22747ea27559db31e07b5d74bb"
        },
        "date": 1793479327314,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 9708,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 9645,
            "range": "± 114",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}