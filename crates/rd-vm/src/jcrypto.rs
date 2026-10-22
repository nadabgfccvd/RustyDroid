//! `javax.crypto` (Cipher/KeyGenerator/Mac/spec classes) and
//! `java.security.MessageDigest`, with pure-Rust AES + SHA-1/SHA-2.
//!
//! Everything is deterministic (Law 4): generated keys derive from a
//! counter keyed hash, never from OS entropy. Supported transformations:
//! AES/ECB, AES/CBC, AES/CTR, AES/GCM with NoPadding or PKCS5Padding;
//! anything else raises the real `NoSuchAlgorithmException` /
//! `NoSuchPaddingException` so probing code (e.g. Tink's capability
//! checks) behaves exactly as on a device lacking the algorithm.
//! Cipher streaming is buffered: `update` accumulates and `doFinal`
//! processes the whole input (inputs here are keys/prefs, not streams).

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::{ElemKind, ObjRef};
use crate::value::Value;

pub(crate) const CIPHER: &str = "Ljavax/crypto/Cipher;";
pub(crate) const MAC: &str = "Ljavax/crypto/Mac;";
pub(crate) const KEY_GENERATOR: &str = "Ljavax/crypto/KeyGenerator;";
pub(crate) const SECRET_KEY_SPEC: &str = "Ljavax/crypto/spec/SecretKeySpec;";
pub(crate) const MESSAGE_DIGEST: &str = "Ljava/security/MessageDigest;";

fn put(vm: &mut Engine, r: ObjRef, f: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, v))
}

fn geti(vm: &Engine, r: ObjRef, f: &str, d: i32) -> i32 {
    match vm.heap.get_field(r, f) {
        Ok(Value::Int(i)) => i,
        _ => d,
    }
}

fn gets(vm: &Engine, r: ObjRef, f: &str) -> String {
    match vm.heap.get_field(r, f) {
        Ok(Value::Obj(o)) => vm.heap.as_str(o).unwrap_or_default().to_string(),
        _ => String::new(),
    }
}

fn exc(c: &str, m: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new(c, m.into()))
}

