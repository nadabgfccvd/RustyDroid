//! Símbolos de libc/libm/pthread/semaphore/tempo/rede que as bibliotecas
//! nativas de apps reais importam e que o shim principal de `runtime.rs`
//! ainda não tinha (`mbtowc`, `strtol`, `strftime`, `sem_*`, ...).
//!
//! Regras deste módulo:
//! - cada função segue o comportamento do C/POSIX para o caso modelado;
//!   o que não é modelado (ex.: `sem_open`, `pipe`, `poll`, `fopen`) não
//!   aparece aqui e continua caindo em `NOT_IMPLEMENTED` (fail closed);
//! - locale é `C` (ASCII), como o padrão do processo de um app Android:
//!   bytes ≥ 0x80 em conversões multibyte dão `EILSEQ`;
//! - processo de thread única: lock sem contenção dá sucesso; `sem_wait`
//!   que bloquearia para sempre é erro fatal, não espera infinita;
//! - fuso horário é UTC (o fuso do device não é modelado);
//! - `getcwd` devolve `/`, cwd inicial de todo processo de app Android.
//!
//! A parte pura (parse, calendário, formatação) é testada sem worker.

use super::{ioerr, Abi, Return, Runtime};
use crate::{RdError, Result};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const EINVAL: u32 = 22;
const ERANGE: u32 = 34;
const EAGAIN: u32 = 11;
const EILSEQ: u32 = 84;
const EOVERFLOW: u32 = 75;
const EAFNOSUPPORT: u32 = 97;
const ETIMEDOUT: u32 = 110;
const SEM_VALUE_MAX: u32 = 0x3fff_ffff;
const MAX_USLEEP_US: u64 = 2_000_000;

/// Estado host do módulo (string de fuso UTC e semáforos por endereço).
#[derive(Debug, Default)]
pub(crate) struct LibcState {
    utc_zone: Option<u64>,
    self_handle: Option<u64>,
    sems: BTreeMap<u64, u32>,
}

/// Campos de `struct tm` (bionic/glibc), já no formato C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tm {
    pub sec: i32,
    pub min: i32,
    pub hour: i32,
    pub mday: i32,
    /// 0 = janeiro.
    pub mon: i32,
    /// Anos desde 1900.
    pub year: i32,
    /// 0 = domingo.
    pub wday: i32,
    /// 0 = 1º de janeiro.
    pub yday: i32,
}

fn is_c_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r')
}

/// Resultado de `strtol` sobre bytes (sem terminador).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LongParse {
    pub value: i64,
    /// Índice logo após o último caractere consumido (0 = nenhuma conversão).
    pub end: usize,
    pub overflow: bool,
}

/// `strtol` no locale C. `base` 0 (detecta `0x`/`0`) ou 2..=36.
pub(crate) fn parse_long(s: &[u8], base: i32) -> LongParse {
    let mut i = 0;
    while i < s.len() && is_c_space(s[i]) {
        i += 1;
    }
    let mut neg = false;
    if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
        neg = s[i] == b'-';
        i += 1;
    }
    let mut base = base as u32;
    let has_hex_prefix =
        |i: usize| s.len() >= i + 2 && s[i] == b'0' && (s[i + 1] == b'x' || s[i + 1] == b'X');
    if (base == 0 || base == 16) && has_hex_prefix(i) {
        // "0x" só é prefixo se vier dígito hex depois; senão converte o "0".
        if s.get(i + 2)
            .is_some_and(|c| (*c as char).is_ascii_hexdigit())
        {
            base = 16;
            i += 2;
        }
    }
    if base == 0 {
        base = if s.get(i) == Some(&b'0') { 8 } else { 10 };
    }
    let digit_start = i;
    let mut acc: u128 = 0;
    let mut overflow = false;
    let limit: u128 = if neg {
        i64::MIN.unsigned_abs() as u128
    } else {
        i64::MAX as u128
    };
    while i < s.len() {
        let Some(d) = (s[i] as char).to_digit(36) else {
            break;
        };
        if d >= base {
            break;
        }
        if !overflow {
            acc = acc * base as u128 + d as u128;
            if acc > limit {
                overflow = true;
            }
        }
        i += 1;
    }
    if i == digit_start {
        return LongParse {
            value: 0,
            end: 0,
            overflow: false,
        };
    }
    let value = if overflow {
        if neg {
            i64::MIN
        } else {
            i64::MAX
        }
    } else if neg {
        (-(acc as i128)) as i64
    } else {
        acc as i64
    };
    LongParse {
        value,
        end: i,
        overflow,
    }
}

