//! Benchmark de regressão (RustyDroid.md §3.2, `bench.yml`).
//!
//! Sem criterion: harness próprio que imprime linhas no formato libtest
//! (`test <nome> ... bench: <ns> ns/iter (+/- <desvio>)`), que é o formato
//! lido pelo `github-action-benchmark` (ferramenta `cargo`).
//!
//! Entrada sintética e determinística: tabelas `method_ids` / `field_ids`
//! de 20 000 entradas (8 bytes cada), com índices válidos.
//!
//! # Calibração (Issue #629)
//!
//! O pool `ubuntu-latest` mistura classes de CPU: o mesmo código rende de
//! ~1,7 µs a ~16,8 µs por iteração conforme a máquina sorteada (a série
//! histórica de `dev/bench/data.js` é bimodal, com um vazio entre 1,25× e 2×
//! e um aglomerado estável em 5,1× da mediana). Comparar ns absolutos entre
//! execuções mede o parque de máquinas, não o código.
//!
//! Por isso o bench publica também `calib_scan_20k`: uma varredura fixa sobre
//! o mesmo buffer, com o mesmo padrão de acesso (leitura sequencial de 8 B por
//! entrada). Ela nunca muda de semântica e serve de denominador. O gate
//! (`scripts/bench_gate.py`) compara a razão `alvo / calibração`, que é
//! invariante à máquina — medido: 8,3× de variação no tempo absoluto
//! (71 µs → 593 µs sob contenção) mantém a razão dentro de 1,83–2,14.

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

/// Varredura de calibração: uma passada sequencial de 8 B por entrada sobre o
/// mesmo buffer, sem alocação. O custo acompanha a máquina (largura de banda de
/// memória, IPC, escala de frequência) como o decode acompanha, então a razão
/// entre os dois é estável entre runners.
///
/// **Não altere a semântica desta função**: ela é o denominador do gate e está
/// versionada junto com `scripts/bench_gate.py` justamente para não mudar.
#[inline(never)]
fn calib_scan(data: &[u8]) -> u64 {
    let mut acc = 0u64;
    let mut o = 0usize;
    while o + 8 <= data.len() {
        let v = u64::from_le_bytes([
            data[o],
            data[o + 1],
            data[o + 2],
            data[o + 3],
            data[o + 4],
            data[o + 5],
            data[o + 6],
            data[o + 7],
        ]);
        acc = acc.rotate_left(7) ^ v;
        o += 8;
    }
    acc
}

/// Mede N alvos **intercalados** no mesmo laço de amostragem.
///
/// Intercalar importa: cada amostra visita todos os alvos em sequência, então
/// deriva térmica, turbo e roubo de CPU incidem igualmente sobre eles. Medir um
/// alvo completo antes do outro deixaria a calibração sempre na cauda mais
/// quente do processo, enviesando a razão que o gate usa.
fn measure_all<'a, const N: usize>(
    mut fns: [Box<dyn FnMut() -> usize + 'a>; N],
) -> [(f64, f64); N] {
    // aquecimento + calibração de iterações: ~20 ms por amostra, por alvo
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
    let data = table(ENTRIES);
    // Ordem: calibração primeiro no array, mas os três são intercalados por
    // amostra, então a posição só define o rótulo de saída.
    let [calib, method, field] = measure_all([
        Box::new(|| black_box(calib_scan(&data)) as usize) as Box<dyn FnMut() -> usize + '_>,
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

    report("calib_scan_20k", calib);
    report("parse_method_ids_20k", method);
    report("parse_field_ids_20k", field);
}
