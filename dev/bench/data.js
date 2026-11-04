window.BENCHMARK_DATA = {
  "lastUpdate": 1793834356734,
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
          "id": "2e3ea5383dcad945e5b53382a3a84f54ce3ceba4",
          "message": "fix(rd-vm): ViewModel base host shim (#232) (#439)\n\n* fix(rd-vm): ViewModel base host shim (getTag, setTagIfAbsent, addCloseable, clear) (#232)\n\nSubclasses DEX de androidx.lifecycle.ViewModel que não definem getTag /\nsetTagIfAbsent / addCloseable / clear falhavam com UNRESOLVED_METHOD,\nmesmo com o shim host de ViewModelStore.\n\n- androidx: classe base Landroidx/lifecycle/ViewModel; tags e closeables\n  ficam nos campos da instância (rd_vm_*), seguidos pelo GC.\n- clear(): chama onCleared (override DEX) e fecha os Closeable em ordem.\n- ViewModelStore.clear() também fecha os Closeable registrados.\n- Teste: crates/rd-vm/tests/androidx_viewmodel_232.rs (repro do #232,\n  via invoke-virtual em bytecode).\n\nDeclaração: código escrito com assistência de IA.\n\n* test(rd-vm): assinatura do probe no teste #232 (LVMProbe;)\n\n* test(rd-vm): nomes snake_case e clippy no teste #232 (-D warnings)\n\n---------\n\nCo-authored-by: xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-10-31T18:02:45-03:00",
          "tree_id": "e741f73ff83b80d09ecb326ef7560d74355b9936",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/2e3ea5383dcad945e5b53382a3a84f54ce3ceba4"
        },
        "date": 1793480584037,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16799,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16776,
            "range": "± 84",
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
          "id": "7df53b91e2e524f29d1cf1beb39368626cb980d8",
          "message": "test(rd-vm): regressão #232 item 1 — new_instance/invoke_instance em shims host (#440)\n\nEngine::new_instance(TaskCompletionSource) e ViewModelStore caem na mesma\ncadeia host de do_invoke (gms/androidx), então <init> resolve. Teste novo\ncobre as duas classes e getTag no ViewModel base via Engine::invoke_instance.\n\nDeclaração: código escrito com assistência de IA.\n\nCo-authored-by: xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-10-31T18:09:41-03:00",
          "tree_id": "25a39cd310bf3b923837a6591da18237a6f6e9af",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/7df53b91e2e524f29d1cf1beb39368626cb980d8"
        },
        "date": 1793481006784,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16865,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16859,
            "range": "± 73",
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
          "id": "b61672413e26259ef9a99f6ff80cb2e687074544",
          "message": "fix(rd-vm): keep full array class in new_array (refs #379) (#441)\n\nnew_array stripped one '[' from the descriptor before calling alloc_array,\nso a nested array class such as [[I was stored as [I. Pass the array\ndescriptor through unchanged, as alloc_array_resized already does.\n\nSingle-dimension and bare-element callers produce the same descriptor as\nbefore. The reflection GC rooting part of #379 is not addressed here.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T08:33:29-03:00",
          "tree_id": "72f72bdd952658a2d50199303f9987b84b9cfbcb",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/b61672413e26259ef9a99f6ff80cb2e687074544"
        },
        "date": 1793532827915,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16998,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16818,
            "range": "± 78",
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
          "id": "47d4a2e9e5887eecd4d1179ad34c1eba65bcbaed",
          "message": "ci(dependabot): drop fuzz cargo update entries that cannot resolve (#442)\n\nDependabot copies only the fuzz crate directory and its path dependencies,\nso rd-dex and rd-apk (edition/version inherited from the workspace root)\nfail with dependency_file_not_resolvable on every run of the rd-dex and\nrd-apk fuzz 'Update' jobs. The fuzz crates themselves keep building through\nscripts/fuzz-campaign.sh; their libfuzzer-sys pin must now be bumped by hand.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T08:58:55-03:00",
          "tree_id": "a6db1d7728c6a1b4eac742c227c997e2df76b292",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/47d4a2e9e5887eecd4d1179ad34c1eba65bcbaed"
        },
        "date": 1793534354092,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16747,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16861,
            "range": "± 572",
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
          "id": "1568b46edc7e77fc6c3e837df91b9e00f41ff12f",
          "message": "docs(worklog): record two durable pitfalls (#454)\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T09:14:12-03:00",
          "tree_id": "85d1e22233df2e87d986b5fca41ed305a254a696",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/1568b46edc7e77fc6c3e837df91b9e00f41ff12f"
        },
        "date": 1793535269177,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 9820,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 9656,
            "range": "± 168",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7cfa20638b0b82db09b1d81b70a00ce4d77be045",
          "message": "ci(deps)(deps): bump actions/setup-python (#449)\n\nBumps the github-actions-minor-and-patch group with 1 update: [actions/setup-python](https://github.com/actions/setup-python).\n\n\nUpdates `actions/setup-python` from 7.0.0 to 7.1.0\n- [Release notes](https://github.com/actions/setup-python/releases)\n- [Commits](https://github.com/actions/setup-python/compare/5fda3b95a4ea91299a34e894583c3862153e4b97...6d4fd22b20465ccc0f7354476588e8df7c9ca0e0)\n\n---\nupdated-dependencies:\n- dependency-name: actions/setup-python\n  dependency-version: 7.1.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n  dependency-group: github-actions-minor-and-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-01T09:18:05-03:00",
          "tree_id": "37e26ab722753157d7e9291545ddf3dce0bc5e5c",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/7cfa20638b0b82db09b1d81b70a00ce4d77be045"
        },
        "date": 1793535513509,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16886,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16891,
            "range": "± 34",
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
          "id": "71016465e16f3e8b30585b85338de0a6de333471",
          "message": "test(rd-vm): cover reflection member enumeration (refs #379) (#455)\n\ngetMethods, getDeclaredMethods, and getConstructors allocate Method and\nConstructor instances from inside a host call. No test covered their result\nset. Pin the public/declared split, the descriptors, and <init> handling.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T09:29:35-03:00",
          "tree_id": "3550584520e380df710003e14ed4ba7945a3efd5",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/71016465e16f3e8b30585b85338de0a6de333471"
        },
        "date": 1793536194566,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 17368,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 17351,
            "range": "± 65",
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
          "id": "9f74be0ffc89a208e2f24cd8a5ed7172f05ce486",
          "message": "fix(vm,render,agent): follow the active viewport on landscape (refs #252) (#456)\n\nset_orientation('landscape') swapped DisplayMetrics to 1440x720, but the layout\nroot, window decor, inflate caps, display list, ScreenModel, and screenshot\nstayed on the portrait 720x1440 constants. The viewport is now one source of\ntruth, Engine::viewport_px(), read from the sysconfig size. Portrait stays the\ndefault, so existing behavior is unchanged.\n\n- window decor and content frames resize on orientation change\n- inflate per-axis caps use the active viewport (800px width is legal in\n  landscape, an 800px height is rejected on the 720px height axis)\n- rd-render gains render_snapshot_sized; the agent screenshot and finished-window\n  ScreenModel use the active viewport\n- MCP screenshot 'size' comes from the PNG IHDR instead of a hard-coded 720x1440\n\nTests: viewport_252 (default, swap, reject) and app_xml landscape_viewport_swaps_the_axis_caps.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T10:01:35-03:00",
          "tree_id": "09f76937059a2fd8fd3e0fa08d7f98de3deb3375",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/9f74be0ffc89a208e2f24cd8a5ed7172f05ce486"
        },
        "date": 1793538120905,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16777,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16780,
            "range": "± 54",
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
          "id": "b366fd325167db7f16fd264719e28b619a7c99d4",
          "message": "chore(deps): drop five unused dependencies flagged by cargo-machete (#457)\n\nRD Sentinel's strict gate fails on cargo-machete (see #273, #390). Each removed\ndependency has no reference in its crate's sources:\n- xtask: serde\n- rd-apk: thiserror\n- rd-cli: anyhow\n- rd-jni: serde_json\n- rd-framework: serde_json, thiserror\n\nCargo.lock only loses entries. cargo check --workspace --all-targets\n--all-features --locked passes, and cargo machete reports nothing.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T10:12:05-03:00",
          "tree_id": "c1aefce6bb666aa7cfb4eef370babf91b084a9b5",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/b366fd325167db7f16fd264719e28b619a7c99d4"
        },
        "date": 1793538744952,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16893,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16847,
            "range": "± 104",
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
          "id": "159d80ad219caada96ef4e1ef7f4ddccd37c05dc",
          "message": "docs(roadmap): note landscape viewport on the M10 row (refs #252) (#458)\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T10:19:18-03:00",
          "tree_id": "93ea37dba103ce2bd34e1518a5808ff4c9724c5f",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/159d80ad219caada96ef4e1ef7f4ddccd37c05dc"
        },
        "date": 1793539183907,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16839,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16825,
            "range": "± 76",
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
          "id": "e41607c9ee7d32133b4c4cca98b05e30994fb58c",
          "message": "docs(roadmap): re-verify M0 inspect against a real APK (F-Droid 2.0.2) (#459)\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T10:27:38-03:00",
          "tree_id": "c89d42175255e0bd2cdab12813dc15c94b6cc542",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/e41607c9ee7d32133b4c4cca98b05e30994fb58c"
        },
        "date": 1793539688923,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 11326,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 11427,
            "range": "± 69",
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
          "id": "0f4b1d5ff887062e349c03fa4784fccb85efabfb",
          "message": "docs: drop stale references to the removed tools/rd-sentinel directory (#460)\n\ntools/rd-sentinel/ is absent on main (deleted in 08ae54f). Remove it from the\nREADME table, the worklog crate map and its test pointer, and the comments in\nthe docs, fuzz and release workflows. Workflow logic, .github/rd-sentinel\npolicy and baseline, CODEOWNERS, and the ADR 010/018 history are unchanged.\nThe gate itself is not changed by this commit.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T10:38:56-03:00",
          "tree_id": "72c2dc16c26bf1da6eae0f875874a4991da05883",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/0f4b1d5ff887062e349c03fa4784fccb85efabfb"
        },
        "date": 1793540356679,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16801,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16842,
            "range": "± 109",
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
          "id": "0f4b1d5ff887062e349c03fa4784fccb85efabfb",
          "message": "docs: drop stale references to the removed tools/rd-sentinel directory (#460)\n\ntools/rd-sentinel/ is absent on main (deleted in 08ae54f). Remove it from the\nREADME table, the worklog crate map and its test pointer, and the comments in\nthe docs, fuzz and release workflows. Workflow logic, .github/rd-sentinel\npolicy and baseline, CODEOWNERS, and the ADR 010/018 history are unchanged.\nThe gate itself is not changed by this commit.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T10:38:56-03:00",
          "tree_id": "72c2dc16c26bf1da6eae0f875874a4991da05883",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/0f4b1d5ff887062e349c03fa4784fccb85efabfb"
        },
        "date": 1793540447414,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16830,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16827,
            "range": "± 59",
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
          "id": "7f113d51119b864eeedc65ee1b08b2f58caef0c2",
          "message": "fix(vm): one source of truth for the fused location fix (refs #229) (#461)\n\nFusedLocationProviderClient had two stores. The host API\n(gms_set_location_fix) wrote gms.location_fix, but DEX calls on the fused client\nare dispatched to m10 first, and m10 read only vdev.location.current. Bytecode\ncallers therefore always saw the São Paulo default and never the host fix.\n\n- m10_set_location now mirrors the fix into gms.location_fix, so the last write\n  wins whether it came from the agent or from the host.\n- LocationManager and FusedLocationProviderClient read through one helper,\n  current_fix(), which prefers gms.location_fix and falls back to the M10\n  default when no fix has been set.\n\ntests/location_fused_229.rs drives the same intrinsic order as bytecode\n(M10, then androidx, then GMS). It fails on main (-23.5505 instead of -23.55)\nand passes now. Full rd-vm: 95 blocks, 859 passed, 0 failed. clippy -D warnings\npasses for rd-vm.\n\nNot in this commit: interp.rs still calls m10 dispatch without the\nvirtual_hardware_enabled gate, while intrinsics.rs gates it. That is the third\npoint in #229 and needs an owner decision.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T11:30:24-03:00",
          "tree_id": "a6aa8feb5a61c13f11774b990035c4778db95d1b",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/7f113d51119b864eeedc65ee1b08b2f58caef0c2"
        },
        "date": 1793543454425,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16831,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16811,
            "range": "± 162",
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
          "id": "53901541218b1b5a1ed7c23ea831b2b030764679",
          "message": "docs(worklog): two durable pitfalls (bytecode dispatch order in tests, bench gate baseline) (#462)\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T11:30:27-03:00",
          "tree_id": "e6c6610071f55821df9e7f31f6673da4a1372be2",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/53901541218b1b5a1ed7c23ea831b2b030764679"
        },
        "date": 1793543479743,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16869,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16915,
            "range": "± 85",
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
          "id": "bd3df0bc5460bedefbcec9b472682885e2cf1064",
          "message": "docs(agent): describe the active viewport in MCP tool text (#465)\n\nThe MCP tool descriptions and mcp-tools.md still said the viewport was\nalways 720x1440. Since the viewport API (#252) landed, landscape swaps\nit to 1440x720. Say both sizes, and that the active viewport is the one\nused.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T12:26:23-03:00",
          "tree_id": "52862558086b0d0850b39ecb69c94a82c335c52e",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/bd3df0bc5460bedefbcec9b472682885e2cf1064"
        },
        "date": 1793546804063,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16795,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16830,
            "range": "± 287",
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
          "id": "b77103ae15fe6036cd7d8b3d637418aebcdace15",
          "message": "docs(worklog): record dependency, regex-limit and disk pitfalls (#466)\n\nThree pitfalls hit while porting cranelift 0.137 (#463) and fancy-regex\n0.20 (#464): the rust-version gate on MSRV, the regex backtracking\nlimit that 0.20 no longer hits with the old pattern, and disk exhaustion\nduring test linking.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T12:28:07-03:00",
          "tree_id": "0ef01eb9721cc70beec138dcc5f39fb4d00afe3e",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/b77103ae15fe6036cd7d8b3d637418aebcdace15"
        },
        "date": 1793546905908,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16903,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16925,
            "range": "± 61",
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
          "id": "5abfe598b08d978f7a9da77edf71c2b8296cd499",
          "message": "feat(engine-wasm): M11 S1 — WASM MVP interpreter (i32/i64), validator, fuel and limits (ADR 013) (#467)\n\nAdds crates/rd-engine-wasm: a pure-Rust, zero-dependency interpreter for\nthe WebAssembly MVP integer subset, as ADR 013 §4 decides.\n\n- Binary decoder and validator. Every module is fully validated before\n  instantiation (operand types, stack height including unreachable code,\n  indices, memory alignment, global mutability, export names).\n- Interpreter with fuel charged per executed instruction (deterministic),\n  structured WasmError codes, and no panic path.\n- Limits from ADR 013 §5: 2 MiB module, 16 MiB memory, 10M fuel, 16 args,\n  16 KiB payload (exported for the bridge), plus call depth, value stack\n  and a call-stack locals budget (64K slots) that bounds recursion.\n- Out of subset fails with NOT_IMPLEMENTED: floats, SIMD, tables and\n  call_indirect, imports, multi-value, element and passive segments.\n- Tests: 25 integration tests on hand-assembled modules, plus a\n  deterministic mutation/no-panic test (20 000 mutants, 5 000 random inputs).\n- Fuzz target wasm_module (libFuzzer), added to fuzz.yml and the\n  fuzz-build job. Verified on MSRV 1.76.\n- xtask CRATE_STATUS row and regenerated docs/compat-matrix.md.\n- docs/m11-wasm.md, ROADMAP M11 row, SUMMARY and FUZZING entries.\n\nNot in this change, and so not claimed: the golden APK, the\nio.rustydroid.wasm.WasmBridge framework class, and the registry/CLI flip.\nThe wasm engine stays runtime_status = not_implemented until they land.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T12:56:58-03:00",
          "tree_id": "833f4ca595f51d7d6be39cbe0c825a6bd9326751",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5abfe598b08d978f7a9da77edf71c2b8296cd499"
        },
        "date": 1793548642477,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 17370,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 17362,
            "range": "± 82",
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
          "id": "dcae791783c1099eb23bff6828706b43a68f5e34",
          "message": "feat(engine-wasm): M11 S1b — bridge WasmBridge atrás de wasm-engine + golden de VM (#468)\n\n- rd-vm: feature `wasm-engine` (off por padrão) com `wasm_bridge.rs`:\n  `io.rustydroid.wasm.WasmBridge.call(String,String,int,int)I` lê\n  assets/<caminho> do APK, instancia do zero e invoca export(a,b) sob o\n  fuel máximo. Falhas viram IllegalStateException com código estável.\n- Caminhos hostis rejeitados com a mesma regra do AssetManager\n  (asset_path_is_hostile agora pub(crate)).\n- Golden de VM (tests/wasm_bridge.rs, 8 testes): retorno, wrap i32,\n  asset ausente, caminho hostil, export ausente, bytes inválidos, laço\n  infinito (FUEL_EXHAUSTED) e Activity.onCreate chamando a bridge via\n  launch_app. Módulos montados byte a byte no teste; nenhum .wasm binário.\n- Registro (`wasm` segue not_implemented) e CLI inalterados; APK golden\n  em golden/engines/wasm/ e rd test ficam para o próximo passo.\n- docs: m11-wasm.md (contrato congelado), ROADMAP M11, compat-matrix\n  (regenerada, só a linha do rd-engine-wasm), worklog.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T13:14:33-03:00",
          "tree_id": "bdaf0ee9e5428227bcf8dc3f2992fdb026fbeded",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/dcae791783c1099eb23bff6828706b43a68f5e34"
        },
        "date": 1793549693791,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 16921,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 16882,
            "range": "± 445",
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
          "id": "e9f2c20d52b092c7b8c98c5c78d7f6f97d92a65f",
          "message": "build(vm): fancy-regex 0.20 port (supersedes #444) (#464)\n\n* chore(deps)(deps): bump fancy-regex from 0.13.0 to 0.20.0\n\nBumps [fancy-regex](https://github.com/fancy-regex/fancy-regex) from 0.13.0 to 0.20.0.\n- [Release notes](https://github.com/fancy-regex/fancy-regex/releases)\n- [Changelog](https://github.com/fancy-regex/fancy-regex/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/fancy-regex/fancy-regex/compare/0.13.0...0.20.0)\n\n---\nupdated-dependencies:\n- dependency-name: fancy-regex\n  dependency-version: 0.20.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\n\n* build(vm): fancy-regex 0.20 port (supersedes #444)\n\n- jregex.rs: fancy_regex::Captures now takes a type parameter. The set_caps\n  closure uses Captures<'_, String>, matching captures_from_pos(&input, ...).\n- tests/regex_lookaround.rs: 0.20 memoizes '(?=a)(a+)+b'. On 2000 'a's with no\n  'b' the correct answer is false, and the old test expected the backtrack error.\n  The test now uses '(?=a)((a|aa)+)+\\1c' on 40 'a's, which still returns\n  REGEX_RUNTIME. The test keeps its original intent: a limit overrun must be a\n  structured error, not a false.\n\nfancy-regex 0.20.0 declares rust-version 1.66, within the 1.76 MSRV.\n\n---------\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T13:40:19-03:00",
          "tree_id": "d0c5bca35bb3181c153324715abe094aaed2a952",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/e9f2c20d52b092c7b8c98c5c78d7f6f97d92a65f"
        },
        "date": 1793551250507,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 11116,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 10926,
            "range": "± 91",
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
          "id": "859b3d154c7934f67077a5f1ec4da9cc2ee89f21",
          "message": "deps: Cranelift 0.137 port + MSRV 1.97 + lint fixes (#463)\n\n* build(jit): move Cranelift to 0.137 and port the leaf JIT (supersedes #445-#448)\n\nBumps cranelift-codegen, -frontend, -jit and -module from 0.104.3 to 0.137.0,\nas in #445-#448, and ports crates/rd-jni/src/jit.rs to the 0.137 API:\n\n- MemFlags::trusted() is gone. Use MemFlagsData::new().with_notrap().with_aligned(),\n  which has the same meaning.\n- FunctionBuilder::finalize() takes module.target_config().\n- declare_var(ty) returns the Variable. The manual Variable::from_u32 indices are gone.\n- The *_imm builders are deprecated. Use the *_imm_s variants. They sign-extend,\n  as the old i64 immediates did, so behavior is unchanged.\n\nValidation: cargo test -p rd-jni --features jit (7 passed), cargo test -p\nrd-vm --features jit --test jit (10 passed), cargo clippy --workspace\n--all-targets --features rd-cli/jit -- -D warnings (exit 0), cargo fmt --check (exit 0).\n\nBlocker, not addressed here: cranelift 0.137 declares rust-version 1.97.0. The\nworkspace MSRV is 1.76, and the policy ceiling is 1.85. The MSRV job runs\n--all-features, so it cannot pass with this bump. Needs an owner decision.\n\n* build(msrv): subir MSRV para 1.97 e liberar clap (destrava #463)\n\n- Cargo.toml: rust-version 1.76 -> 1.97. O Cranelift 0.137 exige 1.97.0.\n- Cargo.toml: remove o teto `clap < 4.6` (o próprio comentário pedia\n  para levantá-lo junto com a subida do MSRV). Cargo.lock: clap 4.6.7,\n  clap_lex 1.1.1; lockfile v4 (Cargo 1.97).\n- ci.yml: job \"Rust MSRV (1.97)\" usa toolchain 1.97.\n- GOVERNANCE.md, README.en.md, worklog.md: MSRV 1.97.0.\n- Removida a label `sentinel:dependency-major-approved`: não existe\n  mais no GitHub. Removidas a chave `dependency_major_label` do\n  policy.json e a linha correspondente em docs/rd-sentinel.md.\n\nVerificado localmente em rustc 1.97.0: `cargo check --workspace`,\n`cargo check -p rd-vm --features jit --locked`,\n`cargo check --workspace --all-features --locked`, `cargo fmt --check`.\nADRs antigos e CHANGELOG mantêm o texto histórico (1.76).\n\n* chore(lint): clippy do stable atual (1.99) sem alterar comportamento\n\nO job \"Rust (stable)\" passou a usar rustc/clippy 1.99, que tem lints a\nmais. Corrigidos os sites que quebram `-D warnings`, todos de forma\nequivalente:\n- `map_or(true, f)` em Option -> `is_none_or(f)` (rd-apk, rd-vm)\n- `x % n == 0` / `!= 0` -> `x.is_multiple_of(n)` (rd-ndk, rd-vm, rd-framework, testes)\n- `chunks_exact(N)` com N constante -> `as_chunks::<N>()` (rd-apk, rd-dex, rd-ndk, rd-vm)\n- `repeat().take()` -> `repeat_n()`; `!(a..b).contains(&x)` (clippy --fix)\n\nVerificado com clippy 1.97 `--workspace --all-targets --all-features\n--locked -- -D warnings` (exit 0), cargo fmt --check, e testes de\nrd-apk/rd-dex/rd-framework/rd-vm (lib + android_assets + wasm_bridge).\nOs 3 testes de rd-ndk/native exigem o NDK (golden/native/build.sh) e\nseguem dependendo do job nativo do CI.\n\n* chore(lint): as_chunks_mut e isolate_lowest_one (clippy 1.99)\n\nDois chunks_exact_mut(2) em android_sysservices -> as_chunks_mut::<2>(); e\nbound & -bound == bound em jrandom -> (bound as u32).isolate_lowest_one()\n== bound as u32 (mesma máscara bit a bit; 0 e i32::MIN preservados).\nVerificado: clippy 1.97 --all-features -D warnings exit 0; rd-vm lib passa.\n\n* fix(vm): restaura semântica NaN de score/coordenada (regressão do clippy --fix)\n\nO clippy --fix transformou comparações explícitas (x < 0.0 || x > 1.0) em\n!(a..=b).contains(&x). Para NaN as duas formas divergem: a versão AOSP\naceita NaN, o contains rejeitava. Restauradas as comparações explícitas em\np41_chooser_instance e p35c_location_convert_from_string, com allow\njustificado de clippy::manual_range_contains nessas duas funções.\n\nVerificado: android_stub_surfaces 109/109; clippy 1.97 all-features exit 0.\n\n---------\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T14:32:34-03:00",
          "tree_id": "63ea1f6cb22903f007427e15732fd525a248ce16",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/859b3d154c7934f67077a5f1ec4da9cc2ee89f21"
        },
        "date": 1793554374278,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3301,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3340,
            "range": "± 194",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3dffa50ecab1e278463ef0d11bc3dbb8b72111af",
          "message": "ci(deps)(deps): bump actions/deploy-pages from 4.0.5 to 5.0.1\n\nBumps [actions/deploy-pages](https://github.com/actions/deploy-pages) from 4.0.5 to 5.0.1.\n- [Release notes](https://github.com/actions/deploy-pages/releases)\n- [Commits](https://github.com/actions/deploy-pages/compare/d6db90164ac5ed86f2b6aed7e0febac5b3c0c03e...368f82528645a54fb793d4d04e342629a3f51346)\n\n---\nupdated-dependencies:\n- dependency-name: actions/deploy-pages\n  dependency-version: 5.0.1\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-01T14:39:17-03:00",
          "tree_id": "4f4ec5baa5d34dfaf02c6c53afbea9b567aaf977",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/3dffa50ecab1e278463ef0d11bc3dbb8b72111af"
        },
        "date": 1793554788725,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3571,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3563,
            "range": "± 15",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "19f2261425cc9bfd7a9ff2dbf129dfed068812ea",
          "message": "ci(deps)(deps): bump actions/configure-pages from 5.0.0 to 6.0.0\n\nBumps [actions/configure-pages](https://github.com/actions/configure-pages) from 5.0.0 to 6.0.0.\n- [Release notes](https://github.com/actions/configure-pages/releases)\n- [Commits](https://github.com/actions/configure-pages/compare/983d7736d9b0ae728b81ab479565c72886d7745b...45bfe0192ca1faeb007ade9deae92b16b8254a0d)\n\n---\nupdated-dependencies:\n- dependency-name: actions/configure-pages\n  dependency-version: 6.0.0\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-01T14:39:39-03:00",
          "tree_id": "5d4c4380e6832a6c341d46edd08503163273f4cf",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/19f2261425cc9bfd7a9ff2dbf129dfed068812ea"
        },
        "date": 1793554822420,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1821,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1804,
            "range": "± 16",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9601071ba9d91534e4534d3313c60a6253971dd9",
          "message": "ci(deps): bump actions/checkout from 4.4.0 to 7.0.1 (#452)\n\nBumps [actions/checkout](https://github.com/actions/checkout) from 4.4.0 to 7.0.1.\n- [Release notes](https://github.com/actions/checkout/releases)\n- [Changelog](https://github.com/actions/checkout/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/actions/checkout/compare/v4.4.0...3d3c42e5aac5ba805825da76410c181273ba90b1)\n\n---\nupdated-dependencies:\n- dependency-name: actions/checkout\n  dependency-version: 7.0.1\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-01T14:41:36-03:00",
          "tree_id": "2f36e3fe5a310f4077a0bfbb8caf222e9b463e4e",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/9601071ba9d91534e4534d3313c60a6253971dd9"
        },
        "date": 1793554917516,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2769,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2774,
            "range": "± 46",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9601071ba9d91534e4534d3313c60a6253971dd9",
          "message": "ci(deps): bump actions/checkout from 4.4.0 to 7.0.1 (#452)\n\nBumps [actions/checkout](https://github.com/actions/checkout) from 4.4.0 to 7.0.1.\n- [Release notes](https://github.com/actions/checkout/releases)\n- [Changelog](https://github.com/actions/checkout/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/actions/checkout/compare/v4.4.0...3d3c42e5aac5ba805825da76410c181273ba90b1)\n\n---\nupdated-dependencies:\n- dependency-name: actions/checkout\n  dependency-version: 7.0.1\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-01T14:41:36-03:00",
          "tree_id": "2f36e3fe5a310f4077a0bfbb8caf222e9b463e4e",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/9601071ba9d91534e4534d3313c60a6253971dd9"
        },
        "date": 1793555115580,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3563,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3560,
            "range": "± 17",
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
          "id": "80b4d72874355b6805d92c1e58f4947eb1b7f330",
          "message": "feat(vm): M11 S1b — APK de demonstração do engine WASM (golden) (#470)\n\nGolden de APK de verdade para o bridge io.rustydroid.wasm.WasmBridge (ADR 013\n§9–10): classes.dex e assets/demo.wasm são lidos do ZIP, a Activity chama o\nbridge em onCreate e o TextView `result` precisa mostrar \"5\" (add(2, 3)).\n\n- rd-testsupport::wasm_demo: gera APK, DEX e módulo WASM de forma determinística.\n- rd-vm tests/wasm_demo_golden.rs (feature wasm-engine): launch_app + getText.\n- golden/engines/wasm/README.md: como reproduzir e o que ainda falta.\n- docs: m11-wasm.md, ROADMAP (M11 parcial), worklog (ruído do gate de bench).\n\nPendente (S1b restante): suíte `rd test`, propagação da feature até rd-cli e\nligação no registro. `wasm` segue not_implemented.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T15:24:23-03:00",
          "tree_id": "955276e4ee249ac7ed31f20415f9611ff6d8d058",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/80b4d72874355b6805d92c1e58f4947eb1b7f330"
        },
        "date": 1793557489678,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3286,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3220,
            "range": "± 22",
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
          "id": "aa1f9451e7c9743e6298d4b287a637497e0111c4",
          "message": "feat(cli): M11 S1b — suíte rd test sobre o APK WASM de demonstração (#471)\n\n- rd-agent e rd-cli: feature `wasm-engine` (propaga para rd-vm). Off por padrão.\n- golden/engines/wasm/: wasm-demo.apk (gerado por rd-testsupport::wasm_demo) e\n  suite.json (install → launch → assert_text \"5\" em io.rustydroid.wasmdemo:id/result).\n- rd-testsupport: ArscFix::with_package, para o nome do pacote do ARSC bater com o\n  manifest (o resource_id observado é `pkg:id/nome`).\n- rd-vm tests: teste de reprodutibilidade (APK commitado == gerado).\n- CI (job Rust (stable)): roda wasm_bridge, wasm_demo_golden e `rd test` com\n  wasm-engine. Antes, nenhum job rodava estes testes.\n\nO registro (`wasm` = not_implemented) não muda neste PR.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T15:50:40-03:00",
          "tree_id": "c2a3a3c5300afbf19e8bc14e55a5ed4038d227f1",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/aa1f9451e7c9743e6298d4b287a637497e0111c4"
        },
        "date": 1793559058983,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3554,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3555,
            "range": "± 19",
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
          "id": "aa1f9451e7c9743e6298d4b287a637497e0111c4",
          "message": "feat(cli): M11 S1b — suíte rd test sobre o APK WASM de demonstração (#471)\n\n- rd-agent e rd-cli: feature `wasm-engine` (propaga para rd-vm). Off por padrão.\n- golden/engines/wasm/: wasm-demo.apk (gerado por rd-testsupport::wasm_demo) e\n  suite.json (install → launch → assert_text \"5\" em io.rustydroid.wasmdemo:id/result).\n- rd-testsupport: ArscFix::with_package, para o nome do pacote do ARSC bater com o\n  manifest (o resource_id observado é `pkg:id/nome`).\n- rd-vm tests: teste de reprodutibilidade (APK commitado == gerado).\n- CI (job Rust (stable)): roda wasm_bridge, wasm_demo_golden e `rd test` com\n  wasm-engine. Antes, nenhum job rodava estes testes.\n\nO registro (`wasm` = not_implemented) não muda neste PR.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T15:50:40-03:00",
          "tree_id": "c2a3a3c5300afbf19e8bc14e55a5ed4038d227f1",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/aa1f9451e7c9743e6298d4b287a637497e0111c4"
        },
        "date": 1793559432263,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2761,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2770,
            "range": "± 49",
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
          "id": "5088262039518917dab93e895ad09f2c205353bf",
          "message": "feat(engine): M11 S1c — adaptador wasm no registro e rd engine run wasm (#472)\n\n- rd-engine-registry: feature `wasm-execution` (liga LANG-12, rd-vm/wasm-engine\n  e o parser DEX) com `WasmProcess`: parse do APK, launch da Activity, a bridge\n  WasmBridge roda no onCreate e o relatório traz a UI.\n- Novo estado `partial_wasm` (\"WASM hospedado pelo RustyDroid via bridge\n  documentada\"); engine #28 sai de `not_implemented` em data/engines.toml.\n- `execution_available(id)` cobre native e wasm; contract.rs, registry tests e\n  engine_cli usam essa gate.\n- rd-cli: `wasm-engine` encaminha `rd-engine-registry/wasm-execution`.\n  `rd engine check wasm` = AVAILABLE_BOUNDED (execution_claimed=false);\n  `rd engine run wasm <apk>` executa o APK de demonstração e mostra `5`.\n- Docs regeradas (engine-report, compat-report, engine-spec), ROADMAP M11,\n  m11-wasm.md, golden README, appendix-g, ADR 013 adendo S1c, worklog.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T16:36:12-03:00",
          "tree_id": "40a54ed8e1bfca6efe7b603ec2d93a50ab2929b5",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5088262039518917dab93e895ad09f2c205353bf"
        },
        "date": 1793561800679,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3230,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3229,
            "range": "± 21",
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
          "id": "5088262039518917dab93e895ad09f2c205353bf",
          "message": "feat(engine): M11 S1c — adaptador wasm no registro e rd engine run wasm (#472)\n\n- rd-engine-registry: feature `wasm-execution` (liga LANG-12, rd-vm/wasm-engine\n  e o parser DEX) com `WasmProcess`: parse do APK, launch da Activity, a bridge\n  WasmBridge roda no onCreate e o relatório traz a UI.\n- Novo estado `partial_wasm` (\"WASM hospedado pelo RustyDroid via bridge\n  documentada\"); engine #28 sai de `not_implemented` em data/engines.toml.\n- `execution_available(id)` cobre native e wasm; contract.rs, registry tests e\n  engine_cli usam essa gate.\n- rd-cli: `wasm-engine` encaminha `rd-engine-registry/wasm-execution`.\n  `rd engine check wasm` = AVAILABLE_BOUNDED (execution_claimed=false);\n  `rd engine run wasm <apk>` executa o APK de demonstração e mostra `5`.\n- Docs regeradas (engine-report, compat-report, engine-spec), ROADMAP M11,\n  m11-wasm.md, golden README, appendix-g, ADR 013 adendo S1c, worklog.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T16:36:12-03:00",
          "tree_id": "40a54ed8e1bfca6efe7b603ec2d93a50ab2929b5",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5088262039518917dab93e895ad09f2c205353bf"
        },
        "date": 1793562237380,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3548,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3396,
            "range": "± 25",
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
          "id": "5088262039518917dab93e895ad09f2c205353bf",
          "message": "feat(engine): M11 S1c — adaptador wasm no registro e rd engine run wasm (#472)\n\n- rd-engine-registry: feature `wasm-execution` (liga LANG-12, rd-vm/wasm-engine\n  e o parser DEX) com `WasmProcess`: parse do APK, launch da Activity, a bridge\n  WasmBridge roda no onCreate e o relatório traz a UI.\n- Novo estado `partial_wasm` (\"WASM hospedado pelo RustyDroid via bridge\n  documentada\"); engine #28 sai de `not_implemented` em data/engines.toml.\n- `execution_available(id)` cobre native e wasm; contract.rs, registry tests e\n  engine_cli usam essa gate.\n- rd-cli: `wasm-engine` encaminha `rd-engine-registry/wasm-execution`.\n  `rd engine check wasm` = AVAILABLE_BOUNDED (execution_claimed=false);\n  `rd engine run wasm <apk>` executa o APK de demonstração e mostra `5`.\n- Docs regeradas (engine-report, compat-report, engine-spec), ROADMAP M11,\n  m11-wasm.md, golden README, appendix-g, ADR 013 adendo S1c, worklog.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T16:36:12-03:00",
          "tree_id": "40a54ed8e1bfca6efe7b603ec2d93a50ab2929b5",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5088262039518917dab93e895ad09f2c205353bf"
        },
        "date": 1793562313683,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2756,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2757,
            "range": "± 10",
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
          "id": "803781ad838030008249bbccad07b694e1e575ff",
          "message": "docs(adr): ADR 020 — M11 S2 React Native, estratégia proposta (#473)\n\nProposta para o slice S2 do M11 (LANG-05): opções de engine JS (embutida em\nRust com bundle de texto, intérprete de Hermes bytecode, Hermes nativo via\ncaminho ELF/JNI, ou adiar para Mono), recomendação de spike com medição antes\nde qualquer dependência nova, critério de \"rodar\" com evidência e os limites\nainda não resolvidos. Nenhum código de execução de RN nesta mudança.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T16:53:30-03:00",
          "tree_id": "fd3643c858117def5cc75febd8a93802be357cae",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/803781ad838030008249bbccad07b694e1e575ff"
        },
        "date": 1793562827858,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2836,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2862,
            "range": "± 37",
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
          "id": "5b7afaffea3fe81ae2035c2ba44efc0fb9afe43d",
          "message": "docs: registra 17 apps alvo ponta a ponta (docs/target-apps.md) (#474)\n\nRequisito do dono: lista de apps que devem rodar de ponta a ponta no RustyDroid.\nCada item tem origem, bloqueios com referência ao roadmap e status real\n(nenhum verificado como funcionando). Registra a política de aquisição (só OSS\ndo F-Droid é baixado; APK proprietário exige decisão do dono) e as decisões\npendentes: Lucky Patcher, GMS e apps \"a escolher\".\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T17:05:52-03:00",
          "tree_id": "0ccf6a111462df6d7c685e84fd0f94f6733f5fbe",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5b7afaffea3fe81ae2035c2ba44efc0fb9afe43d"
        },
        "date": 1793563582231,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3293,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3337,
            "range": "± 252",
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
          "id": "d228a62b4eaa2e252130d05becd606b461c31b1d",
          "message": "docs(target-apps): política de aquisição, Lucky Patcher em andamento, GMS último (#475)\n\nDecisões do dono (2026-11-01): APKs só para teste interno, nunca commitados\n(ficam fora do git, por caminho em variável de ambiente). Lucky Patcher: prosseguir.\nSuporte a Google Play Services fica como um dos últimos itens.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T19:54:58-03:00",
          "tree_id": "a9d8f655a15bbf3f1d1167aafba56d27142cb00b",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/d228a62b4eaa2e252130d05becd606b461c31b1d"
        },
        "date": 1793573718580,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3313,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3286,
            "range": "± 65",
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
          "id": "18516357ff88c166c61527925d9a9c2397f34d09",
          "message": "fix(vm): folga de 1 MiB no orçamento de pilha nativa (#469) (#476)\n\nO orçamento de `enter_stack_frame`/`check_native_stack` era `livre - 512 KiB`\nquando a VMA da pilha se funde com um mapeamento rw vizinho acima dela: o\nteto `half_vma` deixa de limitar e sobram 512 KiB para o último salto\nhost→host antes da próxima checagem. Com a reserva de 1 MiB, o orçamento\nmantém ≥ 1 MiB de folga abaixo dele. Na VMA exata de 2 MiB o teto `half_vma`\n(1 MiB) continua valendo, então a profundidade legítima não muda.\n\nTeste novo: `two_mib_thread_budget_keeps_one_mib_headroom_469` (unit).\n\nValidação: `cargo test -p rd-ndk -p rd-vm --features native` (exit 0),\nclippy `-D warnings` em rd-vm, `stack_overflow_313` 40/40 em loop.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T20:36:45-03:00",
          "tree_id": "8c688443002f67ba3f343390d148ae26ed77749a",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/18516357ff88c166c61527925d9a9c2397f34d09"
        },
        "date": 1793576232939,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3500,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3673,
            "range": "± 833",
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
          "id": "05f05d408be573b7d5400f2ce4865f3f51367471",
          "message": "perf(rd-apk): LazyArsc + LazyStringPool (ADR 014) (#206)\n\n* perf(rd-apk): LazyArsc + LazyStringPool — on-demand ARSC decode (ADR 014)\n\nImplements ADR 014 (ADR 008 'Resources' pillar) with measured evidence:\n\n- LazyStringPool (pool.rs): document behind one Arc<Vec<u8>>; pools are\n  logical regions (parse_shared: zero copies per pool on the ARSC path);\n  per-slot OnceLock cache with LAZY slot allocation; O(1) offsets-table\n  validation (per-offset rejection deferred to access: get -> None,\n  memoized — eager failed the whole parse on the first bad string);\n  get/len/is_empty/iter API compatible; AXML pools get lazy for free.\n- LazyArsc (arsc.rs): parse_owned(Vec) zero-extra-copy path (Apk::open);\n  whole-buffer Arc (Law 2; O(1) Arsc/Package clones); per-package walk\n  unchanged (same guards/breaks) building chunks: Vec<TypeChunkRef> +\n  pairs: Vec<(resid, chunk_idx)> (append + single sort — measured 4x\n  faster than BTreeMap-with-clones at 400k pairs); entries materialize\n  on demand per resid (1:1 port of the eager collector: same absolute\n  addresses, same truncation guards, same string pre-resolution) with\n  a bounded cache (cap 8192, reset on overflow — changes WHEN decode\n  happens, never WHAT is returned; Law 4).\n- New APIs: Package::resource_ids() (getIdentifier path), entries_for\n  (per-resid snapshot, pub), entries_snapshot() (diagnostics), synthetic\n  (tests). Package.entries field removed (audited: no external consumers\n  beyond the mechanical adaptations included here: lib.rs 1 line,\n  android_res.rs 3 lines, app_xml.rs test, 2 examples).\n\nMeasured (release, isolated Arsc::parse bench, APK-Tester v1.0.2):\n- torture-huge-arsc (40.9 MB ARSC, 410k strings, ~400k pairs):\n  parse 137.95 -> 32.34 ms (4.3x; ~550 -> ~130 ms emulated at moto-e5\n  factor); bench-process VmHWM 383 -> 137 MB (-64%).\n- Typical APKs: sub-ms before and after (never worse than eager).\n- PARITY: 56/56 suite APKs (32 engines + sdk ladder + parser-torture +\n  flagship-classic with 1,851 resids) — 36,771 resolve_string/name/value\n  checks over the union of resids vs the vendored eager reference (main\n  87a4145): ZERO divergences. Single documented index difference: one\n  truncated-entry resid (eager never inserted; lazy indexes; both\n  resolve None).\n\nGates: rd-apk 31/31; rd-vm lib 132/132 + app_xml 25/25 + app 10/10 +\nm8 13/13 + m9 8/8 + vm_exec 103/103; rd-framework 34/34; rd-agent\n84/84; rd-cli perf_cli 3/3 (perf gate) + app/inspect/engine/test/mcp\ngreen; fmt clean; clippy --workspace --all-targets -D warnings clean;\nMSRV 1.76 (std only, no new deps); unsafe_code forbid intact.\n\n[LOCK] rd-apk/src/arsc.rs + pool.rs (Z-Agent, per PR-Agent endorsement;\nArenaAgent transfer pending — no other files touched beyond the listed\nmechanical adaptations).\n\nAuthor: Z-Agent (LazyArsc front)\n\n* fuzz+docs: review deltas for #206 — resolve-based fuzzing + explicit Law 1 semantics\n\nPR #206 review (PR-Agent) requested two deltas; both addressed:\n\n1. fuzz_targets/arsc.rs now exercises the LAZY paths, not only the parse:\n   resolve_string/resolve_name/resolve_value over every resid of every\n   package (resource_ids bounded by input size). Covers on-demand string\n   decode (memoized None for malformed), complex bags and reference\n   chains — the ADR 014 fuzz requirement.\n\n2. ADR 014 gains an explicit 'Mudança de semântica de erro (Lei 1)'\n   section with a before/after table: malformed individual string used to\n   fail the whole parse (eager); now it is None at access (parse still\n   fails on header/offsets-table level errors). Valid APKs: identical\n   behavior (measured 0 divergences in 36,771 checks).\n\nAuthor: Z-Agent (LazyArsc front)\n\n* refactor(arsc): per-instance entry cache is a plain Mutex (drop redundant OnceLock)\n\nMutex::new(BTreeMap::new()) does not allocate, so wrapping it in OnceLock\nbought no laziness. The cache is owned by each Package (never global);\npoisoned lock still means 'skip cache, re-decode', never a panic.\n\nAI-assisted (PR-Agent).\n\n* rd-apk: clippy-clean budget counters and UTF-16 reader (#206)\n\nCI's stable toolchain flags fetch_update (renamed to try_update) and\nchunks_exact with a constant size. Budgets now use a small compare_exchange\nhelper (try_add_capped), which works from the MSRV to stable, and the UTF-16\nreader uses as_chunks::<2>(). Behaviour unchanged: rd-apk 48/48 lib.\n\n---------\n\nCo-authored-by: Z-Agent <z-agent@users.noreply.github.com>\nCo-authored-by: PR-Agent <pr-agent@users.noreply.github.com>\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T20:50:50-03:00",
          "tree_id": "8f1c14b7d17cc8616f9937e5542f849d90ca3eb8",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/05f05d408be573b7d5400f2ce4865f3f51367471"
        },
        "date": 1793577070907,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3562,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3588,
            "range": "± 104",
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
          "id": "156eedddbe2099d3a304b106c949c90ee4525e70",
          "message": "docs(target-apps): bloqueio de aquisição do Lucky Patcher (#477)\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T21:03:28-03:00",
          "tree_id": "92103a56553772cf357ada8f306911f112c3393e",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/156eedddbe2099d3a304b106c949c90ee4525e70"
        },
        "date": 1793577836210,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3286,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3223,
            "range": "± 35",
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
          "id": "56f72895ed18f1a04d089eaf35c90c1a90b479e1",
          "message": "fix(ndk): rejeitar AVX (VEX/EVEX) no worker Unicorn — falha fechada (#400) (#478)\n\n* fix(ndk): rejeitar instruções AVX (VEX/EVEX) no worker Unicorn (#400)\n\nO Unicorn 2.1.4 executa código VEX/EVEX de forma errada (floor(5.5)+0.25 virou\n0.25 no CI, #393). Agora o worker falha fechado com NOT_IMPLEMENTED ao encontrar\num prefixo VEX (C4/C5) ou EVEX (62), depois dos prefixos legados e REX, em vez de\ndevolver um valor errado em silêncio.\n\n- golden/native/build.sh gera x86_64-avx2.so (-march=x86-64-v3) como fixture\n  negativa; as fixtures de referência continuam em -march=x86-64.\n- Teste avx_encoded_guest_code_is_rejected_fail_closed_400: controle positivo\n  (baseline = 5.75) e rejeição da fixture AVX.\n- Causa raiz (Unicorn) não corrigida; limite documentado em docs/m9.md.\n\nValidação: cargo test -p rd-ndk -p rd-vm --features native (exit 0),\nclippy -p rd-ndk --features native --all-targets -D warnings, cargo fmt --check.\n\n* test(vm): fixtures JNI em C fixados em x86-64 (baseline) — CI usava v3/VEX (#400)\n\nRunners Ubuntu 26.04 têm x86-64-v3 como padrão do gcc; o worker agora rejeita\nVEX/EVEX com NOT_IMPLEMENTED. jni_318 e gc_host_and_jni compilam C sem -march,\nentão falhavam só no CI. Mesmo pin de golden/native/build.sh.\nReproduzido localmente com -march=x86-64-v3 (jni_318: NOT_IMPLEMENTED) e\nverificado com o pin (7/7 e 4/4).\n\n---------\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-01T21:34:32-03:00",
          "tree_id": "26db9a13ea31d761325c6fac5cc8e9a8ef871f59",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/56f72895ed18f1a04d089eaf35c90c1a90b479e1"
        },
        "date": 1793579690038,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3218,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3213,
            "range": "± 13",
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
          "id": "efdb3940b67f736a8ebb4a790192904f3fa8a704",
          "message": "docs(target-apps): instalador do Lucky Patcher verificado; app real ainda não obtido (#479)\n\n* docs(target-apps): APK do Lucky Patcher de chelpus.com rejeitado (package ru.aaaaabar.installer)\n\n* docs(target-apps): instalador do Lucky Patcher verificado; app real ainda não obtido\n\n---------\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-02T01:24:07-03:00",
          "tree_id": "f2beee81fccbb1fbf50abf777d8550545c1d0fcd",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/efdb3940b67f736a8ebb4a790192904f3fa8a704"
        },
        "date": 1793593467045,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3493,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3496,
            "range": "± 15",
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
          "id": "db99a7f59d0e156a73439c2ab3460ae5362dad68",
          "message": "rd-vm: SdkExtensions.getExtensionVersion e ArrayDeque.descendingIterator (#480)\n\n- SdkExtensions.getExtensionVersion(I)I responde 0 (extensão não fornecida pelo\n  dispositivo virtual). Destrava a inicialização do KeePassDX 4.5.6.\n- ArrayDeque.descendingIterator()Ljava/util/Iterator; devolve um snapshot\n  invertido, somente leitura. Destrava OnBackPressedDispatcher do KeePassDX.\n- Testes: sdk_extensions_host_static_reports_no_extension (app.rs) e\n  array_deque_descending_iterator_walks_back_to_front (vm_exec.rs).\n- docs/target-apps.md: launch medido de Gallery, Contacts e KeePassDX.\n  Nenhum app verificado como funcionando; Context.getDrawable(I) é o próximo\n  bloqueio comum.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-02T01:46:46-03:00",
          "tree_id": "a23934c3ffcc1faf23c2851d36fb420bd08314b9",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/db99a7f59d0e156a73439c2ab3460ae5362dad68"
        },
        "date": 1793594823385,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2761,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2765,
            "range": "± 11",
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
          "id": "5b6a1db4455d5a7bd29fa9ffbc3f3007feb47814",
          "message": "rd-vm: OnBackAnimationCallback <: OnBackInvokedCallback (launch VolumeScroll) (#481)\n\nAPI 34: OnBackAnimationCallback estende OnBackInvokedCallback. Sem essa aresta\nna tabela builtin_interfaces, o check-cast da classe anônima do AndroidX\n(OnBackPressedDispatcher$Api34Impl) lançava ClassCastException no launch do\nVolumeScroll 1.0.2.\n\nTeste: on_back_animation_callback_is_an_on_back_invoked_callback.\ndocs/target-apps.md: candidatos e blockers medidos de Auto-clicker (Klick'r)\ne Auto-scroll (VolumeScroll). Próximo bloqueio comum: Context.getDrawable(I).\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-02T02:53:10-03:00",
          "tree_id": "285b2e68412accdfceadc271b7da2346cfcb27de",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5b6a1db4455d5a7bd29fa9ffbc3f3007feb47814"
        },
        "date": 1793598817357,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2770,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2771,
            "range": "± 12",
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
          "id": "33e76456643e157458d97087d2626045498725db",
          "message": "docs(target-apps): fonte do Lucky Patcher checada (sem APK do pacote) (#482)\n\nA fonte aprovada pelo dono oferece só o LP Installer 12.10.8 (já rejeitado no\nPR #479) e versões antigas que devolvem 404. Nenhum APK com.dimonvideo.luckypatcher\nfoi obtido.\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-02T03:08:34-03:00",
          "tree_id": "94d8600d68551925a8136eb90952cf25d124f752",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/33e76456643e157458d97087d2626045498725db"
        },
        "date": 1793599730709,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1797,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1790,
            "range": "± 18",
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
          "id": "ca03090a5776b8362fea071b64c333aea726c974",
          "message": "fix(rd-dex): 32x decoding and iput-short names; versioned baksmali diff (M1); XAPK bundles; ADR 021/022 (#483)\n\nFive commits: M1 differential gate vs baksmali 2.5.2 (nightly), rd-dex 32x and iput-short fixes, README inspect example, XAPK bundle support (inspect + dex disasm over splits), ADR 021 (proposed) and ADR 022. CI: all 10 jobs green on 0e8c8cc.",
          "timestamp": "2026-11-02T17:11:16-03:00",
          "tree_id": "96a996750934ff8671131ca23d58bf34f7d630e1",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/ca03090a5776b8362fea071b64c333aea726c974"
        },
        "date": 1793650296169,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3236,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3237,
            "range": "± 17",
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
          "id": "f94a3e0cd698d232c0e489e420de1a895c7fbbdc",
          "message": "feat(rd-render): M4 — tiny-skia + shaping rustybuzz, golden de screenshot e fix SharedPreferencesImpl (#484)\n\n* docs(adr): ADR 019 aceito — tiny-skia + rustybuzz (M4)\n\nA spec já fixa tiny-skia e rustybuzz; a pré-condição de MSRV (1.76) não\nvale mais: o workspace está em 1.97. Escopo e limites documentados: sem\nbidi/RTL, sem fidelidade Android. Golden de regressão, não de referência.\n\n* fix(rd-vm): SharedPreferencesImpl é subtipo de SharedPreferences\n\ncheckcast SharedPreferences sobre o impl do host lançava\nClassCastException no cold start do F-Droid. Teste de regressão em\nclasspath::tests.\n\n* feat(rd-render): snapshot com tiny-skia e shaping rustybuzz (M4)\n\nRetângulos e bordas por caminho do tiny-skia; texto shaped pelo rustybuzz\nsobre DejaVu Sans embutida (assets/, com licença). O atlas de 32 px da\nCanvas da VM não muda.\n\n* test(rd-cli): golden de screenshot trivial-app (regressão, não Android)\n\nTrava o PNG de 'rd app shot' e o APK fixture contra golden/screens/.\nAtualização explícita com RD_UPDATE_GOLDEN=1.\n\n* docs(changelog): M4 tiny-skia, golden e fix SharedPreferencesImpl\n\n---------\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-02T18:32:11-03:00",
          "tree_id": "6f0a5c7097be103a2fd8f623b9334aebeefd5a2e",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/f94a3e0cd698d232c0e489e420de1a895c7fbbdc"
        },
        "date": 1793655157107,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3435,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3395,
            "range": "± 35",
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
          "id": "dfdae976d098ea5016fe99cf36e4856d269ec5b2",
          "message": "docs(adr): ADR 023 revisa ADR 001 para múltiplas sessões por processo (#485)\n\nEtapa 1 do layering (ADR 021, opção A) exige N engines no mesmo processo.\nA ADR 001 fixava uma sessão por processo; esta revisão define o modelo,\no isolamento entre sessões, o engine_id, o orçamento de RSS do processo\n(ADR 015) e a ordem: refactor de RunningApp antes de tools novas.\n\nDocs-only. Deve ser mergeada antes de qualquer código de layering.\n\nAI usage: rascunho gerado com apoio de IA e revisado contra os trechos\ncitados de mcp.rs, ADR 015, ADR 017 e ADR 021.\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-02T19:39:44-03:00",
          "tree_id": "995bd462d12eee73ae21bce6384672987b40bf35",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/dfdae976d098ea5016fe99cf36e4856d269ec5b2"
        },
        "date": 1793659205097,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3231,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3221,
            "range": "± 17",
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
          "id": "9e74d75ced34c93c0d43883187f400a38342edc1",
          "message": "feat(agent): instala XAPK sobre todos os splits (base executa) (#487)\n\n* feat(agent): install XAPK bundles over all splits\n\n`install` now parses every split of an .xapk and installs its base split.\nThe response carries `bundle.splits` (id, file, base, dex, native libs,\nabis) and `base_only_execution: true`, because DEX and native libs of the\nother splits are not loaded into the VM yet. An invalid split fails the\ninstall and leaves no app registered. XAPK never takes the snapshot\nshortcut, which only knows single APKs.\n\nRegression: rd-apk install_path_reports_every_split_and_uses_base and\nthe rd-agent tests install_xapk_validates_all_splits_and_runs_the_base and\ninstall_xapk_with_invalid_split_fails_closed.\n\nAI usage: code and tests written with AI assistance; the fixture-based\nsuite was validated with `rd test` on a real two-split XAPK built from\nthe golden NewPipe and Tasks APKs.\n\n* docs: record XAPK install status and split limits\n\nROADMAP: XAPK install over splits is done, with the honest limit that\nexecution is base-only. CHANGELOG: entry for the install change.\n\n---------\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-02T19:39:50-03:00",
          "tree_id": "759b5a70fae9e1cecc5752819d7b691d9a0a4614",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/9e74d75ced34c93c0d43883187f400a38342edc1"
        },
        "date": 1793659242871,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3207,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3195,
            "range": "± 15",
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
          "id": "409b4deb049cf4da81acb2ebe951ec30cf228244",
          "message": "docs(roadmap): layering etapa 1 aceita, ADR 023 mergeada (#488)\n\nAtualiza o status de layering: dono escolheu a etapa 1 (sessões), ADR 023\nrevisa ADR 001 e está mergeada. Código da etapa 1 ainda não iniciado.\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-02T19:48:33-03:00",
          "tree_id": "55891f8570a2c6ba38663fdf4c3c283eb0027254",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/409b4deb049cf4da81acb2ebe951ec30cf228244"
        },
        "date": 1793659743120,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2779,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2764,
            "range": "± 20",
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
          "id": "e5bda69f82a3e62b90fcdb87e3ead0e6800d0de4",
          "message": "refactor(agent): extract RunningApp from AgentSession (#489)\n\nMoves the per-app execution state (engine, engine_package,\nnative_engine, activity, pending_snapshot, screen, ui_epoch) into a\nRunningApp struct owned by AgentSession as `running`. This is step 2 of\nADR 023 (layering stage 1) and is behavior-preserving: no tool, result\nor field semantics change. Tests that inspect the state read\n`s.running.*`.\n\nValidated: cargo test -p rd-agent (95 + 2 + 3 passed), clippy with\n-D warnings for the workspace, fmt.\n\nAI usage: refactor written with AI assistance using mechanical\nrename plus compiler-guided fixes; the rename was checked against each\ncompile error and the full rd-agent test suite.\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-02T20:01:50-03:00",
          "tree_id": "c04cff569fc4deb8ae709196274012b9e824940f",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/e5bda69f82a3e62b90fcdb87e3ead0e6800d0de4"
        },
        "date": 1793660539450,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1720,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1729,
            "range": "± 25",
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
          "id": "22ad2836da814274e3011807144a9171b2395225",
          "message": "fix(rd-vm): scope the WorkRequest builder shim to WorkRequest builders (#490)\n\nThe M8 shim claimed every androidx.work class for `<init>` and `build`.\nSo Configuration$Builder.build() returned a WorkRequest instead of a\nConfiguration. On the F-Droid cold start, WorkManagerImplExtKt.\ncreateWorkManager$default then called Configuration.getTaskExecutor on a\nWorkRequest receiver and failed with NOT_FOUND. The same over-broad\nclaim turned Configuration$Builder.<init> into a no-op.\n\nThe shim now applies only to WorkRequest builders (base, OneTime,\nPeriodic). Other androidx.work classes run the real DEX.\n\nValidated: unit test shim_claims_only_workrequest_builders; clippy\n-D warnings for rd-vm; rd app shot on the F-Droid APK moves past\ngetTaskExecutor and now stops at JobScheduler.forNamespace (next gap).\n\nAI usage: diagnosis and fix written with AI assistance from the VM trace\nand the disassembly of WorkManagerImplExtKt in classes.dex; the\nregression test was checked against the real builder class names.\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-02T20:04:29-03:00",
          "tree_id": "4e05536f95c5755075748ad637569555b86e066b",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/22ad2836da814274e3011807144a9171b2395225"
        },
        "date": 1793660693574,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3263,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3247,
            "range": "± 14",
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
          "id": "7fd374bd7e273c4f350d5737495185f2fa128777",
          "message": "docs(adr): ADR 024 thread-local inventory and path to concurrent engines (#491)\n\nInventories the host thread-local state (9 modules, 1 process static) and\ndecides the path for ADR 023 sessions: migrate ObjRef-keyed tables and\nAPP_STDOUT into HostState (option A), rejecting thread-per-engine and\nswap-on-entry. Complements ADR 017 follow-up.\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-03T03:32:05-03:00",
          "tree_id": "53abf44f0c6c24b9e630e7af6762a4f89078335a",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/7fd374bd7e273c4f350d5737495185f2fa128777"
        },
        "date": 1793687551765,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3433,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3435,
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
          "id": "dcd2e7d3c9c4bc35836edc0fa8b4bb826ce5313e",
          "message": "fix(rd-vm): F-Droid WorkManager init chain — Room _Impl, executor thread identity (#492)\n\n- Room builder instantiates <DB>_Impl when the generated class exists in the\n  DEX (Room convention), falling back to the abstract DB class.\n- JobScheduler.forNamespace arm and Context.isDeviceProtectedStorage (false).\n- Tasks of an inline Executor report a worker Thread for currentThread and\n  null for Looper.myLooper, so Room's main-thread guard no longer fires.\n  Executors still run inline; real concurrency is not implemented.\n\nRegression tests: executor_inline_task_has_no_looper_outside_main_thread\n(fails with the fix reverted), room_impl_descriptor unit test.\n\nKnown remaining blocker: RoomDatabase.connectionManager is not initialized,\nbecause the VM does not run RoomDatabase.init(DatabaseConfiguration).\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-03T03:32:08-03:00",
          "tree_id": "c078b66851395289817a1464a2b33df37eb8c24a",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/dcd2e7d3c9c4bc35836edc0fa8b4bb826ce5313e"
        },
        "date": 1793687570664,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3335,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3334,
            "range": "± 64",
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
          "id": "b1a1451b3687b4006d5576280db6cbf4ff57e20a",
          "message": "feat(xapk): merge split DEX into execution (ADR 022) (#493)\n\nThe install already validated every XAPK split, but execution mounted only\nthe base DEX. Apk::split_dex now carries the DEX of the other splits with\ntheir own zip; dexes_of_apk mounts base first, then split DEX in manifest\norder. The install response reports bundle.split_dex_merged and\nnative_libs_pending; base_only_execution stays true only while split native\nlibs are still not loaded.\n\nTests: rd-apk into_base_carries_split_dex_for_execution;\nrd-agent install_xapk_mounts_split_dex_into_execution.\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-03T04:07:24-03:00",
          "tree_id": "aa81a2b3f6b7c27a6d5dda06682d3113236247e6",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/b1a1451b3687b4006d5576280db6cbf4ff57e20a"
        },
        "date": 1793689679179,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3247,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3193,
            "range": "± 16",
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
          "id": "75b4775a13006e7dbebcfaf2eaa9934d2cdb1a73",
          "message": "docs(roadmap): M1 fuzz smoke result and F-Droid Room blocker (2026-11-03) (#494)\n\n- M1: 90 s smoke per target (5 targets, run sequentially because the\n  parallel script refused with 1.35 GB free RAM): 8.3 M executions,\n  0 crashes. The 24 h campaign is still not run.\n- F-Droid: WorkManager Room path advanced (#492); next blocker is\n  RoomDatabase.connectionManager, which needs a host SQLite driver\n  decision (ADR before code).\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-03T04:18:26-03:00",
          "tree_id": "c25c0d5e0906f7d9381115fdf6197255ef4f1212",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/75b4775a13006e7dbebcfaf2eaa9934d2cdb1a73"
        },
        "date": 1793690325143,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1852,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1830,
            "range": "± 20",
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
          "id": "cb0f2a4d66fc883238499ae0da4cd482f3f42675",
          "message": "refactor(rd-vm): System.out destination lives in the engine (ADR 024, step 1) (#495)\n\nThe System.out destination was a process-global static, so sessions\nsharing a process could write into each other's buffers. It now lives in\nHostState: Engine::set_app_stdout sets it per engine. The static becomes the\ndefault that only seeds Engine::new (set_default_app_stdout, used by the CLI\nand the native registry). The agent sets each session's log buffer on every\nengine it creates, native or DEX.\n\nRegression test: system_out_is_per_engine_not_process_global (two engines\nin one thread, each with its own buffer). Verified: making host_print ignore\nthe engine mode fails the test.\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-03T04:52:35-03:00",
          "tree_id": "d6d2a8087bd524353f13dc9d78c96e6ece64674c",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/cb0f2a4d66fc883238499ae0da4cd482f3f42675"
        },
        "date": 1793692388072,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3556,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3557,
            "range": "± 16",
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
          "id": "2ce8b7438b2269125091e908fc7a12d0ea445a3c",
          "message": "docs(adr): ADR 025 Room runtime on the framework SQLite; ADR 024 status (#496)\n\nADR 025 (proposed): decides how the VM runs Room's init. The F-Droid DEX\nuses FrameworkSQLiteOpenHelperFactory and has no bundled/Android driver\nreferences, so the plan is the real RoomDatabase.init over the framework\nSQLite (ADR 005), in three test-gated steps. Stop-and-review rule if coroutine\nsupport is not enough. Code comes after this ADR is accepted.\n\nADR 024: records that step 1 (System.out per engine) is implemented.\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-03T04:52:38-03:00",
          "tree_id": "eac6bf8aa7b9708a8d1730b3643f2601315dd9e3",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/2ce8b7438b2269125091e908fc7a12d0ea445a3c"
        },
        "date": 1793692409541,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3379,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3387,
            "range": "± 30",
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
          "id": "f593cbe86636a3535c5aee295799e3fd5f4cc340",
          "message": "refactor(rd-vm): ObjRef-keyed host tables live in HostState (ADR 024, step 2) (#497)\n\nPasso 2 do ADR 024. Ver a descrição do PR #497.",
          "timestamp": "2026-11-03T06:28:56-03:00",
          "tree_id": "43644cd6b1a6d0e31e36a3df551444f71e488a01",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/f593cbe86636a3535c5aee295799e3fd5f4cc340"
        },
        "date": 1793698162147,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3240,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3298,
            "range": "± 395",
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
          "id": "3b7185de594f2866b4ea3905f4d3d6e213c5935e",
          "message": "refactor(rd-vm): counters and UUID RNG live in HostState; reset_host_thread_state removed (ADR 024, step 3) (#498)\n\n- jrandom: the uniquifier of unseeded Random() moves from thread_local to\n  Tables.uniq.\n- xml_res: PARSER_HOST_BYTES moves from thread_local to Tables.host_bytes,\n  the host-byte budget of parsers (#370).\n- juuid: the randomUUID RNG moves to HostState.uuid. Each Engine starts at\n  RD_UUID_SEED, independent of earlier engines on the thread (PRF-04).\n\nNo thread-local host state is left except jfile's VFS (step 4), so\nreset_host_thread_state is removed. Its callers (Engine::new and the MCP\nengine teardown) no longer need it.\n\nTests: uuid_sequence_belongs_to_its_engine and\nunseeded_uniquifier_belongs_to_its_engine.\n\nAI usage: drafted with an AI agent, reviewed by the author with cargo\ncheck, clippy and the rd-vm suite (873 passed).\n\nCo-authored-by: Xisto <xisto@example.com>",
          "timestamp": "2026-11-03T06:48:10-03:00",
          "tree_id": "9f1321bbe5530f265c582c3c332e36132f93e2b0",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/3b7185de594f2866b4ea3905f4d3d6e213c5935e"
        },
        "date": 1793699318138,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1850,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1945,
            "range": "± 359",
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
          "id": "e983e028fd9f893f8db62e365150ddb17a7b2f1c",
          "message": "refactor(rd-vm): VFS belongs to the session, not the thread (ADR 024, step 4) (#499)\n\nThe jfile virtual filesystem was a thread_local. Every Engine on a thread\nshared it, and AgentSession::new reset it. This moves it into a session\nhandle:\n\n- `jfile::VfsHandle` (Arc<Mutex<Vfs>>), default `HostState.vfs`.\n- `Engine::set_vfs(handle)` links an engine to its session's disk.\n- `AgentSession` owns a handle, creates it with `VfsHandle::new()` and\n  links every engine it creates to it. `reset_vfs` is removed.\n- `clear_package_data(&handle, pkg)` clears only that package in that\n  session.\n- Free helpers (`os_*`, `read_bytes`, `write_bytes`, `touch`,\n  `mkdirs_pub`, `delete_file`, ...) take an explicit `&VfsHandle`.\n\nRelaunch semantics are kept: a new Engine on the same handle sees the\nfiles written earlier (ADR 017 item 5). Two sessions do not see each\nother's files.\n\nTests:\n- jfile: sessions_do_not_share_files, package_clear_in_one_session_leaves_others_alone,\n  relaunch_in_same_session_keeps_files (two real Engines, one session;\n  a third engine on another session sees nothing).\n- m8: sqlite_named_database_persists_across_engine_launches and\n  shared_preferences_named_stores_persist_across_engine_launches now\n  launch their two engines in one session (`engine_in`). Assertions are\n  unchanged.\n\nADR 024 step 4 is marked done; ADR 017 gets a supersession note for item 4.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T07:18:19-03:00",
          "tree_id": "49051e2ba2e67b517286770bebb26528f7de7767",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/e983e028fd9f893f8db62e365150ddb17a7b2f1c"
        },
        "date": 1793701131011,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2759,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2758,
            "range": "± 13",
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
          "id": "705d5aeefeef38f10f7bc40587ce523ca89b84ab",
          "message": "docs(adr): ADR 025 step 1 findings from the F-Droid DEX; ADR 024 marked complete (#500)\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T07:28:37-03:00",
          "tree_id": "cffc1926dd892ab0fa492191837a786ead25dea3",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/705d5aeefeef38f10f7bc40587ce523ca89b84ab"
        },
        "date": 1793701745237,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3395,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3411,
            "range": "± 264",
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
          "id": "bb7fd5b997570adf8271d6ab91aa61feda0d2146",
          "message": "docs(adr): ADR 025 step 1 spike result — stopped at the coroutine decision point (#501)\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T07:40:11-03:00",
          "tree_id": "5f0b720428df95a2c3d57f8f9b4b841c8f9516fb",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/bb7fd5b997570adf8271d6ab91aa61feda0d2146"
        },
        "date": 1793702454202,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3281,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3288,
            "range": "± 21",
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
          "id": "5d9c0818fa980dfe5d8c59bc32a3ce714e44315e",
          "message": "feat(rd-vm): ADR 025 etapa 1 — Room init real sobre o SQLite de framework (#502)\n\nApp.onCreate do F-Droid passa a rodar até o fim: o WorkDatabase (Room)\nexecuta RoomDatabase.init de verdade, e o WorkManager enfileira o trabalho\nperiódico de repositório.\n\n- room_init: queryCoroutineContext fica nulo, como no Room quando o builder\n  não recebe contexto. Com EmptyCoroutineContext, o init entrava no ramo de\n  dispatcher, e get(Key) devolvia const/4 0 (Int(0) no VM) -> NPE em getClass.\n- WorkRequest$Builder: setters fluentes devolvem o builder (sig_returns_builder);\n  build() de PeriodicWorkRequest/OneTimeWorkRequest devolve o subtipo.\n- NotificationChannel: setGroup(null) é válido; setShowBadge/canShowBadge e\n  setLightColor/getLightColor modelados; setSound/setVibrationPattern/\n  setConversationId aceitam o argumento sem efeito (sem áudio/vibração no host).\n- Intrínseco kotlin Reflection.getOrCreateKotlinClass -> ClassReference, como\n  o stdlib faz sem kotlin-reflect.\n- Caminho A1 do ADR 025 executado; nenhum intrínseco de coroutines foi\n  necessário (JobSupport roda, $FU não é nulo). Contingência B não acionada.\n\nTestes: unitários em m8 (sig_returns_builder, work_request_product) e\ncrates/rd-cli/tests/fdroid_oncreate_e2e.rs (regressão do onCreate; pula sem\no APK golden, como readme_inspect).\n\nAI usage: código e docs escritos com apoio de IA (Agent Mode), revisados\npelos testes listados acima.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T08:36:58-03:00",
          "tree_id": "dde7396d08998c6c86af7d2c7e5f1de295e65a58",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5d9c0818fa980dfe5d8c59bc32a3ce714e44315e"
        },
        "date": 1793705837053,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3211,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3192,
            "range": "± 17",
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
          "id": "66e256445759360a422a363e344f7e41a476f46f",
          "message": "feat(rd-vm): ADR 025 etapa 1 — Room init real sobre o SQLite de framework (#503)\n\n- Novo E2E `rd_test_runs_two_scenarios_over_an_xapk_bundle_with_split_dex`:\n  bundle com base e split de feature (DEX próprio); cenário 1 instala, lança\n  e confere a tela; cenário 2 reinstala o mesmo bundle e confere a tela.\n- ADR 022: DEX de splits entra na execução; libs nativas de split ficam\n  base-only por decisão, declaradas por `native_libs_pending` /\n  `base_only_execution`. Corrige a consequência que dizia que install e\n  `rd test` operavam só no split base.\n- ROADMAP: status do XAPK atualizado.\n\nAI usage: teste e docs escritos com apoio de IA; o teste foi executado localmente\n(`cargo test -p rd-cli --test test_cli`, 5 passed).\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T08:45:16-03:00",
          "tree_id": "73e03d938a4ba0e389ccf0d53d6b8cccb8a7eb60",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/66e256445759360a422a363e344f7e41a476f46f"
        },
        "date": 1793706336343,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2763,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2761,
            "range": "± 13",
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
          "id": "da3d1ded38a314d4a9ef6f2ab5327f3fc69104e3",
          "message": "fix(rd-vm): virtual_hardware_enabled vale no despacho de bytecode (#229) (#505)\n\nO ponto 3 da #229 era o único aberto: intrinsics.rs consultava o flag, interp.rs não. Gate por classe (m10::is_virtual_hardware_class), não por módulo — os módulos M10 misturam hardware e software. ADR 026. Testes: m10_gate_229.rs (probe em bytecode DEX real).",
          "timestamp": "2026-11-03T09:04:35-03:00",
          "tree_id": "f8629233dba6c96a241e78d0e342353e20a07e6e",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/da3d1ded38a314d4a9ef6f2ab5327f3fc69104e3"
        },
        "date": 1793707496497,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3325,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3335,
            "range": "± 86",
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
          "id": "da3d1ded38a314d4a9ef6f2ab5327f3fc69104e3",
          "message": "fix(rd-vm): virtual_hardware_enabled vale no despacho de bytecode (#229) (#505)\n\nO ponto 3 da #229 era o único aberto: intrinsics.rs consultava o flag, interp.rs não. Gate por classe (m10::is_virtual_hardware_class), não por módulo — os módulos M10 misturam hardware e software. ADR 026. Testes: m10_gate_229.rs (probe em bytecode DEX real).",
          "timestamp": "2026-11-03T09:04:35-03:00",
          "tree_id": "f8629233dba6c96a241e78d0e342353e20a07e6e",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/da3d1ded38a314d4a9ef6f2ab5327f3fc69104e3"
        },
        "date": 1793707573561,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3202,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3194,
            "range": "± 18",
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
          "id": "1612354df52bba43130892b12c02972b764b1f89",
          "message": "feat(vm): NewPipe cold start — binarySearch, field hiding, cause chain (#506)\n\nCollections.binarySearch over List; InflateException in the builtin hierarchy; instance fields obey Java field hiding (JLS 8.3); Caused by chain on uncaught exceptions. Each one is the real blocker found by running the golden NewPipe APK end to end.",
          "timestamp": "2026-11-03T09:07:24-03:00",
          "tree_id": "8fa3325e78e6d39cea13b17391a54e48b4c72bef",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/1612354df52bba43130892b12c02972b764b1f89"
        },
        "date": 1793707671436,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3135,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3143,
            "range": "± 18",
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
          "id": "839a3a3bea06a5e58b3ae1b75dc56864d86286ac",
          "message": "feat(res): ARSC array resources + TypedArray.getTextArray (#507)\n\nArsc::resolve_array decodes aapt/aapt2 array item naming and follows the parent chain; TypedArray.getTextArray materializes the CharSequence[] (string/int/bool/float items, references dereferenced). NewPipe now finishes App.onCreate.",
          "timestamp": "2026-11-03T09:07:47-03:00",
          "tree_id": "d18b12a55be9b3e55529a250bfc4159a3e5ed2ef",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/839a3a3bea06a5e58b3ae1b75dc56864d86286ac"
        },
        "date": 1793707702311,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2768,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2776,
            "range": "± 41",
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
          "id": "5499b599e5dbe522f24b98a7c96599a942d80b78",
          "message": "feat(rd-agent): sessões de app no MCP (ADR 023, etapa 1) (#510)\n\nUm processo rd mcp hospeda até N sessões (RD_MCP_MAX_SESSIONS, padrão 2).\nNovas tools session_open, session_list, session_close; tools de app aceitam\nengine_id opcional. Sem engine_id, a tool usa a sessão padrão.\n\nCatálogo MCP 47 -> 50. Contratos, docs gerados e gate native-languages\natualizados. Desvios e pendências (roteamento por M10, orçamento de\nmemória) declarados na ADR 023.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T09:24:39-03:00",
          "tree_id": "82decdc047da7929e44adc27f56d1290291f1132",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5499b599e5dbe522f24b98a7c96599a942d80b78"
        },
        "date": 1793708716107,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3301,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3300,
            "range": "± 8",
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
          "id": "bcdf0b97dc7bc9a5cae53f141ff7a252d421a1fa",
          "message": "feat(vm): Collections completion, javax.net socket factories, ObjectStreamField + NewPipe e2e (#509)\n\nCollections: reverseOrder(+Comparator), nCopies, frequency, fill, swap, rotate, copy, disjoint, replaceAll. jnet: SocketFactory/SSLSocketFactory construction exists, createSocket answers the structured network gap. ObjectStreamField metadata. e2e test pins the NewPipe gaps closed in #506/#507/#509.",
          "timestamp": "2026-11-03T09:31:08-03:00",
          "tree_id": "563c9c24b87e0b8107608a75c00a2365c60dccc1",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/bcdf0b97dc7bc9a5cae53f141ff7a252d421a1fa"
        },
        "date": 1793709088871,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3231,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3211,
            "range": "± 43",
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
          "id": "4003a6d01c332039955039a0fcc11379d6b1f548",
          "message": "docs(roadmap): ADR 023 etapa 1 mergeada (#510) (#511)\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T09:36:36-03:00",
          "tree_id": "58d5bb8927a4e68ec3a30b62ba06b1cae1b0316d",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/4003a6d01c332039955039a0fcc11379d6b1f548"
        },
        "date": 1793709441595,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3564,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3669,
            "range": "± 308",
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
          "id": "4d0123b62eab1b50f7f213761ad2c2c1e68b34eb",
          "message": "fix(vm): sun.misc.Unsafe operands were off by one (#512)\n\nThe receiver is not part of the intrinsic args, so putInt/putLong/putObject stored a default and compareAndSetInt/Long/Object compared the wrong word and always answered false. okhttp's ConcurrentHashMap.initTable() then spun until the fuel budget died inside Application.onCreate (NewPipe). Added arrayBaseOffset/arrayIndexScale/addressSize/pageSize with a non-colliding array offset space.",
          "timestamp": "2026-11-03T12:38:19-03:00",
          "tree_id": "b241f92048015dac725fbdb0506e2d20ae269739",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/4d0123b62eab1b50f7f213761ad2c2c1e68b34eb"
        },
        "date": 1793720322309,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3254,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3194,
            "range": "± 25",
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
          "id": "31b403a899264e5d653eab7a2833b72b0dd70bd5",
          "message": "feat(rd-agent): orçamento de heap por sessão e roteador de entrada (ADR 023, decisões 3 e 5) (#513)\n\nCada sessão recebe min(ram_app_heap_mb, ram_process_cap_mb / N) de heap; a\nsoma das sessões nunca passa do teto do processo. session_info mostra o\norçamento. Toda tool que injeta entrada passa pelo InputRouter do hub, que\nregistra sessão de destino, ordem e aceitação; session_list expõe\nlast_input_seq. O roteador fica no hub, não no M10 (o M10 não tem roteador):\ndeclarado na ADR 023.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T12:44:22-03:00",
          "tree_id": "6c79f7088b19cf515570e7dfabade98d7ebebbbc",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/31b403a899264e5d653eab7a2833b72b0dd70bd5"
        },
        "date": 1793720706892,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3275,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3264,
            "range": "± 102",
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
          "id": "ebb332a1ed840ea0ab849526821325844966d66d",
          "message": "feat(vm): java.util.ResourceBundle (class-based bundles) (#514)\n\ngetBundle with the JDK candidate order, getObject/handleGetObject/getString/getStringArray/containsKey over the guest getContents(), MissingResourceException naming the bundle. ListResourceBundle is-a ResourceBundle in the builtin hierarchy. NewPipe's App.onCreate now runs ~236 DEX methods and stops at NotificationChannelCompat.setGroup.",
          "timestamp": "2026-11-03T12:49:50-03:00",
          "tree_id": "2b5aaea71738f7bbaa6e44e6dfea709520fa5067",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/ebb332a1ed840ea0ab849526821325844966d66d"
        },
        "date": 1793721013012,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3593,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3594,
            "range": "± 19",
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
          "id": "cdcbfd38125f4a6aff4c85822bf5818bfc883d3f",
          "message": "docs(roadmap): ADR 023 decisões 3 e 5 implementadas no hub (#513) (#515)\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T12:56:13-03:00",
          "tree_id": "b4073ff116d2a80b9477033d7aa7e9f4d3ed7646",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/cdcbfd38125f4a6aff4c85822bf5818bfc883d3f"
        },
        "date": 1793721397783,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3560,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3645,
            "range": "± 338",
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
          "id": "be2863087c30f1ad082c81f6388e990007ba52dc",
          "message": "fix(vm): NotificationChannel null literal + java.util.EnumSet.of (#516)\n\nThe DEX null literal (const/4 v0,0 -> Int(0)) was rejected by the NotificationChannel setters (AOSP accepts null; an unset name reads back as null). Channel.name/description are Option<String>. Plus java.util.EnumSet.of(1..5) as a host Set with the JDK's duplicate/null exceptions.",
          "timestamp": "2026-11-03T12:58:28-03:00",
          "tree_id": "ee4a646f4e7ec323338e3ce684bd64d55992cffe",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/be2863087c30f1ad082c81f6388e990007ba52dc"
        },
        "date": 1793721573843,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2760,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2756,
            "range": "± 11",
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
          "id": "4d2089a4a4e59b789266b7417d19dc9201c79191",
          "message": "feat(vm): java.util.EnumMap + RD_TRACE_CLINIT (#517)\n\nEnumMap shares the host Map model (put/get/remove/containsKey/size/isEmpty/clear/keySet/values/entrySet) with <init>(Class) and <init>(EnumMap), and java.util.Map in its builtin interface list. RD_TRACE_CLINIT=1 prints the class and original cause the first time a <clinit> fails -- the JLS 12.4.2 error alone hides it, and that trace is what found the missing constructor.",
          "timestamp": "2026-11-03T13:08:39-03:00",
          "tree_id": "0bab9b25b1be9d2d052ca2e51a7115e8ac3ad46d",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/4d2089a4a4e59b789266b7417d19dc9201c79191"
        },
        "date": 1793722145861,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2764,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2758,
            "range": "± 11",
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
          "id": "b5a557adde3c9f0af5093435fdd62c7096cb70b0",
          "message": "feat(vm): CookieManager, Boolean.getBoolean, Context.getDrawable (#518)\n\nCookieManager with the RFC 6265 subset APKs use (host-only vs Domain, Path, Secure refused over http, Max-Age on the virtual clock, path-most-specific-first, ValueCallback), Boolean.getBoolean over the system-property table, and Context.getDrawable resolving colors to ColorDrawable while naming the vector-drawable capability gap. With these, NewPipe's Application.onCreate runs to completion and execution reaches MainActivity.",
          "timestamp": "2026-11-03T13:17:40-03:00",
          "tree_id": "d671e2050b13cdb3750cf72e985586b38a1dc34a",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/b5a557adde3c9f0af5093435fdd62c7096cb70b0"
        },
        "date": 1793722696059,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 2760,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 2759,
            "range": "± 12",
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
          "id": "e4d038a3e3327af092dcabe78867755795f434c4",
          "message": "feat(vm): run the F-Droid/Compose launch path end to end (#519)\n\n* feat(vm): run the F-Droid/Compose launch path end to end\n\n`rd app run golden-apks/org.fdroid.fdroid.apk` now gets past Compose's\nfirst composition and the app's cold start instead of stopping at the\nfirst unimplemented platform class. Each gap below was reached from the\nreal app, not from a stub list.\n\nFramework surfaces F-Droid touches during startup:\n\n- `View.getAutofillId` returns a stable `AutofillId` per view\n  (`AndroidComposeView` reads it while attaching).\n- `Context.isUiContext`, `WindowManager.getCurrentWindowMetrics` /\n  `getMaximumWindowMetrics` and `WindowMetrics` (bounds/density/insets).\n- `StrictMode` policies as class-scoped singletons with identity\n  `equals` (Compose compares the policy it installed).\n- `PackageManager.getPackageInstaller` with real install sessions\n  (`createSession`/`openSession`/`Session.openWrite`/`commit` writing into\n  the VM's virtual disk) and `getSystemAvailableFeatures`, whose\n  array agrees with `hasSystemFeature`.\n- `Activity.setIntent` (F-Droid rewrites the launch intent in its router).\n\nNew `java.*` surfaces:\n\n- `java.nio.charset` coders + `CharBuffer` (jcharset.rs): incremental\n  UTF-8/UTF-16/ISO-8859-1/US-ASCII decoders that keep split code points\n  across reads, `CodingErrorAction`, `CoderResult`, encoder, and a\n  windowed `CharBuffer` (`wrap(char[],off,len)`, `slice`, position\n  state machine). `kotlinx.serialization`'s `CharsetReader` drives every\n  ktor JSON body through this.\n- `java.text.Collator`/`CollationKey` (jtext_collator.rs): three-level\n  collation (base letter, accent, case) with `setStrength`.\n- `java.util.concurrent.CountDownLatch` (threads.rs): guest threads park\n  and wake, the main thread drains ready threads until the latch opens.\n- `org.json` (org_json.rs): `JSONObject`/`JSONArray`/`JSONTokener` with\n  the lenient parsing Android accepts; F-Droid parses its index and\n  preferences with it.\n- `android.net` connectivity: an always-on validated Wi-Fi network\n  (`Network`, `NetworkCapabilities`, `NetworkInfo`).\n- `android.graphics.PathMeasure` and the `ColorFilter`/`Shader` class\n  hierarchies for `check-cast`.\n\nRuntime:\n\n- `VmConfig::app_runtime()` (1024-frame cap) for hosts on a normal\n  stack; the 128-frame default stays for 2 MiB threads (#313).\n- `rd app run` executes the app on a 32 MiB thread: an interpreter frame\n  is far larger than an ART frame, so Compose's legitimate ~400-frame\n  layout pass exhausted the 8 MiB main stack while staying below the\n  guest cap.\n- Guest `Iterator` implementations run their own DEX body (the host\n  funnel only answers for `HostObj::Iter`).\n- Static calls into classes absent from the DEX and not platform-provided\n  throw a catchable `NoClassDefFoundError` (androidx.window's sidecar\n  probe depends on it).\n- `TimeUnit.convert` finds its unit argument on both call paths.\n\nTests: `tests/compose_path_fdroid.rs` (autofill id, isUiContext, window\nmetrics, NCDFE, guest iterator, app_runtime), plus module tests for the\ndecoder (split code point, malformed actions), encoder round trip,\n`CharBuffer` windows, `compact`, collation strengths and keys.\n\nRemaining known gap: the launch now stops at `VM_TYPE_ERROR: esperado\nlong, got Null` during Compose's first measure/place pass.\n\nAI assistance: this change was produced with an AI coding agent (Xisto)\nunder human review; every behavioural claim above is backed by a test in\nthis PR or by the app run quoted in the commit.\n\n* fix(vm): packed colour longs keep ARGB in the low bits\n\nCompose's first measure/place pass died with `VM_TYPE_ERROR: esperado long,\ngot Null`: `Paint.getColorLong()` had no host implementation, so a `long`\nregister held `null` and the failure only surfaced later inside `binop`.\n\n- `Paint.getColorLong()` returns the AOSP packed form (colour-space id in\n  the high 32 bits, ARGB in the low 32 bits; sRGB is id 0).\n- `Color.pack(int)` / `Color.colorSpace(long)` / `Color.isSrgb` /\n  `Color.isWideGamut` / `Color.{alpha,red,green,blue}(long)` follow the same\n  packing.\n- `Paint.setColor(long)` read the colour from the *high* 32 bits, i.e. it\n  always installed the colour-space id (transparent black for sRGB). It now\n  reads the low 32 bits.\n\nWith this, `rd app run golden-apks/org.fdroid.fdroid.apk` completes the\nlaunch and prints the composed hierarchy:\n\n    FitWindowsLinearLayout @id/action_bar_root [0,0 720x1440] vertical\n      ViewStubCompat @id/action_mode_bar_stub\n      ContentFrameLayout @16908290 [0,0 720x1440]\n        ComposeView @0 [0,0 720x1440]\n          AndroidComposeView @0 [0,0 720x1440]\n\nTest: `graphics::tests::paint_color_long_round_trips_through_a_packed_color`.\n\nAI assistance: produced with an AI coding agent (Xisto) under human review;\nthe behaviour is pinned by the test above and by the app run quoted here.\n\n* fix(vm): MotionEvent.getClassification answers CLASSIFICATION_NONE\n\nThe first tap into the composed UI stopped at\n`NOT_FOUND: MotionEvent.getClassification()`. AOSP returns\n`CLASSIFICATION_NONE` (0) for synthetic finger input without a\nclassification, which is exactly the VM's touch model; Compose reads it on\nevery down event.\n\n`rd app run golden-apks/org.fdroid.fdroid.apk --script \"tap 360,700\"` now\ndelivers the tap (`listener acionado: true`) and still prints the composed\nhierarchy.\n\nAI assistance: produced with an AI coding agent (Xisto) under human review;\nbehaviour confirmed by the app run quoted above.\n\n* fix(vm): android.net constructors stay with their owning host module\n\n`android_net.rs` answered every `<init>` on `Landroid/net/*` with a\n`NoSuchMethodError`, but it only hosts `ConnectivityManager`,\n`Network*` and `NetworkInfo`. The rest of the package (IPsec/IKE, NSD,\nSIP, SSL, Wi-Fi) lives in `android_stub_surfaces`, which runs later in the\ndispatch chain — so those constructors could never be reached and 9 tests\nin `android_stub_surfaces` failed.\n\nDeclining unknown names makes the chain fall through, as every other host\nmodule does. `cargo test -p rd-vm` is green again (109 tests in that\ntarget, 107 test targets overall).\n\nAI assistance: produced with an AI coding agent (Xisto) under human review.\n\n* fix(vm): drop android_net's now-unused exception helper\n\n`exc` lost its only caller when `android_net` stopped claiming every\n`Landroid/net/*` constructor; `cargo clippy --workspace --all-targets -D\nwarnings` (and every job that runs it, including NDK/JNI, Cranelift JIT\nand Appendix G) then failed on `function exc is never used`.\n\nAI assistance: produced with an AI coding agent (Xisto) under human review.\n\n---------\n\nCo-authored-by: Xisto <xisto@users.noreply.github.com>",
          "timestamp": "2026-11-03T14:06:36-03:00",
          "tree_id": "ab34beaf0fa85313fc0be9b0161453dd4b693816",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/e4d038a3e3327af092dcabe78867755795f434c4"
        },
        "date": 1793725636695,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3352,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3441,
            "range": "± 353",
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
          "id": "ac917046e8ea56b57bb6ab2811c98316bd761757",
          "message": "fix(vm): ConcurrentHashMap(IFI), DataInputStream e deque em LinkedList (launch F-Droid) (#520)\n\n* fix(vm): ConcurrentHashMap(int,float,int), DataInputStream e deque em LinkedList\n\nPróximos gaps do launch do F-Droid (teste fdroid_oncreate_e2e, APK real):\n\n- ConcurrentHashMap.<init>(IFI)V: host map com as validações da JDK\n  (fator > 0, capacidade >= 0, nível de concorrência > 0).\n- java.io.DataInputStream: big-endian sobre qualquer InputStream (lê\n  byte a byte pelo read() do stream interno; EOF vira EOFException).\n  Caminho: BuiltInsBinaryVersion.readFrom do Kotlin reflect.\n- LinkedList: addFirst/addLast/removeFirst/... usam o estado List; nulls\n  são aceitos (ArrayDeque continua rejeitando).\n\nTestes de regressão unitários em intrinsics.rs e jio_chars.rs.\n\nAI usage: código e testes gerados com assistência de IA e validados\nlocalmente (workspace, clippy, golden suite no JDK 21).\n\n* docs: corrige o estado do launch do F-Droid (ADR 025, ROADMAP)\n\n---------\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T15:58:16-03:00",
          "tree_id": "ab3bf8a3f782e41da5f7df89a7ae183a0128718a",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/ac917046e8ea56b57bb6ab2811c98316bd761757"
        },
        "date": 1793732339813,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1710,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1724,
            "range": "± 65",
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
          "id": "021d0e6a6f04d44e35b56f1df530bee34b64857b",
          "message": "docs(roadmap): campanha curta de fuzz M1 (5 targets, 300 s cada) (#521)\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T16:27:32-03:00",
          "tree_id": "2e6f3d0be5913242941674b4280dce7429ee875f",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/021d0e6a6f04d44e35b56f1df530bee34b64857b"
        },
        "date": 1793734073902,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3452,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3453,
            "range": "± 83",
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
          "id": "5751d6b1b130e581c0e026be63dec7167928310b",
          "message": "fix(vm): F-Droid onCreate conclui e a primeira tela desenha (ADR 027) (#522)\n\n* docs(adr): ADR 027 — metadados de Class para reflexão do Kotlin, sem falsear\n\n* fix(vm): Class.getTypeParameters/isAnnotation/isSynthetic e AssertionError(Object) (launch F-Droid)\n\nFecha os próximos três gaps do caminho do Ktor pelo Kotlin reflect no\nApp.onCreate do F-Droid real (fdroid_oncreate_e2e passa):\n\n- Class.getTypeParameters(): array vazio só quando a classe é sabidamente\n  não genérica (array/primitivo; classe DEX sem <...> no atributo\n  dalvik.annotation.Signature; lista fechada de tipos de plataforma).\n  Qualquer outra classe, ou classe genérica, responde NOT_IMPLEMENTED.\n  Não há falsa resposta vazia para classes genéricas.\n- Class.isAnnotation() e isSynthetic(): flags ACC_ANNOTATION e\n  ACC_SYNTHETIC da definição DEX da classe.\n- AssertionError(Object): mensagem = String.valueOf(detail); Throwable\n  como detalhe vira causa. AssertionError entra na hierarquia de Error.\n\nTestes: launch_gap_tests (intrinsics.rs). Decisão em ADR 027.\n\nAI usage: código e testes gerados com assistência de IA e validados\nlocalmente (workspace 1451 passed, clippy, golden 150/150 no JDK 21).\n\n* docs: F-Droid onCreate conclui; próximos gaps de reflexão (ADR 025, ROADMAP)\n\n* fix(rd-cli): roda a CLI numa thread de 32 MiB; primeiro quadro do F-Droid desenha\n\nNa thread principal de 8 MiB, o orçamento de pilha nativa cortava o draw\ndo Compose em ~400 quadros (StackOverflowError, tela branca em rd app shot).\nA CLI inteira passa a rodar na mesma thread de 32 MiB que rd app run já\nusava. Todos os comandos encerram com process::exit; o join só volta em\npânico (código 101).\n\nTeste: fdroid_first_frame_draws_without_stack_overflow (APK real; pula sem\no APK, como o outro teste E2E).\n\nAI usage: código e teste com assistência de IA, validados localmente.\n\n* docs: primeiro quadro do F-Droid renderiza (ROADMAP, ADR 025)\n\n---------\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T17:37:21-03:00",
          "tree_id": "5d7738e53e563b404d41a4f66c49aec0c4773333",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5751d6b1b130e581c0e026be63dec7167928310b"
        },
        "date": 1793738274034,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3244,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3243,
            "range": "± 15",
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
          "id": "2f70bc47197fe5fab1493a2c8b9527a5faed91e1",
          "message": "perf(vm): memória compacta — Value 16 bytes, nomes internados, Insn menor (ADR 028) (#523)\n\n- Value::StrPlaceholder(Box<String>): Value de 24 para 16 bytes.\n- HeapObj::Instance usa Arc<str> para classe e nomes de campo, internados no\n  Heap (Heap::intern); free_slot reutiliza o nome do tombstone.\n- Kind::Payload(Box<Payload>) com Kind::payload(): Insn de 56 para 24 bytes,\n  entrada decodificada de 64 para 32.\n- rd-cli: caminho de execução usa Dex::parse_unverified (como o MCP).\n- RD_MEM_REPORT=1 no rd app shot: contadores do heap e do código decodificado.\n\nPrimeiro quadro do F-Droid: pico de RSS 196 096 -> 163 808 KB (-16,5%, debug),\nimagem idêntica. Medidas e limites no ADR 028 e em docs/gc.md.\n\nTestes: rd-dex lib 104 ok; rd-vm (lib + integração) 944 ok; golden VM 150/150\n(JDK 21); fdroid_oncreate_e2e 2 ok; clippy -D warnings limpo.\n\nAI usage: implementação e medições com assistência de IA; validado localmente.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-03T22:10:17-03:00",
          "tree_id": "1ed712e4bf3528a4bb26ed522d5c166e2e0b2152",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/2f70bc47197fe5fab1493a2c8b9527a5faed91e1"
        },
        "date": 1793754660749,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3519,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3351,
            "range": "± 19",
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
          "id": "2bad5248f01e9e8c07fc99b612a337cf13f2725c",
          "message": "docs(memory): composição do RSS e próximos passos (ADR 029, proposto) (#524)\n\n* docs(memory): composição do RSS e próximos passos (ADR 029, proposto)\n\nAmostragem de malloc no primeiro quadro do F-Droid (build debug): buffers de\nDEX ~14 MB, instruções decodificadas ~11 MB, índice de classes, Vec<Value>,\níndices de ids e caches. Ressalva: bytes pedidos, não residentes.\n\nRegistra as opções e a ordem dos próximos passos: contar construções de índice\nde classes, medir sessões no mesmo processo com DexBacking compartilhado,\ndescartar o código bruto no caminho da VM. Nenhum código muda nesta ADR.\nAtualiza docs/gc.md com o resumo da composição.\n\nAI usage: medições e texto com assistência de IA; números vindos de amostragem\nlocal reproduzível (fora do repo).\n\n* docs(memory): corrige composição do RSS após amostragem exata (ADR 029)\n\nGrandes alocações agora são contadas exatas. Retira a atribuição de 8 MB ao\níndice de classes (ruído de amostragem). Registra o APK inteiro lido em memória\n(12 MB) como primeiro candidato, e o resolve_cache (~5 MB) como segundo.\n\n* docs(memory): ação 1 da ADR 029 medida sem ganho (APK paginado)\n\nApk::open_paged no caminho da CLI: pico 172 168 KB contra 169 252 KB (v5),\nsaída idêntica. Estimativa de 12 MB refutada; mudança revertida.\n\n---------\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T00:33:33-03:00",
          "tree_id": "15959921fa68877f5e1f36261f7780acfde52085",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/2bad5248f01e9e8c07fc99b612a337cf13f2725c"
        },
        "date": 1793763246349,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3209,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3210,
            "range": "± 67",
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
          "id": "858b6357c08b6334228830a7145ee96b4df9c053",
          "message": "perf(vm): chave do resolve_cache com nomes internados (ADR 029) (#525)\n\nA chave era (String, String, String) e era alocada a cada resolve_method,\nmesmo no acerto do cache. Agora usa Arc<str> de um interner por Classpath:\na busca não aloca depois do primeiro uso do nome.\n\nMedido (debug, primeiro quadro F-Droid, MALLOC_ARENA_MAX=1): pico 169 216 KB\n-> 163 156 KB (-3,6%); captura de tela idêntica (39 082 bytes, mesmos pixels).\nrd-vm --lib: 188 passed.\n\nAI usage: mudança e medição com assistência de IA; números de VmHWM locais.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T01:05:09-03:00",
          "tree_id": "9fe76c4e4d1fb56b1ec4c8dce870ede4c9a9dc8f",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/858b6357c08b6334228830a7145ee96b4df9c053"
        },
        "date": 1793765127767,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3558,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3559,
            "range": "± 14",
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
          "id": "e464fe7e4f24a1c750b6932d6310fbe88ae1de0e",
          "message": "docs(memory): ADR 030 — custo marginal de sessões e caminho para o alvo (#526)\n\n* docs(memory): ADR 030 — custo marginal de sessões e caminho para o alvo; ADR 029 com resultados\n\nMedido no rd mcp (debug): 1 sessão 175 744 KB; 2 sessões 303 748 KB; custo marginal\n≈128 MB por sessão. Alvo de 6 sessões em 200 MB exige ≈33 MB marginal; compartilhamento\nde DEX/APK e layout não bastam sozinhos (estimativas registradas, não medidas).\n\nADR 029: resultado medido da chave do resolve_cache (−4,2 MB médio, duas rodadas\npareadas) e decisão de não descartar o código bruto (Lei 2).\n\nAI usage: medições e texto com assistência de IA; drive.py fora do repositório.\n\n* docs(memory): ADR 030 — build release bloqueado por OOM nesta sandbox\n\nrustc de rd-vm em release morre com ~1,4 GB de RSS (LTO thin/cgu=1, cgu=16 e\ncgu=256 testados). Número de release fica pendente de máquina com mais RAM.\n\n---------\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T01:25:05-03:00",
          "tree_id": "44fa8d50698e533c78d2ea0369d65a81c1085a16",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/e464fe7e4f24a1c750b6932d6310fbe88ae1de0e"
        },
        "date": 1793766334174,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1835,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1898,
            "range": "± 23",
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
          "id": "5ad87131fe558103c903613bc71b30d505ea543b",
          "message": "docs(memory): ADR 031 — DEX e APK compartilhados entre sessões (proposta) (#527)\n\nDesenho, riscos e critérios de aceite para a próxima etapa da ADR 030.\nFato verificado: Dex e Arc<Dex> são Send + Sync.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T01:34:56-03:00",
          "tree_id": "48d928d8b245a4ef0d4065ec92af247c2ce2728f",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/5ad87131fe558103c903613bc71b30d505ea543b"
        },
        "date": 1793766920652,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3492,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3381,
            "range": "± 23",
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
          "id": "bedbb986a3e3ce47b9eed5c28d6a5f2c15e592cd",
          "message": "perf(agent): APK e DEX compartilhados entre sessões do mesmo arquivo (ADR 031) (#528)\n\nSessões do rd mcp que instalam o mesmo arquivo reusam o Arc<Apk> e os\nArc<Dex> montados, em vez de ler e parsear de novo. Os DEX são guardados como\nWeak: o custo sai da memória quando o último engine para.\n\n- rd-vm: Classpath::dexes é Vec<Arc<Dex>>; novos Classpath::new_shared e\n  Engine::new_shared (new continua igual).\n- rd-agent: registro ApkKey -> Weak<SharedApk> (caminho canônico, tamanho, mtime).\n- Medição (debug, mesmo APK, install+launch+screenshot): 2 sessões 303,6 MB ->\n  256,9 MB; custo marginal da 2ª sessão 128 MB -> 80 MB. 1 sessão sem mudança.\n- Testes: reuso de Arc entre sessões, liberação ao fechar, arquivo alterado\n  não reusado, falha de montagem não cacheada.\n\nDisclosure de IA: código e texto escritos com apoio de IA; revisados, testados\ne medidos localmente.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T03:07:17-03:00",
          "tree_id": "44485a0084ab8650ac7d732492b7dc1a11643b5c",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/bedbb986a3e3ce47b9eed5c28d6a5f2c15e592cd"
        },
        "date": 1793772455499,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3292,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3192,
            "range": "± 20",
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
          "id": "8f6a22d2455fc3a58c52061ea674dd5bb339be12",
          "message": "docs(memory): ADR 030 — estado atualizado (passo 1 feito na ADR 031) (#529)\n\nRegistra o resultado do compartilhamento de DEX/APK (PR #528): custo marginal da\n2ª sessão 127,9 MB -> 80,9 MB (-36,7%). Passos 2 e 3 seguem pendentes.\n\nDisclosure de IA: texto redigido com apoio de IA; links e varredura de token\nverificados localmente com scripts/check_md_links.py.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T03:18:10-03:00",
          "tree_id": "af90de2f1c22cc0eb630b29ffffec461b8fe9a04",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/8f6a22d2455fc3a58c52061ea674dd5bb339be12"
        },
        "date": 1793773108744,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3445,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3392,
            "range": "± 18",
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
          "id": "0e5fe7c57ba299544bce4fdc507e2b0637c33299",
          "message": "docs(memory): ADR 032 — cache de código decodificado compartilhado (proposta) (#530)\n\nDesenho, riscos e critérios de aceite para a opção 2 da ADR 030. Medição\npendente: a economia é estimativa.\n\nDisclosure de IA: texto redigido com apoio de IA; links verificados localmente.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T03:32:30-03:00",
          "tree_id": "7eedd7d933de9225e3c3696a025b4011cc685517",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/0e5fe7c57ba299544bce4fdc507e2b0637c33299"
        },
        "date": 1793773977277,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3393,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3395,
            "range": "± 14",
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
          "id": "a38984194c745a82f9b2e18f14af00e399d1c770",
          "message": "perf(vm): cache de código decodificado compartilhado entre sessões (ADR 032) (#531)\n\nDecodeCache guarda code_item e class_data por (dex, offset) e é compartilhado\npelos Classpath de um mesmo conjunto de DEX (ADR 031). Cada Classpath mantém um\nL1 local; o lock só entra em miss. O cache vive por Weak junto com os DEX.\n\nMedição (debug, rd mcp, VmHWM, mesmo APK):\n- 2 sessões: 256 708 KB -> 238 432 KB (-7,1%)\n- custo marginal da 2ª sessão: 80,4 MB -> 61,0 MB (-24,1%)\n- 1 sessão: +1,1 MB (+0,6%, ruído)\n- screenshots idênticas à base\n\nTestes: rd-vm 946 (lib 189), rd-agent 112 lib + integração; clippy -D warnings\ne fmt limpos.\n\nDisclosure de IA: código e texto escritos com apoio de IA; revisados, testados\ne medidos localmente.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T05:11:14-03:00",
          "tree_id": "a61e10082630d2fa59d8bae2793530df0c65d495",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/a38984194c745a82f9b2e18f14af00e399d1c770"
        },
        "date": 1793779902093,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3143,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3109,
            "range": "± 90",
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
          "id": "65bedb619f42572ced4ccacc273bc4db21b77a29",
          "message": "docs(memory): ADR 033 — layout de campos de instância por classe (proposta) (#532)\n\nDesenho e critérios para a opção 3 da ADR 030. Economia estimada, não medida.\n\nDisclosure de IA: texto redigido com apoio de IA; links verificados localmente.\n\nCo-authored-by: Xisto <xisto@example.invalid>",
          "timestamp": "2026-11-04T05:11:17-03:00",
          "tree_id": "3af7b148ba86933a4a21d4a93baacc357381b71d",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/65bedb619f42572ced4ccacc273bc4db21b77a29"
        },
        "date": 1793779927300,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3287,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3283,
            "range": "± 21",
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
          "id": "d3fd06105529817d29ea9087be522f26c1246f8b",
          "message": "Merge pull request #533 from nadabgfccvd/fix/vm-monitores-getintent-deny\n\nrd-vm: monitores reais, const-class primitivo, getIntent null e cargo-deny no CI",
          "timestamp": "2026-11-04T13:41:21-03:00",
          "tree_id": "98c861b76c407c18da7ff15c85c60a1b2c7d89d4",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/d3fd06105529817d29ea9087be522f26c1246f8b"
        },
        "date": 1793810501463,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1780,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1764,
            "range": "± 33",
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
          "id": "133d042532c9d608fc045b0e9afccd8dd22e9adb",
          "message": "perf(vm): fix the app-override fast path regression; cut host-call overhead (#534)\n\n* perf(vm): fix the app-override fast path regression; cut host-call overhead\n\nCorrectness fix to the previous commit (103ae3f). Its DEX-first guard only\nchecked that the method reference and the receiver were DEX classes. A DEX\nsubclass of a platform collection (e.g. ArrayDeque) took the app override\nand lost its host state, so `launch` failed with NOT_FOUND on\nLjava/util/ArrayDeque;->poll(). The guard now also requires the receiver's\nwhole superclass chain to be DEX-defined and to end at Object\n(Classpath::is_dex_only_chain, memoized like is_subtype).\n\nAlso:\n- invoke: receiver class is an Arc<str> (Heap::class_arc_of) instead of a\n  String allocated on every invoke.\n- java/lang/String charAt/length (String or CharSequence mref) go straight\n  to the String implementation. length uses len() for ASCII instead of\n  re-encoding UTF-16. RD_STR_SHADOW=1 also runs the full handler chain and\n  panics on any difference (validation switch, off by default).\n- instance_intrinsic_dispatch is split into the handler chain and\n  instance_intrinsic_core (the final match). No behaviour change.\n- android_stub_surfaces::class_of returns &str (no String per gate).\n- Classpath::code, Engine::statics/intrinsic_by_class, Heap names/remembered\n  use FxHashMap/FxHashSet instead of SipHash.\n\nMeasured on this sandbox, same box, back-to-back, fresh `rd mcp` per run.\nBaseline = working tree before the DEX-first override change.\n- launch observe=off median: 709 -> 548 ms (n=5)\n- launch observe=on  median: 1101 -> 835 ms (n=5)\n- launch CPU time (user+sys) median: 734 -> 596 ms (n=5)\n- tap_text observe=on median: 1036 -> 701 ms (mcp_bench, 3 runs)\n- key observe=on median: 907 -> 604 ms (mcp_bench, 3 runs)\n\nCI fixes (the previous push failed fmt and clippy):\n- cargo fmt --all\n- clippy: needless_borrow in interp.rs (field_ref_shared bindings) and\n  android_stub_surfaces.rs (class_of now returns &str); no behaviour change\n\nVerification:\n- cargo test -p rd-vm --lib: 193 passed\n- cargo test -p rd-agent --lib: 112 passed\n- scripts/comprix-regression.py --cycles 2: success, 17 steps, no failures\n- RD_STR_SHADOW=1 over launch and tap runs: no mismatches, no failed rows\n- 12-step observe sequence: identical to the pre-change build except the\n  runtime_health timing line\n\n* perf(vm): fix the app-override fast path regression; cut host-call overhead\n\nCorrectness fix to the previous commit (103ae3f). Its DEX-first guard only\nchecked that the method reference and the receiver were DEX classes. A DEX\nsubclass of a platform collection (e.g. ArrayDeque) took the app override\nand lost its host state, so `launch` failed with NOT_FOUND on\nLjava/util/ArrayDeque;->poll(). The guard now also requires the receiver's\nwhole superclass chain to be DEX-defined and to end at Object\n(Classpath::is_dex_only_chain, memoized like is_subtype).\n\nAlso:\n- invoke: receiver class is an Arc<str> (Heap::class_arc_of) instead of a\n  String allocated on every invoke.\n- java/lang/String charAt/length (String or CharSequence mref) go straight\n  to the String implementation. length uses len() for ASCII instead of\n  re-encoding UTF-16. RD_STR_SHADOW=1 also runs the full handler chain and\n  panics on any difference (validation switch, off by default).\n- instance_intrinsic_dispatch is split into the handler chain and\n  instance_intrinsic_core (the final match). No behaviour change.\n- android_stub_surfaces::class_of returns &str (no String per gate).\n- Classpath::code, Engine::statics/intrinsic_by_class, Heap names/remembered\n  use FxHashMap/FxHashSet instead of SipHash.\n\nMeasured on this sandbox, same box, back-to-back, fresh `rd mcp` per run.\nBaseline = working tree before the DEX-first override change.\n- launch observe=off median: 709 -> 548 ms (n=5)\n- launch observe=on  median: 1101 -> 835 ms (n=5)\n- launch CPU time (user+sys) median: 734 -> 596 ms (n=5)\n- tap_text observe=on median: 1036 -> 701 ms (mcp_bench, 3 runs)\n- key observe=on median: 907 -> 604 ms (mcp_bench, 3 runs)\n\nCI fixes (the previous push failed fmt and clippy):\n- cargo fmt --all\n- clippy: needless_borrow in interp.rs (field_ref_shared bindings) and\n  android_stub_surfaces.rs (class_of now returns &str); no behaviour change\n\nVerification:\n- cargo test -p rd-vm --lib: 193 passed\n- cargo test -p rd-agent --lib: 112 passed\n- scripts/comprix-regression.py --cycles 2: success, 17 steps, no failures\n- RD_STR_SHADOW=1 over launch and tap runs: no mismatches, no failed rows\n- 12-step observe sequence: identical to the pre-change build except the\n  runtime_health timing line\n\n---------\n\nCo-authored-by: Arena Agent <agent@arena.ai>",
          "timestamp": "2026-11-04T19:49:27-03:00",
          "tree_id": "e7876ed6c1882510d9023b408917be12dbd4b013",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/133d042532c9d608fc045b0e9afccd8dd22e9adb"
        },
        "date": 1793832584297,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 1735,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 1785,
            "range": "± 104",
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
          "id": "4c54936eec2eb9bccc45fee71c5dbbe9416bb1ee",
          "message": "docs(adr): ADR 034 — caminho quente de invoke (proposta) (#535)\n\nProposta de desenho para a próxima fatia de desempenho: cache de decisão\nDEX-first por site de invoke, coverage com FxHash, custos fixos por chamada\n(gc_frames, gc_due, cp.code). Inclui medições da rodada atual, a convenção\nde medição com observe on/off e critérios de aceite pareados.\n\nDocs only. Implementação em PR separado após esta ADR ser mergeada.\n\nCo-authored-by: Arena Agent <agent@arena.ai>",
          "timestamp": "2026-11-04T19:49:33-03:00",
          "tree_id": "e6b9c4c19912d1cc56e47a9a90b65877e4900609",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/4c54936eec2eb9bccc45fee71c5dbbe9416bb1ee"
        },
        "date": 1793832612567,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3238,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3307,
            "range": "± 124",
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
          "id": "b32cfbe4787a3bede65cb075050bc810e6481328",
          "message": "perf(vm): ADR 034 — invoke-site cache, pooled frame buffers, FxHash coverage (#536)\n\n* perf(vm): ADR 034 — invoke-site cache, pooled frame buffers, FxHash coverage\n\n- Per-site memo of the DEX-first invoke decision, keyed by (dex, method,\n  opcode) and the receiver class (up to 4 receivers per site). The DEX is\n  immutable after the Classpath is built, so the memo is exact.\n- Register and argument buffers come from an Engine pool instead of one\n  allocation per frame; the invoke no longer copies its argument slice\n  (raw.to_vec) before the call.\n- GC root buffers are pooled as well; gc_due checks the cheap condition first.\n- coverage uses FxHashSet.\n\nMeasured on the same box, observe on, 6 alternating pairs (median):\nlaunch 640 -> 599 ms, tap_text 495 -> 471 ms, key 440 -> 399 ms.\nGolden regression (2 cycles, 17 steps) passes; screen parity identical\nexcept session_id and gc pause timing.\n\n* fix(vm): clippy needless_borrow on the frame register buffer\n\n* fix(vm): clippy ptr_arg on the frame register buffer\n\n---------\n\nCo-authored-by: Arena Agent <agent@arena.ai>",
          "timestamp": "2026-11-04T20:03:28-03:00",
          "tree_id": "d3b30f512fce3d4857db0ab5af2711584fabe180",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/b32cfbe4787a3bede65cb075050bc810e6481328"
        },
        "date": 1793833428007,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3334,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3338,
            "range": "± 31",
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
          "id": "4d5f6b4c06bcdf36b538c4be1c57d9c0d842b0a0",
          "message": "docs(adr): ADR 034 resultados e ADR 035 (identidades inteiras no caminho quente) (#537)\n\nCo-authored-by: Arena Agent <agent@arena.ai>",
          "timestamp": "2026-11-04T20:18:44-03:00",
          "tree_id": "0f2f65602af05e899e549c5f7ccb29c806c898e4",
          "url": "https://github.com/nadabgfccvd/RustyDroid/commit/4d5f6b4c06bcdf36b538c4be1c57d9c0d842b0a0"
        },
        "date": 1793834356371,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse_method_ids_20k",
            "value": 3287,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse_field_ids_20k",
            "value": 3284,
            "range": "± 7",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}