/// Prefixo decimal de `strtod` no locale C: `[ws][+-]dígitos[.dígitos][e[+-]dígitos]`.
/// Devolve `(valor, bytes consumidos)`; sem conversão → `(0.0, 0)`.
pub(crate) fn parse_double_prefix(s: &[u8]) -> (f64, usize) {
    let mut i = 0;
    while i < s.len() && is_c_space(s[i]) {
        i += 1;
    }
    let start = i;
    if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
        i += 1;
    }
    let mut digits = 0;
    while i < s.len() && s[i].is_ascii_digit() {
        i += 1;
        digits += 1;
    }
    if i < s.len() && s[i] == b'.' {
        i += 1;
        while i < s.len() && s[i].is_ascii_digit() {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return (0.0, 0);
    }
    if i < s.len() && (s[i] == b'e' || s[i] == b'E') {
        let mut j = i + 1;
        if j < s.len() && (s[j] == b'+' || s[j] == b'-') {
            j += 1;
        }
        if j < s.len() && s[j].is_ascii_digit() {
            while j < s.len() && s[j].is_ascii_digit() {
                j += 1;
            }
            i = j;
        }
    }
    let text = std::str::from_utf8(&s[start..i]).unwrap_or("0");
    (text.parse::<f64>().unwrap_or(0.0), i)
}

/// Dias desde 1970-01-01 (algoritmo de calendário gregoriano proléptico).
pub(crate) fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `gmtime` em UTC. `None` se o ano não cabe em `int` (EOVERFLOW).
pub(crate) fn tm_from_epoch(t: i64) -> Option<Tm> {
    let days = t.div_euclid(86_400);
    let secs = t.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let year = y - 1900;
    if year < i32::MIN as i64 || year > i32::MAX as i64 {
        return None;
    }
    let yday = days - days_from_civil(y, 1, 1);
    Some(Tm {
        sec: (secs % 60) as i32,
        min: ((secs / 60) % 60) as i32,
        hour: (secs / 3600) as i32,
        mday: d as i32,
        mon: (m - 1) as i32,
        year: year as i32,
        wday: (days + 4).rem_euclid(7) as i32,
        yday: yday as i32,
    })
}

