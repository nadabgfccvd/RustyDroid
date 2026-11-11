//! Benchmark de regressão (RustyDroid.md §3.2, `bench.yml`).
//!
//! Sem criterion: harness próprio que imprime linhas no formato libtest
//! (`test <nome> ... bench: <ns> ns/iter (+/- <desvio>)`), que é o formato
//! lido pelo `github-action-benchmark` (ferramenta `cargo`).
//!
//! Entrada sintética e determinística: tabelas `method_ids` / `field_ids`
//! de 20 000 entradas (8 bytes cada), com índices válidos.
//!
//! # Dois modos, e por quê (Issue #629, ADR 038)
//!
//! **Modo wall-clock (padrão).** Imprime ns/iter no formato libtest, que o
//! `github-action-benchmark` lê para desenhar o gráfico em `gh-pages`. É
//! **advisory**: o pool `ubuntu-latest` mistura classes de CPU e a série
//! histórica vai de 1 736 a 17 370 ns por iteração com o mesmo código. Medido
//! em três execuções consecutivas: o alvo variou 2,3% e a calibração que eu
//! tinha posto como denominador variou 62% — normalizar piorou a métrica.
//!
//! **Modo `--count <nome>`.** Executa **uma** iteração de um alvo e sai, para
//! rodar sob `valgrind --tool=callgrind`. Instruções retiradas são
//! determinísticas: três rebuilds do mesmo fonte dão exatamente o mesmo número
//! (delta de 45 552 instruções para `parse_method_ids_20k`, três vezes). Esse é
//! o modo que o gate bloqueante usa (`scripts/bench_ir.py`).

use std::hint::black_box;
use std::time::Instant;

const ENTRIES: u32 = 20_000;
const SAMPLES: usize = 30;

fn table(entries: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(entries as usize * 8);
    for i in 0..entries {
        buf.extend_from_slice(&((i % 512) as u16).to_le_bytes()); // class_idx
        buf.extend_from_slice(&((i % 64) as u16).to_le_bytes()); // proto/type_idx
        buf.extend_from_slice(&i.to_le_bytes()); // name_idx
    }
    buf
}

/// Mede N alvos **intercalados** no mesmo laço de amostragem.
///
/// Intercalar importa: cada amostra visita todos os alvos em sequência, então
/// deriva térmica, turbo e roubo de CPU incidem igualmente sobre eles. Medir um
/// alvo completo antes do outro deixaria um deles sempre na cauda mais
/// quente do processo, enviesando a razão que o gate usa.
fn measure_all<'a, const N: usize>(
    mut fns: [Box<dyn FnMut() -> usize + 'a>; N],
) -> [(f64, f64); N] {
    // aquecimento + escolha do número de iterações: ~20 ms por amostra, por alvo
    let mut iters = [0usize; N];
    for (i, f) in fns.iter_mut().enumerate() {
        for _ in 0..3 {
            black_box(f());
        }
        let t0 = Instant::now();
        black_box(f());
        let one = t0.elapsed().as_nanos().max(1) as f64;
        iters[i] = ((20_000_000.0 / one).ceil() as usize).clamp(1, 1_000_000);
    }

    let mut per_iter: [Vec<f64>; N] = std::array::from_fn(|_| Vec::with_capacity(SAMPLES));
    for _ in 0..SAMPLES {
        for (i, f) in fns.iter_mut().enumerate() {
            let t = Instant::now();
            for _ in 0..iters[i] {
                black_box(f());
            }
            per_iter[i].push(t.elapsed().as_nanos() as f64 / iters[i] as f64);
        }
    }

    std::array::from_fn(|i| {
        let v = &per_iter[i];
        let mean = v.iter().sum::<f64>() / SAMPLES as f64;
        let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / SAMPLES as f64;
        (mean, var.sqrt())
    })
}

fn report(name: &str, (mean, dev): (f64, f64)) {
    println!(
        "test {name} ... bench: {:>12} ns/iter (+/- {:.0})",
        mean.round() as u64,
        dev.round()
    );
}

fn main() {
    // `--count <name>`: exactly one iteration of one target, then exit. Meant to
    // be run under `valgrind --tool=callgrind`, which counts retired
    // instructions instead of wall-clock time. Instruction counts are
    // deterministic and machine-invariant, which is the only way a 10% gate can
    // mean anything on a shared runner pool (issue #629, ADR 038 amendment).
    let mut args = std::env::args().skip(1);
    if let Some(flag) = args.next() {
        if flag == "--count" {
            let name = args.next().unwrap_or_default();
            let data = table(ENTRIES);
            let n = match name.as_str() {
                // Linha de base do processo: `table()` já rodou, então o que
                // sobra é o custo fixo de startup + alocação do buffer. O gate
                // subtrai esta contagem para que o limiar de 10% incida sobre o
                // trabalho, não sobre o runtime.
                "noop" => {
                    black_box(&data);
                    0
                }
                "parse_method_ids_20k" => {
                    let r = rd_dex::methods::parse_method_ids(&data, ENTRIES, 0);
                    black_box(&r);
                    r.as_ref().map(|v| v.len()).unwrap_or(0)
                }
                "parse_field_ids_20k" => {
                    let r = rd_dex::fields::parse_field_ids(&data, ENTRIES, 0);
                    black_box(&r);
                    r.as_ref().map(|v| v.len()).unwrap_or(0)
                }
                other => {
                    eprintln!("unknown bench: {other}");
                    std::process::exit(2);
                }
            };
            // O valor é impresso para o otimizador não descartar o trabalho e
            // para o log mostrar que a iteração realmente aconteceu.
            println!("counted {name} (len={n})");
            return;
        }
    }

    let data = table(ENTRIES);
    // Os dois alvos são intercalados por amostra, então deriva térmica e turbo
    // incidem igualmente sobre eles.
    let [method, field] = measure_all([
        Box::new(|| {
            let r = rd_dex::methods::parse_method_ids(&data, ENTRIES, 0);
            let n = r.as_ref().map(|v| v.len()).unwrap_or(0);
            // mantém o resultado materializado: sem isto o otimizador apaga o decode
            black_box(&r);
            n
        }),
        Box::new(|| {
            let r = rd_dex::fields::parse_field_ids(&data, ENTRIES, 0);
            let n = r.as_ref().map(|v| v.len()).unwrap_or(0);
            // mantém o resultado materializado: sem isto o otimizador apaga o decode
            black_box(&r);
            n
        }),
    ]);

    report("parse_method_ids_20k", method);
    report("parse_field_ids_20k", field);
}
