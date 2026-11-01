window.BENCHMARK_DATA = {
  "lastUpdate": 1793548642985,
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
      }
    ]
  }
}