const WDAY_FULL: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTH_FULL: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// `strftime` no locale C para o subconjunto de conversões listado abaixo.
/// Conversão fora do subconjunto → `Err` (não é inventada).
pub(crate) fn format_tm(
    fmt: &[u8],
    tm: &Tm,
    gmtoff: i64,
    zone: &[u8],
) -> std::result::Result<Vec<u8>, String> {
    let mut out = String::new();
    let year = tm.year as i64 + 1900;
    let hour12 = if tm.hour % 12 == 0 { 12 } else { tm.hour % 12 };
    let wd = tm.wday.rem_euclid(7) as usize;
    let mo = tm.mon.rem_euclid(12) as usize;
    let mut i = 0;
    while i < fmt.len() {
        let c = fmt[i];
        if c != b'%' {
            out.push(c as char);
            i += 1;
            continue;
        }
        i += 1;
        let Some(&spec) = fmt.get(i) else {
            return Err("strftime: '%' no fim do formato".into());
        };
        i += 1;
        match spec {
            b'%' => out.push('%'),
            b'n' => out.push('\n'),
            b't' => out.push('\t'),
            b'Y' => out.push_str(&year.to_string()),
            b'C' => out.push_str(&format!("{:02}", year.div_euclid(100))),
            b'y' => out.push_str(&format!("{:02}", year.rem_euclid(100))),
            b'm' => out.push_str(&format!("{:02}", mo + 1)),
            b'd' => out.push_str(&format!("{:02}", tm.mday)),
            b'e' => out.push_str(&format!("{:2}", tm.mday)),
            b'H' => out.push_str(&format!("{:02}", tm.hour)),
            b'I' => out.push_str(&format!("{:02}", hour12)),
            b'M' => out.push_str(&format!("{:02}", tm.min)),
            b'S' => out.push_str(&format!("{:02}", tm.sec)),
            b'j' => out.push_str(&format!("{:03}", tm.yday + 1)),
            b'p' => out.push_str(if tm.hour < 12 { "AM" } else { "PM" }),
            b'a' => out.push_str(&WDAY_FULL[wd][..3]),
            b'A' => out.push_str(WDAY_FULL[wd]),
            b'b' | b'h' => out.push_str(&MONTH_FULL[mo][..3]),
            b'B' => out.push_str(MONTH_FULL[mo]),
            b'u' => out.push_str(&(if wd == 0 { 7 } else { wd }).to_string()),
            b'w' => out.push_str(&wd.to_string()),
            b'F' => out.push_str(&format!("{year}-{:02}-{:02}", mo + 1, tm.mday)),
            b'T' => out.push_str(&format!("{:02}:{:02}:{:02}", tm.hour, tm.min, tm.sec)),
            b'R' => out.push_str(&format!("{:02}:{:02}", tm.hour, tm.min)),
            b'D' => out.push_str(&format!(
                "{:02}/{:02}/{:02}",
                mo + 1,
                tm.mday,
                year.rem_euclid(100)
            )),
            b'z' => {
                let sign = if gmtoff < 0 { '-' } else { '+' };
                let mins = gmtoff.abs() / 60;
                out.push_str(&format!("{sign}{:02}{:02}", mins / 60, mins % 60));
            }
            b'Z' => out.push_str(&String::from_utf8_lossy(zone)),
            other => {
                return Err(format!(
                    "strftime: conversão %{} fora do subconjunto modelado",
                    other as char
                ))
            }
        }
    }
    Ok(out.into_bytes())
}

/// Caracteres ASCII de um `wchar_t` (4 bytes); `None` se não for ASCII.
fn wc_to_ascii(w: u32) -> Option<u8> {
    (w < 0x80).then_some(w as u8)
}

impl Runtime {
    /// Offsets de `tm_gmtoff` e `tm_zone` para a ABI (bionic LP64 e LP32).
    fn tm_tail_offsets(&self) -> (u64, u64) {
        // LP32: gmtoff (long, 4 bytes) em 36, zone em 40.
        // LP64: gmtoff (long, 8 bytes) em 40 (alinhado), zone em 48.
        let off_gmtoff = if self.abi == Abi::Arm { 36 } else { 40 };
        let off_zone = off_gmtoff + self.abi.pointer_size() as u64;
        (off_gmtoff, off_zone)
    }

    fn read_tm(&mut self, addr: u64) -> Result<Tm> {
        let b = self.read(addr, 36)?;
        let f = |i: usize| i32::from_le_bytes([b[i * 4], b[i * 4 + 1], b[i * 4 + 2], b[i * 4 + 3]]);
        Ok(Tm {
            sec: f(0),
            min: f(1),
            hour: f(2),
            mday: f(3),
            mon: f(4),
            year: f(5),
            wday: f(6),
            yday: f(7),
        })
    }

    fn write_tm(&mut self, addr: u64, tm: &Tm, gmtoff: i64, zone_ptr: u64) -> Result<()> {
        let fields = [
            tm.sec, tm.min, tm.hour, tm.mday, tm.mon, tm.year, tm.wday, tm.yday, 0,
        ];
        let mut bytes = Vec::with_capacity(36);
        for f in fields {
            bytes.extend_from_slice(&f.to_le_bytes());
        }
        self.write(addr, &bytes)?;
        let (off_g, off_z) = self.tm_tail_offsets();
        if self.abi == Abi::Arm {
            self.write(addr + off_g, &(gmtoff as i32).to_le_bytes())?;
        } else {
            self.write(addr + off_g, &gmtoff.to_le_bytes())?;
        }
        self.write_word(addr + off_z, zone_ptr)
    }

    fn utc_zone_ptr(&mut self) -> Result<u64> {
        if let Some(p) = self.libc.utc_zone {
            return Ok(p);
        }
        let p = self.alloc_bytes(b"UTC\0")?;
        self.libc.utc_zone = Some(p);
        Ok(p)
    }

