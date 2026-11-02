window.BENCHMARK_DATA = {
  "lastUpdate": 1793577836961,
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
      }
    ]
  }
}