fn bytes_of(vm: &Engine, v: &Value) -> Vec<u8> {
    match v {
        Value::Obj(a) => vm
            .heap
            .as_array_elems(*a)
            .map(|(e, _)| {
                e.iter()
                    .map(|x| x.as_int().map(|i| i as u8).unwrap_or(0))
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn byte_array(vm: &mut Engine, data: &[u8]) -> Result<Value, VmExit> {
    let arr = vm
        .heap
        .alloc_array_zeroed(ElemKind::Int, data.len(), "[B")
        .map_err(crate::err::oom_throwable)?;
    if let Ok(elems) = vm.heap.array_elems_mut(arr) {
        for (slot, b) in elems.iter_mut().zip(data.iter()) {
            *slot = Value::Int(*b as i8 as i32);
        }
    }
    Ok(Value::Obj(arr))
}

fn buf_get(vm: &Engine, r: ObjRef) -> Vec<u8> {
    match vm.heap.get_field(r, "jc_buf") {
        Ok(v) => bytes_of(vm, &v),
        _ => Vec::new(),
    }
}

fn buf_set(vm: &mut Engine, r: ObjRef, data: &[u8]) -> Result<(), VmExit> {
    let a = byte_array(vm, data)?;
    put(vm, r, "jc_buf", a)
}

// ---------------------------------------------------------------- AES

fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut p = 0u8;
    for _ in 0..8 {
        if b & 1 != 0 {
            p ^= a;
        }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    p
}

fn sbox() -> [u8; 256] {
    let mut s = [0u8; 256];
    for (i, slot) in s.iter_mut().enumerate() {
        let x = i as u8;
        // multiplicative inverse in GF(2^8)
        let inv = if x == 0 {
            0
        } else {
            (1..256u16).find(|&k| gf_mul(x, k as u8) == 1).unwrap_or(0) as u8
        };
        *slot = inv
            ^ inv.rotate_left(1)
            ^ inv.rotate_left(2)
            ^ inv.rotate_left(3)
            ^ inv.rotate_left(4)
            ^ 0x63;
    }
    s
}

fn inv_sbox(s: &[u8; 256]) -> [u8; 256] {
    let mut inv = [0u8; 256];
    for (i, v) in s.iter().enumerate() {
        inv[*v as usize] = i as u8;
    }
    inv
}

const RCON: [u8; 10] = [1, 2, 4, 8, 16, 32, 64, 128, 27, 54];

fn expand_key(key: &[u8], s: &[u8; 256]) -> Vec<u8> {
    let nk = key.len() / 4;
    let nr = nk + 6;
    let mut w = key.to_vec();
    let mut i = nk;
    while w.len() < 16 * (nr + 1) {
        let mut t = [
            w[w.len() - 4],
            w[w.len() - 3],
            w[w.len() - 2],
            w[w.len() - 1],
        ];
        if i % nk == 0 {
            t = [
                s[t[1] as usize] ^ RCON[i / nk - 1],
                s[t[2] as usize],
                s[t[3] as usize],
                s[t[0] as usize],
            ];
        } else if nk > 6 && i % nk == 4 {
            t = [
                s[t[0] as usize],
                s[t[1] as usize],
                s[t[2] as usize],
                s[t[3] as usize],
            ];
        }
        for b in t {
            let prev = w[w.len() - 4 * nk];
            w.push(prev ^ b);
        }
        i += 1;
    }
    w
}

fn aes_block(key: &[u8], input: &[u8; 16], encrypt: bool) -> [u8; 16] {
    let s = sbox();
    let si = inv_sbox(&s);
    let w = expand_key(key, &s);
    let nr = key.len() / 4 + 6;
    let mut st = *input;
    let add_rk = |st: &mut [u8; 16], w: &[u8], round: usize| {
        for i in 0..16 {
            st[i] ^= w[round * 16 + i];
        }
    };
    let sub = |st: &mut [u8; 16], table: &[u8; 256]| {
        for b in st.iter_mut() {
            *b = table[*b as usize];
        }
    };
    let shift = |st: &mut [u8; 16], enc: bool| {
        let old = *st;
        for r in 0..4 {
            for c in 0..4 {
                let src = if enc { (c + r) % 4 } else { (c + 4 - r) % 4 };
                st[c * 4 + r] = old[src * 4 + r];
            }
        }
    };
    let mix = |st: &mut [u8; 16], enc: bool| {
        for c in 0..4 {
            let i = c * 4;
            let (a0, a1, a2, a3) = (st[i], st[i + 1], st[i + 2], st[i + 3]);
            if enc {
                st[i] = gf_mul(a0, 2) ^ gf_mul(a1, 3) ^ a2 ^ a3;
                st[i + 1] = a0 ^ gf_mul(a1, 2) ^ gf_mul(a2, 3) ^ a3;
                st[i + 2] = a0 ^ a1 ^ gf_mul(a2, 2) ^ gf_mul(a3, 3);
                st[i + 3] = gf_mul(a0, 3) ^ a1 ^ a2 ^ gf_mul(a3, 2);
            } else {
                st[i] = gf_mul(a0, 14) ^ gf_mul(a1, 11) ^ gf_mul(a2, 13) ^ gf_mul(a3, 9);
                st[i + 1] = gf_mul(a0, 9) ^ gf_mul(a1, 14) ^ gf_mul(a2, 11) ^ gf_mul(a3, 13);
                st[i + 2] = gf_mul(a0, 13) ^ gf_mul(a1, 9) ^ gf_mul(a2, 14) ^ gf_mul(a3, 11);
                st[i + 3] = gf_mul(a0, 11) ^ gf_mul(a1, 13) ^ gf_mul(a2, 9) ^ gf_mul(a3, 14);
            }
        }
    };
    if encrypt {
        add_rk(&mut st, &w, 0);
        for round in 1..nr {
            sub(&mut st, &s);
            shift(&mut st, true);
            mix(&mut st, true);
            add_rk(&mut st, &w, round);
        }
        sub(&mut st, &s);
        shift(&mut st, true);
        add_rk(&mut st, &w, nr);
    } else {
        add_rk(&mut st, &w, nr);
        for round in (1..nr).rev() {
            sub(&mut st, &si);
            shift(&mut st, false);
            add_rk(&mut st, &w, round);
            mix(&mut st, false);
        }
        sub(&mut st, &si);
        shift(&mut st, false);
        add_rk(&mut st, &w, 0);
    }
    st
}

fn xor16(a: &mut [u8], b: &[u8]) {
    for i in 0..16.min(b.len()) {
        a[i] ^= b[i];
    }
}

fn pkcs7_pad(data: &[u8]) -> Vec<u8> {
    let pad = 16 - (data.len() % 16);
    let mut out = data.to_vec();
    out.extend(std::iter::repeat(pad as u8).take(pad));
    out
}

fn pkcs7_unpad(data: &[u8]) -> Result<Vec<u8>, VmExit> {
    let bad = || exc("Ljavax/crypto/BadPaddingException;", "padding inválido");
    let Some(&last) = data.last() else {
        return Err(bad());
    };
    if last == 0 || last > 16 || data.len() < last as usize {
        return Err(bad());
    }
    if data[data.len() - last as usize..]
        .iter()
        .any(|&b| b != last)
    {
        return Err(bad());
    }
    Ok(data[..data.len() - last as usize].to_vec())
}

fn gf128_mul(x: &[u8; 16], y: &[u8; 16]) -> [u8; 16] {
    let mut z = [0u8; 16];
    let mut v = *y;
    for &xb in x.iter() {
        for j in 0..8 {
            if (xb >> (7 - j)) & 1 == 1 {
                for (zk, vk) in z.iter_mut().zip(v.iter()) {
                    *zk ^= *vk;
                }
            }
            let lsb = v[15] & 1;
            for k in (1..16).rev() {
                v[k] = (v[k] >> 1) | ((v[k - 1] & 1) << 7);
            }
            v[0] >>= 1;
            if lsb == 1 {
                v[0] ^= 0xe1;
            }
        }
    }
    z
}

fn ghash(h: &[u8; 16], data: &[u8]) -> [u8; 16] {
    let mut y = [0u8; 16];
    for chunk in data.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        xor16(&mut y, &block);
        y = gf128_mul(&y, h);
    }
    y
}

fn inc32(mut ctr: [u8; 16]) -> [u8; 16] {
    let c = u32::from_be_bytes([ctr[12], ctr[13], ctr[14], ctr[15]]).wrapping_add(1);
    ctr[12..16].copy_from_slice(&c.to_be_bytes());
    ctr
}

fn gcm_j0(iv: &[u8], h: &[u8; 16]) -> [u8; 16] {
    if iv.len() == 12 {
        let mut j = [0u8; 16];
        j[..12].copy_from_slice(iv);
        j[15] = 1;
        j
    } else {
        let mut d = iv.to_vec();
        while d.len() % 16 != 0 {
            d.push(0);
        }
        let mut lens = [0u8; 16];
        lens[8..16].copy_from_slice(&(iv.len() as u64 * 8).to_be_bytes());
        d.extend_from_slice(&lens);
        ghash(h, &d)
    }
}

fn gcm_crypt(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    let zero = [0u8; 16];
    let h = aes_block(key, &zero, true);
    let j0 = gcm_j0(iv, &h);
    let mut ctr = j0;
    let mut out = Vec::with_capacity(data.len());
    for chunk in data.chunks(16) {
        ctr = inc32(ctr);
        let ks = aes_block(key, &ctr, true);
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        xor16(&mut block, &ks);
        out.extend_from_slice(&block[..chunk.len()]);
    }
    out
}

fn gcm_tag(key: &[u8], iv: &[u8], aad: &[u8], cipher_text: &[u8]) -> [u8; 16] {
    let zero = [0u8; 16];
    let h = aes_block(key, &zero, true);
    let j0 = gcm_j0(iv, &h);
    let mut gh = Vec::new();
    gh.extend_from_slice(aad);
    while gh.len() % 16 != 0 {
        gh.push(0);
    }
    gh.extend_from_slice(cipher_text);
    while gh.len() % 16 != 0 {
        gh.push(0);
    }
    let mut lens = [0u8; 16];
    lens[0..8].copy_from_slice(&(aad.len() as u64 * 8).to_be_bytes());
    lens[8..16].copy_from_slice(&(cipher_text.len() as u64 * 8).to_be_bytes());
    gh.extend_from_slice(&lens);
    let mut s = ghash(&h, &gh);
    let mask = aes_block(key, &j0, true);
    xor16(&mut s, &mask);
    s
}

// ---------------------------------------------------------------- SHA

pub(crate) fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let mut msg = data.to_vec();
    let ml = data.len() as u64 * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&ml.to_be_bytes());
    for block in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let tmp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = tmp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for (i, v) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

const K32: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const H256: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const H224: [u32; 8] = [
    0xc1059ed8, 0x367cd507, 0x3070dd17, 0xf70e5939, 0xffc00b31, 0x68581511, 0x64f98fa7, 0xbefa4fa4,
];

pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    sha256_core(data, H256)
}

fn sha256_core(data: &[u8], h_init: [u32; 8]) -> [u8; 32] {
    let mut h: [u32; 8] = h_init;
    let mut msg = data.to_vec();
    let ml = data.len() as u64 * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&ml.to_be_bytes());
    for block in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K32[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

const K64: [u64; 80] = [
    0x428a2f98d728ae22,
    0x7137449123ef65cd,
    0xb5c0fbcfec4d3b2f,
    0xe9b5dba58189dbbc,
    0x3956c25bf348b538,
    0x59f111f1b605d019,
    0x923f82a4af194f9b,
    0xab1c5ed5da6d8118,
    0xd807aa98a3030242,
    0x12835b0145706fbe,
    0x243185be4ee4b28c,
    0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f,
    0x80deb1fe3b1696b1,
    0x9bdc06a725c71235,
    0xc19bf174cf692694,
    0xe49b69c19ef14ad2,
    0xefbe4786384f25e3,
    0x0fc19dc68b8cd5b5,
    0x240ca1cc77ac9c65,
    0x2de92c6f592b0275,
    0x4a7484aa6ea6e483,
    0x5cb0a9dcbd41fbd4,
    0x76f988da831153b5,
    0x983e5152ee66dfab,
    0xa831c66d2db43210,
    0xb00327c898fb213f,
    0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2,
    0xd5a79147930aa725,
    0x06ca6351e003826f,
    0x142929670a0e6e70,
    0x27b70a8546d22ffc,
    0x2e1b21385c26c926,
    0x4d2c6dfc5ac42aed,
    0x53380d139d95b3df,
    0x650a73548baf63de,
    0x766a0abb3c77b2a8,
    0x81c2c92e47edaee6,
    0x92722c851482353b,
    0xa2bfe8a14cf10364,
    0xa81a664bbc423001,
    0xc24b8b70d0f89791,
    0xc76c51a30654be30,
    0xd192e819d6ef5218,
    0xd69906245565a910,
    0xf40e35855771202a,
    0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8,
    0x1e376c085141ab53,
    0x2748774cdf8eeb99,
    0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63,
    0x4ed8aa4ae3418acb,
    0x5b9cca4f7763e373,
    0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc,
    0x78a5636f43172f60,
    0x84c87814a1f0ab72,
    0x8cc702081a6439ec,
    0x90befffa23631e28,
    0xa4506cebde82bde9,
    0xbef9a3f7b2c67915,
    0xc67178f2e372532b,
    0xca273eceea26619c,
    0xd186b8c721c0c207,
    0xeada7dd6cde0eb1e,
    0xf57d4f7fee6ed178,
    0x06f067aa72176fba,
    0x0a637dc5a2c898a6,
    0x113f9804bef90dae,
    0x1b710b35131c471b,
    0x28db77f523047d84,
    0x32caab7b40c72493,
    0x3c9ebe0a15c9bebc,
    0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6,
    0x597f299cfc657e2a,
    0x5fcb6fab3ad6faec,
    0x6c44198c4a475817,
];

fn sha512_core(data: &[u8], h0: [u64; 8]) -> [u8; 64] {
    let mut h = h0;
    let mut msg = data.to_vec();
    let ml = data.len() as u128 * 8;
    msg.push(0x80);
    while msg.len() % 128 != 112 {
        msg.push(0);
    }
    msg.extend_from_slice(&ml.to_be_bytes());
    for block in msg.chunks_exact(128) {
        let mut w = [0u64; 80];
        for i in 0..16 {
            w[i] = u64::from_be_bytes(block[8 * i..8 * i + 8].try_into().unwrap());
        }
        for i in 16..80 {
            let s0 = w[i - 15].rotate_right(1) ^ w[i - 15].rotate_right(8) ^ (w[i - 15] >> 7);
            let s1 = w[i - 2].rotate_right(19) ^ w[i - 2].rotate_right(61) ^ (w[i - 2] >> 6);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..80 {
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K64[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 64];
    for (i, v) in h.iter().enumerate() {
        out[8 * i..8 * i + 8].copy_from_slice(&v.to_be_bytes());
    }
    out
}

const H512: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

const H384: [u64; 8] = [
    0xcbbb9d5dc1059ed8,
    0x629a292a367cd507,
    0x9159015a3070dd17,
    0x152fecd8f70e5939,
    0x67332667ffc00b31,
    0x8eb44a8768581511,
    0xdb0c2e0d64f98fa7,
    0x47b5481dbefa4fa4,
];

pub(crate) fn sha512(data: &[u8]) -> [u8; 64] {
    sha512_core(data, H512)
}

pub(crate) fn digest_for(alg: &str, data: &[u8]) -> Option<Vec<u8>> {
    let a = alg.replace('-', "").to_ascii_uppercase();
    Some(match a.as_str() {
        "MD5" => crate::juuid::md5_pub(data).to_vec(),
        "SHA1" => sha1(data).to_vec(),
        "SHA224" => sha256_core(data, H224)[..28].to_vec(),
        "SHA256" => sha256(data).to_vec(),
        "SHA384" => sha512_core(data, H384)[..48].to_vec(),
        "SHA512" => sha512(data).to_vec(),
        _ => return None,
    })
}

fn digest_len(alg: &str) -> i32 {
    let a = alg.replace('-', "").to_ascii_uppercase();
    match a.as_str() {
        "MD5" => 16,
        "SHA1" => 20,
        "SHA224" => 28,
        "SHA256" => 32,
        "SHA384" => 48,
        "SHA512" => 64,
        _ => 0,
    }
}

fn hmac(alg: &str, key: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    let block = match alg.replace('-', "").to_ascii_uppercase().as_str() {
        "HMACSHA1" => 64usize,
        "HMACSHA224" | "HMACSHA256" => 64,
        "HMACSHA384" | "HMACSHA512" => 128,
        _ => return None,
    };
    let hash_alg = alg
        .trim_start_matches("Hmac")
        .trim_start_matches("HMAC")
        .to_string();
    let mut k = if key.len() > block {
        digest_for(&hash_alg, key)?
    } else {
        key.to_vec()
    };
    k.resize(block, 0);
    let inner: Vec<u8> = k
        .iter()
        .map(|b| b ^ 0x36)
        .chain(data.iter().copied())
        .collect();
    let inner_hash = digest_for(&hash_alg, &inner)?;
    let outer: Vec<u8> = k
        .iter()
        .map(|b| b ^ 0x5c)
        .chain(inner_hash.iter().copied())
        .collect();
    digest_for(&hash_alg, &outer)
}

// ---------------------------------------------------------------- VM

struct Transform {
    mode: String,
    padding_pkcs5: bool,
}

fn parse_transform(t: &str) -> Result<Transform, VmExit> {
    let parts: Vec<&str> = t.split('/').collect();
    if parts.len() > 3 || !parts[0].eq_ignore_ascii_case("AES") {
        return Err(exc(
            "Ljava/security/NoSuchAlgorithmException;",
            format!("Algorithm {t} not available"),
        ));
    }
    let mode = if parts.len() >= 2 {
        parts[1].to_ascii_uppercase()
    } else {
        "ECB".to_string()
    };
    let padding = if parts.len() == 3 {
        parts[2].to_ascii_uppercase()
    } else {
        "PKCS5PADDING".to_string()
    };
    if !matches!(mode.as_str(), "ECB" | "CBC" | "CTR" | "GCM") {
        return Err(exc(
            "Ljava/security/NoSuchAlgorithmException;",
            format!("Algorithm {t} not available"),
        ));
    }
    let padding_pkcs5 = match padding.as_str() {
        "NOPADDING" => false,
        "PKCS5PADDING" | "PKCS7PADDING" => true,
        _ => {
            return Err(exc(
                "Ljavax/crypto/NoSuchPaddingException;",
                format!("Padding {padding} not available"),
            ))
        }
    };
    Ok(Transform {
        mode,
        padding_pkcs5,
    })
}

fn buf_get_aad(vm: &Engine, r: ObjRef) -> Vec<u8> {
    match vm.heap.get_field(r, "jc_aad") {
        Ok(v) => bytes_of(vm, &v),
        _ => Vec::new(),
    }
}

fn buf_set_aad(vm: &mut Engine, r: ObjRef, data: &[u8]) -> Result<(), VmExit> {
    let a = byte_array(vm, data)?;
    put(vm, r, "jc_aad", a)
}

fn cipher_crypt(vm: &Engine, recv: ObjRef, input: &[u8]) -> Result<Vec<u8>, VmExit> {
    let alg = gets(vm, recv, "jc_alg");
    let tr = parse_transform(&alg)?;
    let key = match vm.heap.get_field(recv, "jc_key") {
        Ok(v) => bytes_of(vm, &v),
        _ => Vec::new(),
    };
    if !matches!(key.len(), 16 | 24 | 32) {
        return Err(exc(
            "Ljava/security/InvalidKeyException;",
            "chave AES inválida",
        ));
    }
    let encrypt = geti(vm, recv, "jc_opmode", 1) != 2;
    let iv = match vm.heap.get_field(recv, "jc_iv") {
        Ok(v) => bytes_of(vm, &v),
        _ => vec![0u8; 16],
    };
    let aad = buf_get_aad(vm, recv);
    match tr.mode.as_str() {
        "ECB" | "CBC" => {
            let data = if encrypt && tr.padding_pkcs5 {
                pkcs7_pad(input)
            } else {
                if input.len() % 16 != 0 {
                    return Err(exc(
                        "Ljavax/crypto/IllegalBlockSizeException;",
                        "comprimento não é múltiplo do bloco",
                    ));
                }
                input.to_vec()
            };
            let mut out = Vec::with_capacity(data.len());
            let mut prev = iv.clone();
            if encrypt {
                for chunk in data.chunks_exact(16) {
                    let mut block: [u8; 16] = chunk.try_into().unwrap();
                    if tr.mode == "CBC" {
                        xor16(&mut block, &prev);
                    }
                    let enc = aes_block(&key, &block, true);
                    if tr.mode == "CBC" {
                        prev = enc.to_vec();
                    }
                    out.extend_from_slice(&enc);
                }
                Ok(out)
            } else {
                for chunk in data.chunks_exact(16) {
                    let block: [u8; 16] = chunk.try_into().unwrap();
                    let plain0 = aes_block(&key, &block, false);
                    let mut plain = plain0;
                    if tr.mode == "CBC" {
                        xor16(&mut plain, &prev);
                        prev = chunk.to_vec();
                    }
                    out.extend_from_slice(&plain);
                }
                if tr.padding_pkcs5 {
                    pkcs7_unpad(&out)
                } else {
                    Ok(out)
                }
            }
        }
        "CTR" => {
            let mut out = Vec::with_capacity(input.len());
            let mut ctr: [u8; 16] = iv
                .as_slice()
                .try_into()
                .map_err(|_| exc("Ljava/security/InvalidKeyException;", "IV CTR inválido"))?;
            for chunk in input.chunks(16) {
                let ks = aes_block(&key, &ctr, true);
                let mut block = [0u8; 16];
                block[..chunk.len()].copy_from_slice(chunk);
                xor16(&mut block, &ks);
                out.extend_from_slice(&block[..chunk.len()]);
                ctr = inc32(ctr);
            }
            Ok(out)
        }
        "GCM" => {
            let tag_bits = geti(vm, recv, "jc_tagbits", 128);
            let tag_len = ((tag_bits / 8) as usize).min(16);
            if encrypt {
                let ct = gcm_crypt(&key, &iv, input);
                let tag = gcm_tag(&key, &iv, &aad, &ct);
                let mut out = ct;
                out.extend_from_slice(&tag[..tag_len]);
                Ok(out)
            } else {
                if input.len() < tag_len {
                    return Err(exc("Ljavax/crypto/AEADBadTagException;", "tag ausente"));
                }
                let (ct, given_tag) = input.split_at(input.len() - tag_len);
                let expect = gcm_tag(&key, &iv, &aad, ct);
                if expect[..tag_len] != *given_tag {
                    return Err(exc("Ljavax/crypto/AEADBadTagException;", "tag inválida"));
                }
                Ok(gcm_crypt(&key, &iv, ct))
            }
        }
        _ => Err(exc(
            "Ljava/security/NoSuchAlgorithmException;",
            "modo não suportado",
        )),
    }
}

fn key_bytes_of(vm: &Engine, key_obj: &Value) -> Vec<u8> {
    match key_obj {
        Value::Obj(o) => match vm.heap.get_field(*o, "jc_key") {
            Ok(v) => bytes_of(vm, &v),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

pub(crate) fn static_intrinsic(
    vm: &mut Engine,
    class: &str,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let _ = sig;
    let str0 = |vm: &Engine| -> String {
        match args.first() {
            Some(Value::Obj(o)) => vm.heap.as_str(*o).unwrap_or_default().to_string(),
            _ => String::new(),
        }
    };
    Ok(Some(match (class, name) {
        (CIPHER, "getInstance") => {
            let t = str0(vm);
            let _ = parse_transform(&t)?;
            let o = vm
                .heap
                .alloc_instance(CIPHER.to_string(), Vec::new())
                .map_err(crate::err::oom_throwable)?;
            let s = crate::intrinsics::alloc_string(vm, t)?;
            put(vm, o, "jc_alg", Value::Obj(s))?;
            put(vm, o, "jc_opmode", Value::Int(0))?;
            Value::Obj(o)
        }
        (MAC, "getInstance") => {
            let a = str0(vm);
            if hmac(&a, b"", b"").is_none() {
                return Err(exc(
                    "Ljava/security/NoSuchAlgorithmException;",
                    format!("Algorithm {a} not available"),
                ));
            }
            let o = vm
                .heap
                .alloc_instance(MAC.to_string(), Vec::new())
                .map_err(crate::err::oom_throwable)?;
            let s = crate::intrinsics::alloc_string(vm, a)?;
            put(vm, o, "jc_alg", Value::Obj(s))?;
            Value::Obj(o)
        }
        (KEY_GENERATOR, "getInstance") => {
            let a = str0(vm);
            if !a.eq_ignore_ascii_case("AES") {
                return Err(exc(
                    "Ljava/security/NoSuchAlgorithmException;",
                    format!("Algorithm {a} not available"),
                ));
            }
            let o = vm
                .heap
                .alloc_instance(KEY_GENERATOR.to_string(), Vec::new())
                .map_err(crate::err::oom_throwable)?;
            let s = crate::intrinsics::alloc_string(vm, a)?;
            put(vm, o, "jc_alg", Value::Obj(s))?;
            put(vm, o, "jc_keysize", Value::Int(128))?;
            Value::Obj(o)
        }
        (MESSAGE_DIGEST, "getInstance") => {
            let a = str0(vm);
            if digest_for(&a, b"").is_none() {
                return Err(exc(
                    "Ljava/security/NoSuchAlgorithmException;",
                    format!("Algorithm {a} not available"),
                ));
            }
            let o = vm
                .heap
                .alloc_instance(MESSAGE_DIGEST.to_string(), Vec::new())
                .map_err(crate::err::oom_throwable)?;
            let s = crate::intrinsics::alloc_string(vm, a)?;
            put(vm, o, "jc_alg", Value::Obj(s))?;
            Value::Obj(o)
        }
        (MESSAGE_DIGEST, "isEqual") => {
            let a = args.first().map(|v| bytes_of(vm, v)).unwrap_or_default();
            let b = args.get(1).map(|v| bytes_of(vm, v)).unwrap_or_default();
            Value::Int((a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y)) as i32)
        }
        _ => return Ok(None),
    }))
}

/// Extracts the byte-slice argument of `update/doFinal/digest` style
/// calls: ([B), ([BII) or a ByteBuffer.
fn sig_bytes(vm: &Engine, args: &[Value], sig: &str) -> Option<Vec<u8>> {
    let first = args.first()?;
    if sig.contains("II") {
        let all = bytes_of(vm, first);
        let off = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
        let len = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
        return Some(all.get(off..off + len).unwrap_or(&[]).to_vec());
    }
    if let Value::Obj(o) = first {
        if vm.heap.as_array_elems(*o).is_ok() {
            return Some(bytes_of(vm, first));
        }
        if let Ok(Value::Obj(arr)) = vm.heap.get_field(*o, "bb_arr") {
            let pos = match vm.heap.get_field(*o, "bb_pos") {
                Ok(Value::Int(i)) => i as usize,
                _ => 0,
            };
            let lim = match vm.heap.get_field(*o, "bb_lim") {
                Ok(Value::Int(i)) => i as usize,
                _ => usize::MAX,
            };
            let all = bytes_of(vm, &Value::Obj(arr));
            return Some(all.get(pos..lim.min(all.len())).unwrap_or(&[]).to_vec());
        }
    }
    Some(bytes_of(vm, first))
}

pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match recv_class {
        SECRET_KEY_SPEC => Ok(Some(match name {
            "<init>" => {
                let key = args.first().cloned().unwrap_or(Value::Null);
                let kb = bytes_of(vm, &key);
                let kb = if args.len() >= 3 && sig.contains("II") {
                    let off = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let len = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    kb.get(off..off + len).unwrap_or(&[]).to_vec()
                } else {
                    kb
                };
                let arr = byte_array(vm, &kb)?;
                put(vm, recv, "jc_key", arr)?;
                let alg = args.last().cloned().unwrap_or(Value::Null);
                put(vm, recv, "jc_alg", alg)?;
                Value::Null
            }
            "getEncoded" => vm.heap.get_field(recv, "jc_key").unwrap_or(Value::Null),
            "getAlgorithm" => vm.heap.get_field(recv, "jc_alg").unwrap_or(Value::Null),
            "getFormat" => Value::Obj(crate::intrinsics::alloc_string(vm, "RAW".to_string())?),
            _ => return Ok(None),
        })),
        "Ljavax/crypto/spec/IvParameterSpec;" => Ok(Some(match name {
            "<init>" => {
                let kb = bytes_of(vm, &args.first().cloned().unwrap_or(Value::Null));
                let kb = if args.len() >= 3 {
                    let off = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let len = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    kb.get(off..off + len).unwrap_or(&[]).to_vec()
                } else {
                    kb
                };
                let arr = byte_array(vm, &kb)?;
                put(vm, recv, "jc_iv", arr)?;
                Value::Null
            }
            "getIV" => vm.heap.get_field(recv, "jc_iv").unwrap_or(Value::Null),
            _ => return Ok(None),
        })),
        "Ljavax/crypto/spec/GCMParameterSpec;" => Ok(Some(match name {
            "<init>" => {
                put(
                    vm,
                    recv,
                    "jc_tagbits",
                    args.first().cloned().unwrap_or(Value::Int(128)),
                )?;
                let src = args.get(1).cloned().unwrap_or(Value::Null);
                let kb = bytes_of(vm, &src);
                let kb = if args.len() >= 4 {
                    let off = args.get(2).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    let len = args.get(3).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    kb.get(off..off + len).unwrap_or(&[]).to_vec()
                } else {
                    kb
                };
                let arr = byte_array(vm, &kb)?;
                put(vm, recv, "jc_iv", arr)?;
                Value::Null
            }
            "getIV" => vm.heap.get_field(recv, "jc_iv").unwrap_or(Value::Null),
            "getTLen" => Value::Int(geti(vm, recv, "jc_tagbits", 128)),
            _ => return Ok(None),
        })),
        CIPHER => cipher_instance(vm, recv, name, sig, args),
        MAC => Ok(Some(match name {
            "init" => {
                let kb = key_bytes_of(vm, &args.first().cloned().unwrap_or(Value::Null));
                let arr = byte_array(vm, &kb)?;
                put(vm, recv, "jc_key", arr)?;
                buf_set(vm, recv, &[])?;
                Value::Null
            }
            "update" => {
                let mut buf = buf_get(vm, recv);
                if let Some(b) = sig_bytes(vm, args, sig) {
                    buf.extend_from_slice(&b);
                } else if let Some(v) = args.first().and_then(|v| v.as_int().ok()) {
                    buf.push(v as u8);
                }
                buf_set(vm, recv, &buf)?;
                Value::Null
            }
            "doFinal" => {
                let mut buf = buf_get(vm, recv);
                if !args.is_empty() {
                    if let Some(b) = sig_bytes(vm, args, sig) {
                        buf.extend_from_slice(&b);
                    }
                }
                let alg = gets(vm, recv, "jc_alg");
                let key = match vm.heap.get_field(recv, "jc_key") {
                    Ok(v) => bytes_of(vm, &v),
                    _ => Vec::new(),
                };
                let out = hmac(&alg, &key, &buf).ok_or_else(|| {
                    exc(
                        "Ljava/security/NoSuchAlgorithmException;",
                        "HMAC indisponível",
                    )
                })?;
                buf_set(vm, recv, &[])?;
                byte_array(vm, &out)?
            }
            "getAlgorithm" => {
                let a = gets(vm, recv, "jc_alg");
                Value::Obj(crate::intrinsics::alloc_string(vm, a)?)
            }
            "getMacLength" => {
                let a = gets(vm, recv, "jc_alg");
                Value::Int(digest_len(
                    a.trim_start_matches("Hmac").trim_start_matches("HMAC"),
                ))
            }
            "reset" => {
                buf_set(vm, recv, &[])?;
                Value::Null
            }
            "clone" => Value::Obj(recv),
            _ => return Ok(None),
        })),
        KEY_GENERATOR => Ok(Some(match name {
            "init" => {
                if let Some(sz) = args.first().and_then(|v| v.as_int().ok()) {
                    put(vm, recv, "jc_keysize", Value::Int(sz))?;
                }
                Value::Null
            }
            "generateKey" => {
                let bits = geti(vm, recv, "jc_keysize", 128);
                let n = vm.fw.jcrypto_key_counter;
                vm.fw.jcrypto_key_counter = n.wrapping_add(1);
                let mut seed = b"rd-keygen".to_vec();
                seed.extend_from_slice(&n.to_be_bytes());
                seed.extend_from_slice(&bits.to_be_bytes());
                let digest = sha256(&seed);
                let key = &digest[..(bits / 8) as usize];
                let spec = vm
                    .heap
                    .alloc_instance(SECRET_KEY_SPEC.to_string(), Vec::new())
                    .map_err(crate::err::oom_throwable)?;
                let arr = byte_array(vm, key)?;
                put(vm, spec, "jc_key", arr)?;
                let alg = crate::intrinsics::alloc_string(vm, "AES".to_string())?;
                put(vm, spec, "jc_alg", Value::Obj(alg))?;
                Value::Obj(spec)
            }
            _ => return Ok(None),
        })),
        MESSAGE_DIGEST => Ok(Some(match name {
            "update" => {
                let mut buf = buf_get(vm, recv);
                if sig == "(B)V" {
                    buf.push(args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) as u8);
                } else if let Some(b) = sig_bytes(vm, args, sig) {
                    buf.extend_from_slice(&b);
                }
                buf_set(vm, recv, &buf)?;
                Value::Null
            }
            "digest" => {
                let mut buf = buf_get(vm, recv);
                if sig == "([B)I" {
                    let alg = gets(vm, recv, "jc_alg");
                    let out = digest_for(&alg, &buf).unwrap_or_default();
                    buf_set(vm, recv, &[])?;
                    if let Some(Value::Obj(dst)) = args.first().cloned() {
                        if let Ok(elems) = vm.heap.array_elems_mut(dst) {
                            for (slot, b) in elems.iter_mut().zip(out.iter()) {
                                *slot = Value::Int(*b as i8 as i32);
                            }
                        }
                    }
                    return Ok(Some(Value::Int(out.len() as i32)));
                }
                if !args.is_empty() {
                    if let Some(b) = sig_bytes(vm, args, sig) {
                        buf.extend_from_slice(&b);
                    }
                }
                let alg = gets(vm, recv, "jc_alg");
                let out = digest_for(&alg, &buf).unwrap_or_default();
                buf_set(vm, recv, &[])?;
                byte_array(vm, &out)?
            }
            "reset" => {
                buf_set(vm, recv, &[])?;
                Value::Null
            }
            "getAlgorithm" => {
                let a = gets(vm, recv, "jc_alg");
                Value::Obj(crate::intrinsics::alloc_string(vm, a)?)
            }
            "getDigestLength" => Value::Int(digest_len(&gets(vm, recv, "jc_alg"))),
            "clone" => Value::Obj(recv),
            _ => return Ok(None),
        })),
        _ => Ok(None),
    }
}

fn cipher_instance(
    vm: &mut Engine,
    recv: ObjRef,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    Ok(Some(match name {
        "<init>" => Value::Null,
        "init" => {
            let opmode = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
            put(vm, recv, "jc_opmode", Value::Int(opmode))?;
            let key = key_bytes_of(vm, &args.get(1).cloned().unwrap_or(Value::Null));
            let arr = byte_array(vm, &key)?;
            put(vm, recv, "jc_key", arr)?;
            buf_set(vm, recv, &[])?;
            buf_set_aad(vm, recv, &[])?;
            // optional AlgorithmParameterSpec (Iv/GCM) as third arg
            if let Some(Value::Obj(spec)) = args.get(2).cloned() {
                if let Ok(iv) = vm.heap.get_field(spec, "jc_iv") {
                    put(vm, recv, "jc_iv", iv)?;
                }
                let t = geti(vm, spec, "jc_tagbits", 0);
                if t != 0 {
                    put(vm, recv, "jc_tagbits", Value::Int(t))?;
                }
            }
            if vm.heap.get_field(recv, "jc_iv").is_err() {
                let arr = byte_array(vm, &[0u8; 16])?;
                put(vm, recv, "jc_iv", arr)?;
            }
            Value::Null
        }
        "updateAAD" => {
            let mut aad = buf_get_aad(vm, recv);
            if let Some(b) = sig_bytes(vm, args, sig) {
                aad.extend_from_slice(&b);
            }
            buf_set_aad(vm, recv, &aad)?;
            Value::Null
        }
        "update" => {
            let mut buf = buf_get(vm, recv);
            if let Some(b) = sig_bytes(vm, args, sig) {
                buf.extend_from_slice(&b);
            }
            buf_set(vm, recv, &buf)?;
            if sig.ends_with("[B") {
                byte_array(vm, &[])?
            } else {
                Value::Int(0)
            }
        }
        "doFinal" => {
            let mut buf = buf_get(vm, recv);
            if !args.is_empty() {
                if sig == "([B)[B" || sig.contains("II") {
                    if let Some(b) = sig_bytes(vm, args, sig) {
                        buf.extend_from_slice(&b);
                    }
                } else if let Some(Value::Obj(dst)) = args.first().cloned() {
                    // doFinal(byte[] out, int off) style
                    let out = cipher_crypt(vm, recv, &buf)?;
                    buf_set(vm, recv, &[])?;
                    let off = args.get(1).and_then(|v| v.as_int().ok()).unwrap_or(0) as usize;
                    if let Ok(elems) = vm.heap.array_elems_mut(dst) {
                        for (slot, b) in elems.iter_mut().skip(off).zip(out.iter()) {
                            *slot = Value::Int(*b as i8 as i32);
                        }
                    }
                    return Ok(Some(Value::Int(out.len() as i32)));
                }
            }
            let out = cipher_crypt(vm, recv, &buf)?;
            buf_set(vm, recv, &[])?;
            byte_array(vm, &out)?
        }
        "getIV" => vm.heap.get_field(recv, "jc_iv").unwrap_or(Value::Null),
        "getBlockSize" => Value::Int(16),
        "getOutputSize" => {
            let n = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0);
            Value::Int(n + 32)
        }
        "getAlgorithm" => {
            let a = gets(vm, recv, "jc_alg");
            Value::Obj(crate::intrinsics::alloc_string(vm, a)?)
        }
        _ => return Ok(None),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_fips197() {
        let key: [u8; 16] = std::array::from_fn(|i| i as u8);
        let pt: [u8; 16] = std::array::from_fn(|i| (i * 0x11) as u8);
        let ct = aes_block(&key, &pt, true);
        assert_eq!(
            ct,
            [
                0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4,
                0xc5, 0x5a
            ]
        );
        assert_eq!(aes_block(&key, &ct, false), pt);
    }

    #[test]
    fn sha256_abc() {
        assert_eq!(
            sha256(b"abc"),
            [
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad
            ]
        );
    }

    #[test]
    fn gcm_empty_vector() {
        // NIST GCM test case: zero key/IV, empty plaintext -> tag only.
        let key = [0u8; 16];
        let iv = [0u8; 12];
        let ct = gcm_crypt(&key, &iv, &[]);
        assert!(ct.is_empty());
        let tag = gcm_tag(&key, &iv, &[], &ct);
        assert_eq!(
            tag,
            [
                0x58, 0xe2, 0xfc, 0xce, 0xfa, 0x7e, 0x30, 0x61, 0x36, 0x7f, 0x1d, 0x57, 0xa4, 0xe7,
                0x45, 0x5a
            ]
        );
    }
}