    fn self_handle(&mut self) -> Result<u64> {
        if let Some(p) = self.libc.self_handle {
            return Ok(p);
        }
        let p = self.allocate(16)?;
        self.libc.self_handle = Some(p);
        Ok(p)
    }

    /// Símbolos novos do módulo. `None` = não é deste módulo (segue o shim).
    pub(super) fn libc_more(&mut self, name: &str, a: &[u64]) -> Result<Option<Return>> {
        let int_ret = |v: i64| {
            Some(Return {
                bits: v as u64,
                fp_bits: None,
            })
        };
        let r = match name {
            "mbtowc" => {
                // (wchar_t *pwc, const char *s, size_t n)
                if a[1] == 0 {
                    return Ok(int_ret(0));
                }
                if a[2] == 0 {
                    return Ok(int_ret(-1));
                }
                let b = self.read(a[1], 1)?[0];
                match b {
                    0 => {
                        if a[0] != 0 {
                            self.write(a[0], &0u32.to_le_bytes())?;
                        }
                        int_ret(0)
                    }
                    b if b < 0x80 => {
                        if a[0] != 0 {
                            self.write(a[0], &u32::from(b).to_le_bytes())?;
                        }
                        int_ret(1)
                    }
                    _ => {
                        self.set_errno(EILSEQ)?;
                        int_ret(-1)
                    }
                }
            }
            "mbstowcs" => {
                // (wchar_t *dst, const char *src, size_t n)
                let src = self.c_string(a[1])?;
                if src.iter().any(|b| *b >= 0x80) {
                    self.set_errno(EILSEQ)?;
                    return Ok(int_ret(-1));
                }
                let count = src.len() as u64;
                if a[0] == 0 {
                    return Ok(int_ret(count as i64));
                }
                {
                    let n = count.min(a[2]) as usize;
                    let mut out = Vec::with_capacity(n * 4 + 4);
                    for b in &src[..n] {
                        out.extend_from_slice(&u32::from(*b).to_le_bytes());
                    }
                    if (n as u64) < a[2] && n == src.len() {
                        out.extend_from_slice(&0u32.to_le_bytes());
                    }
                    self.write(a[0], &out)?;
                }
                int_ret(count.min(a[2]) as i64)
            }
            "wcstombs" => {
                // (char *dst, const wchar_t *src, size_t n)
                let mut chars = Vec::new();
                let mut i = 0u64;
                loop {
                    let w = u32::from_le_bytes(
                        self.read(a[1] + i * 4, 4)?
                            .try_into()
                            .map_err(|_| ioerr("wchar read"))?,
                    );
                    if w == 0 {
                        break;
                    }
                    let Some(c) = wc_to_ascii(w) else {
                        self.set_errno(EILSEQ)?;
                        return Ok(int_ret(-1));
                    };
                    chars.push(c);
                    i += 1;
                    if i > 1 << 20 {
                        return Err(RdError::new("NATIVE_MEMORY", "wcstombs sem terminador"));
                    }
                }
                let count = chars.len() as u64;
                if a[0] == 0 {
                    return Ok(int_ret(count as i64));
                }
                {
                    let n = count.min(a[2]) as usize;
                    let mut out = chars[..n].to_vec();
                    if (n as u64) < a[2] && n == chars.len() {
                        out.push(0);
                    }
                    self.write(a[0], &out)?;
                }
                int_ret(count.min(a[2]) as i64)
            }
            "iswspace" => int_ret(i64::from(matches!(a[0] as u32, 9..=13 | 32))),
            "strtol" => {
                // (const char *s, char **end, int base)
                let base = a[2] as u32 as i32;
                if base != 0 && !(2..=36).contains(&base) {
                    self.set_errno(EINVAL)?;
                    return Ok(int_ret(0));
                }
                let s = self.c_string(a[0])?;
                let p = parse_long(&s, base);
                if p.overflow {
                    self.set_errno(ERANGE)?;
                }
                if a[1] != 0 {
                    // Sem conversão, *end = nptr (C99 7.22.1.4).
                    let end = if p.end == 0 {
                        a[0]
                    } else {
                        a[0] + p.end as u64
                    };
                    self.write_word(a[1], end)?;
                }
                int_ret(p.value)
            }
            "atof" => {
                let s = self.c_string(a[0])?;
                let (v, _) = parse_double_prefix(&s);
                Some(Return {
                    bits: 0,
                    fp_bits: Some(v.to_bits()),
                })
            }
            "time" => {
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| ioerr("relógio anterior a 1970"))?
                    .as_secs() as i64;
                if a[0] != 0 {
                    self.write(a[0], &secs.to_le_bytes())?;
                }
                int_ret(secs)
            }
            "gmtime_r" | "localtime_r" => {
                // (const time_t *t, struct tm *result): ambos em UTC.
                let t = i64::from_le_bytes(
                    self.read(a[0], 8)?
                        .try_into()
                        .map_err(|_| ioerr("time_t"))?,
                );
                match tm_from_epoch(t) {
                    Some(tm) => {
                        let z = self.utc_zone_ptr()?;
                        self.write_tm(a[1], &tm, 0, z)?;
                        Some(Return {
                            bits: a[1],
                            fp_bits: None,
                        })
                    }
                    None => {
                        self.set_errno(EOVERFLOW)?;
                        Some(Return {
                            bits: 0,
                            fp_bits: None,
                        })
                    }
                }
            }
            "strftime" => {
                // (char *s, size_t max, const char *fmt, const struct tm *tm)
                let fmt = self.c_string(a[2])?;
                let tm = self.read_tm(a[3])?;
                let (off_g, off_z) = self.tm_tail_offsets();
                let gmtoff = if self.abi == Abi::Arm {
                    i32::from_le_bytes(
                        self.read(a[3] + off_g, 4)?
                            .try_into()
                            .map_err(|_| ioerr("gmtoff"))?,
                    ) as i64
                } else {
                    i64::from_le_bytes(
                        self.read(a[3] + off_g, 8)?
                            .try_into()
                            .map_err(|_| ioerr("gmtoff"))?,
                    )
                };
                let zone_ptr = self.read_word(a[3] + off_z)?;
                let zone = if zone_ptr == 0 {
                    Vec::new()
                } else {
                    self.c_string(zone_ptr)?
                };
                let text = format_tm(&fmt, &tm, gmtoff, &zone).map_err(|e| {
                    RdError::new("NOT_IMPLEMENTED", format!("native import strftime: {e}"))
                })?;
                if (text.len() as u64) < a[1] {
                    let mut out = text.clone();
                    out.push(0);
                    self.write(a[0], &out)?;
                    int_ret(text.len() as i64)
                } else {
                    int_ret(0)
                }
            }
            "usleep" => {
                let us = a[0] as u32 as u64;
                if us > MAX_USLEEP_US {
                    return Err(RdError::new(
                        "NATIVE_TIMEOUT",
                        format!("usleep({us}) acima do teto de {MAX_USLEEP_US} µs"),
                    ));
                }
                std::thread::sleep(Duration::from_micros(us));
                int_ret(0)
            }
            "getcwd" => {
                // (char *buf, size_t size): cwd de processo de app = "/".
                if a[1] < 2 {
                    self.set_errno(ERANGE)?;
                    return Ok(int_ret(0));
                }
                let dst = if a[0] == 0 { self.allocate(2)? } else { a[0] };
                self.write(dst, b"/\0")?;
                Some(Return {
                    bits: dst,
                    fp_bits: None,
                })
            }
            "inet_pton" => {
                // (int af, const char *src, void *dst)
                let src = self.c_string(a[1])?;
                let text = String::from_utf8_lossy(&src).to_string();
                match a[0] as u32 as i32 {
                    2 => match text.parse::<std::net::Ipv4Addr>() {
                        Ok(ip) => {
                            self.write(a[2], &ip.octets())?;
                            int_ret(1)
                        }
                        Err(_) => int_ret(0),
                    },
                    10 => match text.parse::<std::net::Ipv6Addr>() {
                        Ok(ip) => {
                            self.write(a[2], &ip.octets())?;
                            int_ret(1)
                        }
                        Err(_) => int_ret(0),
                    },
                    _ => {
                        self.set_errno(EAFNOSUPPORT)?;
                        int_ret(-1)
                    }
                }
            }
            "pthread_self" => Some(Return {
                bits: self.self_handle()?,
                fp_bits: None,
            }),
            "pthread_mutexattr_init"
            | "pthread_mutexattr_destroy"
            | "pthread_mutexattr_settype" => {
                // Tipo do mutex não é guardado: thread única, sem contenção.
                int_ret(0)
            }
            "pthread_rwlock_tryrdlock" | "pthread_rwlock_trywrlock" => int_ret(0),
            "sem_init" => {
                // (sem_t *sem, int pshared, unsigned value)
                let value = a[2] as u32;
                if value > SEM_VALUE_MAX {
                    self.set_errno(EINVAL)?;
                    return Ok(int_ret(-1));
                }
                self.libc.sems.insert(a[0], value);
                int_ret(0)
            }
            "sem_destroy" => {
                if self.libc.sems.remove(&a[0]).is_none() {
                    self.set_errno(EINVAL)?;
                    return Ok(int_ret(-1));
                }
                int_ret(0)
            }
            "sem_post" => match self.libc.sems.get(&a[0]).copied() {
                None => {
                    self.set_errno(EINVAL)?;
                    int_ret(-1)
                }
                Some(v) if v >= SEM_VALUE_MAX => {
                    self.set_errno(EOVERFLOW)?;
                    int_ret(-1)
                }
                Some(v) => {
                    self.libc.sems.insert(a[0], v + 1);
                    int_ret(0)
                }
            },
            "sem_wait" => match self.libc.sems.get(&a[0]).copied() {
                None => {
                    self.set_errno(EINVAL)?;
                    int_ret(-1)
                }
                Some(0) => {
                    // Bloquearia para sempre: não há outra thread para postar.
                    return Err(RdError::new(
                        "NATIVE_DEADLOCK",
                        "sem_wait com valor 0 em runtime de thread única",
                    ));
                }
                Some(v) => {
                    self.libc.sems.insert(a[0], v - 1);
                    int_ret(0)
                }
            },
            "sem_trywait" => match self.libc.sems.get(&a[0]).copied() {
                None => {
                    self.set_errno(EINVAL)?;
                    int_ret(-1)
                }
                Some(0) => {
                    self.set_errno(EAGAIN)?;
                    int_ret(-1)
                }
                Some(v) => {
                    self.libc.sems.insert(a[0], v - 1);
                    int_ret(0)
                }
            },
            "sem_timedwait" => match self.libc.sems.get(&a[0]).copied() {
                None => {
                    self.set_errno(EINVAL)?;
                    int_ret(-1)
                }
                Some(0) => {
                    // Ninguém posta: o prazo expira (ETIMEDOUT é o resultado real).
                    self.set_errno(ETIMEDOUT)?;
                    int_ret(-1)
                }
                Some(v) => {
                    self.libc.sems.insert(a[0], v - 1);
                    int_ret(0)
                }
            },
            "sem_getvalue" => match self.libc.sems.get(&a[0]).copied() {
                None => {
                    self.set_errno(EINVAL)?;
                    int_ret(-1)
                }
                Some(v) => {
                    if a[1] != 0 {
                        self.write(a[1], &(v as i32).to_le_bytes())?;
                    }
                    int_ret(0)
                }
            },
            "__read_chk" => {
                // (int fd, void *buf, size_t count, size_t bufsize)
                if a[2] > a[3] {
                    return Err(RdError::new("NATIVE_ABORT", "__read_chk: count > bufsize"));
                }
                // Sem tabela de fds: o mesmo resultado de `read` (EBADF).
                self.set_errno(9)?;
                int_ret(-1)
            }
            _ => None,
        };
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strtol_matches_c_for_common_inputs() {
        assert_eq!(
            parse_long(b"  -42xyz", 10),
            LongParse {
                value: -42,
                end: 5,
                overflow: false
            }
        );
        assert_eq!(parse_long(b"0x1fz", 0).value, 0x1f);
        assert_eq!(parse_long(b"0x1fz", 0).end, 4);
        assert_eq!(parse_long(b"017", 0).value, 0o17);
        assert_eq!(
            parse_long(b"0xg", 0),
            LongParse {
                value: 0,
                end: 1,
                overflow: false
            }
        );
        assert_eq!(
            parse_long(b"zz", 10),
            LongParse {
                value: 0,
                end: 0,
                overflow: false
            }
        );
        assert_eq!(parse_long(b"z", 36).value, 35);
    }

    #[test]
    fn strtol_saturates_on_overflow() {
        let hi = parse_long(b"99999999999999999999999", 10);
        assert!(hi.overflow);
        assert_eq!(hi.value, i64::MAX);
        let lo = parse_long(b"-99999999999999999999999", 10);
        assert!(lo.overflow);
        assert_eq!(lo.value, i64::MIN);
        assert_eq!(parse_long(b"-9223372036854775808", 10).value, i64::MIN);
        assert!(!parse_long(b"-9223372036854775808", 10).overflow);
    }

    #[test]
    fn strtod_prefix_consumes_longest_decimal() {
        assert_eq!(parse_double_prefix(b"  3.25e2abc"), (325.0, 8));
        assert_eq!(parse_double_prefix(b"-.5"), (-0.5, 3));
        assert_eq!(parse_double_prefix(b"1e"), (1.0, 1));
        assert_eq!(parse_double_prefix(b"abc"), (0.0, 0));
    }

    #[test]
    fn civil_calendar_known_dates() {
        // 2000-03-01 00:00:00 UTC = 951868800
        let t = tm_from_epoch(951_868_800).unwrap();
        assert_eq!((t.year + 1900, t.mon + 1, t.mday), (2000, 3, 1));
        assert_eq!(t.wday, 3, "2000-03-01 foi quarta-feira");
        assert_eq!(
            t.yday, 60,
            "ano bissexto: 31 + 29 dias antes de março (0-based)"
        );
        // 1970-01-01 foi quinta-feira.
        let e = tm_from_epoch(0).unwrap();
        assert_eq!((e.year, e.mon, e.mday, e.wday, e.yday), (70, 0, 1, 4, 0));
        // Negativo: 1969-12-31 23:59:59.
        let n = tm_from_epoch(-1).unwrap();
        assert_eq!(
            (n.year, n.mon, n.mday, n.hour, n.min, n.sec),
            (69, 11, 31, 23, 59, 59)
        );
        // 2026-11-10 00:00:00 UTC.
        let r = tm_from_epoch(days_from_civil(2026, 11, 10) * 86_400).unwrap();
        assert_eq!(
            (r.year + 1900, r.mon + 1, r.mday, r.hour),
            (2026, 11, 10, 0)
        );
    }

    #[test]
    fn strftime_subset_c_locale() {
        let tm = tm_from_epoch(951_868_800 + 13 * 3600 + 5 * 60 + 9).unwrap();
        let out = format_tm(b"%Y-%m-%d %H:%M:%S %a %b %j %I%p %%", &tm, 0, b"UTC").unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "2000-03-01 13:05:09 Wed Mar 061 01PM %"
        );
        assert_eq!(
            String::from_utf8(format_tm(b"%z %Z", &tm, -3 * 3600, b"BRT").unwrap()).unwrap(),
            "-0300 BRT"
        );
        assert!(
            format_tm(b"%c", &tm, 0, b"UTC").is_err(),
            "fora do subconjunto: erro, não texto inventado"
        );
    }

    #[test]
    fn ascii_wide_conversions_are_locale_c() {
        assert_eq!(wc_to_ascii(0x41), Some(b'A'));
        assert_eq!(wc_to_ascii(0xe9), None);
    }

    fn rt() -> Runtime {
        super::super::p0_242_255_tests::mock_runtime(Abi::X86_64).expect("mock runtime")
    }

    fn call(rt: &mut Runtime, name: &str, args: &[u64]) -> Return {
        rt.libc_more(name, args)
            .expect("libc_more ok")
            .expect("símbolo do módulo")
    }

    fn cstr(rt: &mut Runtime, s: &[u8]) -> u64 {
        let mut b = s.to_vec();
        b.push(0);
        rt.alloc_bytes(&b).expect("alloc")
    }

    #[test]
    fn mbtowc_ascii_and_eilseq() {
        let mut rt = rt();
        let s = cstr(&mut rt, b"A\xe9");
        let pwc = rt.allocate(4).unwrap();
        assert_eq!(call(&mut rt, "mbtowc", &[pwc, s, 4]).bits as i64, 1);
        assert_eq!(rt.read(pwc, 4).unwrap(), 65u32.to_le_bytes().to_vec());
        assert_eq!(call(&mut rt, "mbtowc", &[pwc, s + 1, 4]).bits as i64, -1);
    }

    #[test]
    fn strtol_sets_endptr_and_range() {
        let mut rt = rt();
        let s = cstr(&mut rt, b"  123abc");
        let end = rt.allocate(8).unwrap();
        let r = call(&mut rt, "strtol", &[s, end, 10]);
        assert_eq!(r.bits as i64, 123);
        assert_eq!(rt.read_word(end).unwrap(), s + 5);
        // Sem conversão: *endptr = nptr.
        let n = cstr(&mut rt, b"xyz");
        call(&mut rt, "strtol", &[n, end, 10]);
        assert_eq!(rt.read_word(end).unwrap(), n);
        // Base inválida: EINVAL, valor 0.
        assert_eq!(call(&mut rt, "strtol", &[s, 0, 1]).bits, 0);
    }

    #[test]
    fn gmtime_then_strftime_round_trip() {
        let mut rt = rt();
        let t = rt.alloc_bytes(&951_868_800i64.to_le_bytes()).unwrap();
        let tm = rt.allocate(64).unwrap();
        let r = call(&mut rt, "gmtime_r", &[t, tm]);
        assert_eq!(r.bits, tm);
        let fields = rt.read_tm(tm).unwrap();
        assert_eq!(
            (
                fields.year,
                fields.mon,
                fields.mday,
                fields.wday,
                fields.yday
            ),
            (100, 2, 1, 3, 60)
        );
        let fmt = cstr(&mut rt, b"%Y-%m-%d %Z");
        let out = rt.allocate(64).unwrap();
        let n = call(&mut rt, "strftime", &[out, 64, fmt, tm]).bits;
        assert_eq!(n, 14);
        assert_eq!(rt.c_string(out).unwrap(), b"2000-03-01 UTC".to_vec());
        // Buffer pequeno: 0 (C), sem escrever.
        assert_eq!(call(&mut rt, "strftime", &[out, 4, fmt, tm]).bits, 0);
    }

    #[test]
    fn semaphores_follow_posix_counts() {
        let mut rt = rt();
        let sem = rt.allocate(32).unwrap();
        assert_eq!(call(&mut rt, "sem_init", &[sem, 0, 1]).bits, 0);
        assert_eq!(call(&mut rt, "sem_wait", &[sem]).bits, 0);
        assert_eq!(call(&mut rt, "sem_trywait", &[sem]).bits as i64, -1);
        assert_eq!(call(&mut rt, "sem_timedwait", &[sem, 0]).bits as i64, -1);
        call(&mut rt, "sem_post", &[sem]);
        let out = rt.allocate(4).unwrap();
        call(&mut rt, "sem_getvalue", &[sem, out]);
        assert_eq!(rt.read(out, 4).unwrap(), 1i32.to_le_bytes().to_vec());
        // sem_wait que bloquearia: erro fatal, não espera.
        call(&mut rt, "sem_wait", &[sem]);
        assert!(rt.libc_more("sem_wait", &[sem, 0]).is_err());
        assert_eq!(call(&mut rt, "sem_destroy", &[sem]).bits, 0);
        assert_eq!(
            call(&mut rt, "sem_post", &[sem]).bits as i64,
            -1,
            "destruído: EINVAL"
        );
    }

    #[test]
    fn inet_pton_v4_and_getcwd() {
        let mut rt = rt();
        let src = cstr(&mut rt, b"10.1.2.3");
        let dst = rt.allocate(16).unwrap();
        assert_eq!(call(&mut rt, "inet_pton", &[2, src, dst]).bits, 1);
        assert_eq!(rt.read(dst, 4).unwrap(), vec![10, 1, 2, 3]);
        let bad = cstr(&mut rt, b"10.1.2");
        assert_eq!(call(&mut rt, "inet_pton", &[2, bad, dst]).bits, 0);
        let buf = rt.allocate(8).unwrap();
        assert_eq!(call(&mut rt, "getcwd", &[buf, 8]).bits, buf);
        assert_eq!(rt.c_string(buf).unwrap(), b"/".to_vec());
    }
}
