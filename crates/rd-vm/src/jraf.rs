//! `java.io.RandomAccessFile` sobre o disco virtual de `jfile` (`os_*`).
//!
//! Modos `r`, `rw`, `rws` e `rwd` (sync não modelado: o disco é virtual e
//! síncrono). Leitura/escrita de primitivas em big-endian, como em
//! `DataInput`/`DataOutput`. Erros de E/S viram `IOException`, a abertura
//! vira `FileNotFoundException` e fim de arquivo em `readFully`/`readInt`
//! vira `EOFException`.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

pub(crate) const RAF: &str = "Ljava/io/RandomAccessFile;";
const FILE: &str = "Ljava/io/File;";
const FD: &str = "Ljava/io/FileDescriptor;";
const FD_FIELD: &str = "raf_fd";
const O_RDONLY: i32 = 0;
const O_RDWR: i32 = 2;
const O_CREAT: i32 = 0o100;
const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;

fn io_err(msg: impl Into<String>) -> VmExit {
    VmExit::Exception(Throwable::new("Ljava/io/IOException;", msg.into()))
}

fn errno_text(e: i32) -> &'static str {
    match e {
        2 => "No such file or directory",
        13 => "Permission denied",
        21 => "Is a directory",
        _ => "I/O error",
    }
}

fn map_errno(e: i32) -> VmExit {
    io_err(errno_text(e))
}

fn fd_of(vm: &Engine, r: ObjRef) -> Result<i32, VmExit> {
    match vm.heap.get_field(r, FD_FIELD) {
        Ok(Value::Int(fd)) if fd >= 0 => Ok(fd),
        _ => Err(io_err("Stream Closed")),
    }
}

fn str_of(vm: &Engine, v: &Value) -> Option<String> {
    match v {
        Value::Obj(o) => vm.heap.as_str(*o).ok().map(str::to_string),
        _ => None,
    }
}

/// Caminho de um `java.io.File` (campo `rd_path`) ou de uma `String`.
fn path_arg(vm: &Engine, v: &Value) -> Option<String> {
    if let Value::Obj(o) = v {
        if vm.heap.class_of(*o).map(|c| c == FILE).unwrap_or(false) {
            return match vm.heap.get_field(*o, "rd_path") {
                Ok(p) => str_of(vm, &p),
                _ => None,
            };
        }
    }
    str_of(vm, v)
}

/// Bytes de um `byte[]` (elementos `Value::Int` com sinal de 8 bits).
fn array_bytes(vm: &Engine, arr: ObjRef) -> Vec<u8> {
    vm.heap
        .as_array_elems(arr)
        .map(|(e, _)| {
            e.iter()
                .map(|x| x.as_int().map(|i| i as u8).unwrap_or(0))
                .collect()
        })
        .unwrap_or_default()
}

/// Grava `data` em `arr[off..off+data.len()]`.
fn write_array(vm: &mut Engine, arr: ObjRef, off: usize, data: &[u8]) -> Result<(), VmExit> {
    let elems = vm
        .heap
        .array_elems_mut(arr)
        .map_err(|e| VmExit::Exception(Throwable::new("Ljava/lang/IllegalStateException;", e)))?;
    for (slot, b) in elems.iter_mut().skip(off).zip(data.iter()) {
        *slot = Value::Int(*b as i8 as i32);
    }
    Ok(())
}

fn arr_arg(args: &[Value], i: usize) -> Result<ObjRef, VmExit> {
    match args.get(i) {
        Some(Value::Obj(o)) => Ok(*o),
        _ => Err(VmExit::Exception(Throwable::new(
            "Ljava/lang/NullPointerException;",
            "RandomAccessFile: byte[] nulo",
        ))),
    }
}

fn int_arg(args: &[Value], i: usize) -> i32 {
    args.get(i).and_then(|v| v.as_int().ok()).unwrap_or(0)
}

fn long_arg(args: &[Value], i: usize) -> i64 {
    args.get(i).and_then(|v| v.as_long().ok()).unwrap_or(0)
}

/// Lê exatamente `n` bytes ou lança `EOFException` (readFully/readInt/readLong).
fn read_exact(vm: &mut Engine, fd: i32, n: usize) -> Result<Vec<u8>, VmExit> {
    let h = vm.fw.vfs.clone();
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let chunk = crate::jfile::os_read(&h, fd, n - out.len()).map_err(map_errno)?;
        if chunk.is_empty() {
            return Err(VmExit::Exception(Throwable::new(
                "Ljava/io/EOFException;",
                String::new(),
            )));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

fn write_all(vm: &mut Engine, fd: i32, data: &[u8]) -> Result<(), VmExit> {
    let h = vm.fw.vfs.clone();
    let mut done = 0;
    while done < data.len() {
        let n = crate::jfile::os_write(&h, fd, &data[done..]).map_err(map_errno)?;
        if n == 0 {
            return Err(io_err("write retornou 0 bytes"));
        }
        done += n;
    }
    Ok(())
}

pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if recv_class != RAF {
        return Ok(None);
    }
    if name == "<init>" {
        if sig != "(Ljava/io/File;Ljava/lang/String;)V"
            && sig != "(Ljava/lang/String;Ljava/lang/String;)V"
        {
            return Ok(None);
        }
        let path = args.first().and_then(|v| path_arg(vm, v)).ok_or_else(|| {
            VmExit::Exception(Throwable::new("Ljava/lang/NullPointerException;", "file"))
        })?;
        let mode = args.get(1).and_then(|v| str_of(vm, v)).unwrap_or_default();
        let flags = match mode.as_str() {
            "r" => O_RDONLY,
            "rw" | "rws" | "rwd" => O_RDWR | O_CREAT,
            _ => {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IllegalArgumentException;",
                    format!(
                        "Illegal mode \"{mode}\" must be one of \"r\", \"rw\", \"rws\", or \"rwd\""
                    ),
                )))
            }
        };
        let h = vm.fw.vfs.clone();
        match crate::jfile::os_open(&h, &path, flags, 0o666) {
            Ok(fd) => {
                crate::err::put_field_result(vm.heap.put_field(recv, FD_FIELD, Value::Int(fd)))?;
            }
            Err(e) => {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/io/FileNotFoundException;",
                    format!("{path} ({})", errno_text(e)),
                )))
            }
        }
        return Ok(Some(Value::Null));
    }
    let v = match (name, sig) {
        ("close", "()V") => {
            if let Ok(Value::Int(fd)) = vm.heap.get_field(recv, FD_FIELD) {
                if fd >= 0 {
                    let h = vm.fw.vfs.clone();
                    let _ = crate::jfile::os_close(&h, fd);
                    crate::err::put_field_result(vm.heap.put_field(
                        recv,
                        FD_FIELD,
                        Value::Int(-1),
                    ))?;
                }
            }
            Value::Null
        }
        // FileDescriptor com o mesmo fd do VFS (campo `descriptor`, como em
        // `jfile`): `sync`/`valid` do FileDescriptor já respondem sobre ele.
        ("getFD", "()Ljava/io/FileDescriptor;") => {
            let fd = fd_of(vm, recv)?;
            let o = vm
                .heap
                .alloc_instance(
                    FD.to_string(),
                    vec![("descriptor".to_string(), Value::Int(fd))],
                )
                .map_err(crate::err::oom_throwable)?;
            Value::Obj(o)
        }
        ("read", "()I") => {
            let fd = fd_of(vm, recv)?;
            let h = vm.fw.vfs.clone();
            let b = crate::jfile::os_read(&h, fd, 1).map_err(map_errno)?;
            Value::Int(b.first().map(|x| *x as i32).unwrap_or(-1))
        }
        ("read", "([B)I") | ("read", "([BII)I") => {
            let fd = fd_of(vm, recv)?;
            let arr = arr_arg(args, 0)?;
            let (off, len) = if sig == "([BII)I" {
                (int_arg(args, 1), int_arg(args, 2))
            } else {
                (
                    0,
                    vm.heap
                        .as_array_elems(arr)
                        .map(|(e, _)| e.len() as i32)
                        .unwrap_or(0),
                )
            };
            let alen = vm
                .heap
                .as_array_elems(arr)
                .map(|(e, _)| e.len())
                .unwrap_or(0) as i32;
            if off < 0 || len < 0 || off.checked_add(len).is_none_or(|e| e > alen) {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IndexOutOfBoundsException;",
                    format!("off={off} len={len} alen={alen}"),
                )));
            }
            if len == 0 {
                Value::Int(0)
            } else {
                let h = vm.fw.vfs.clone();
                let data = crate::jfile::os_read(&h, fd, len as usize).map_err(map_errno)?;
                if data.is_empty() {
                    Value::Int(-1)
                } else {
                    write_array(vm, arr, off as usize, &data)?;
                    Value::Int(data.len() as i32)
                }
            }
        }
        ("readFully", "([B)V") => {
            let fd = fd_of(vm, recv)?;
            let arr = arr_arg(args, 0)?;
            let n = vm
                .heap
                .as_array_elems(arr)
                .map(|(e, _)| e.len())
                .unwrap_or(0);
            let data = read_exact(vm, fd, n)?;
            write_array(vm, arr, 0, &data)?;
            Value::Null
        }
        ("readInt", "()I") => {
            let fd = fd_of(vm, recv)?;
            let b = read_exact(vm, fd, 4)?;
            Value::Int(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        }
        ("readLong", "()J") => {
            let fd = fd_of(vm, recv)?;
            let b = read_exact(vm, fd, 8)?;
            let mut a = [0u8; 8];
            a.copy_from_slice(&b);
            Value::Long(i64::from_be_bytes(a))
        }
        ("write", "(I)V") => {
            let fd = fd_of(vm, recv)?;
            write_all(vm, fd, &[int_arg(args, 0) as u8])?;
            Value::Null
        }
        ("write", "([B)V") | ("write", "([BII)V") => {
            let fd = fd_of(vm, recv)?;
            let arr = arr_arg(args, 0)?;
            let all = array_bytes(vm, arr);
            let (off, len) = if sig == "([BII)V" {
                (int_arg(args, 1), int_arg(args, 2))
            } else {
                (0, all.len() as i32)
            };
            if off < 0 || len < 0 || off as usize + len as usize > all.len() {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IndexOutOfBoundsException;",
                    format!("off={off} len={len} alen={}", all.len()),
                )));
            }
            write_all(vm, fd, &all[off as usize..(off + len) as usize])?;
            Value::Null
        }
        ("writeInt", "(I)V") => {
            let fd = fd_of(vm, recv)?;
            write_all(vm, fd, &int_arg(args, 0).to_be_bytes())?;
            Value::Null
        }
        ("writeLong", "(J)V") => {
            let fd = fd_of(vm, recv)?;
            write_all(vm, fd, &long_arg(args, 0).to_be_bytes())?;
            Value::Null
        }
        ("seek", "(J)V") => {
            let fd = fd_of(vm, recv)?;
            let pos = long_arg(args, 0);
            if pos < 0 {
                return Err(io_err("Negative seek offset"));
            }
            let h = vm.fw.vfs.clone();
            crate::jfile::os_lseek(&h, fd, pos, SEEK_SET).map_err(map_errno)?;
            Value::Null
        }
        ("getFilePointer", "()J") => {
            let fd = fd_of(vm, recv)?;
            let h = vm.fw.vfs.clone();
            Value::Long(crate::jfile::os_lseek(&h, fd, 0, SEEK_CUR).map_err(map_errno)?)
        }
        ("length", "()J") => {
            let fd = fd_of(vm, recv)?;
            let h = vm.fw.vfs.clone();
            Value::Long(crate::jfile::os_fstat(&h, fd).map_err(map_errno)?.size)
        }
        ("setLength", "(J)V") => {
            let fd = fd_of(vm, recv)?;
            let h = vm.fw.vfs.clone();
            crate::jfile::os_ftruncate(&h, fd, long_arg(args, 0)).map_err(map_errno)?;
            Value::Null
        }
        _ => return Ok(None),
    };
    Ok(Some(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::VmConfig;

    fn vm() -> Engine {
        let vm = Engine::new(Vec::new(), VmConfig::default());
        crate::jfile::mkdirs_pub(&vm.fw.vfs.clone(), "/data/raf");
        vm
    }

    fn jstr(vm: &mut Engine, s: &str) -> Value {
        Value::Obj(crate::intrinsics::alloc_string(vm, s.to_string()).unwrap())
    }

    fn new_raf(vm: &mut Engine, path: &str, mode: &str) -> Result<ObjRef, VmExit> {
        let recv = vm.heap.alloc_instance(RAF.to_string(), vec![]).unwrap();
        let p = jstr(vm, path);
        let m = jstr(vm, mode);
        instance_intrinsic(
            vm,
            RAF,
            "<init>",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            recv,
            &[p, m],
        )?;
        Ok(recv)
    }

    fn call(
        vm: &mut Engine,
        r: ObjRef,
        name: &str,
        sig: &str,
        args: &[Value],
    ) -> Result<Option<Value>, VmExit> {
        instance_intrinsic(vm, RAF, name, sig, r, args)
    }

    fn exc_class(e: VmExit) -> String {
        match e {
            VmExit::Exception(t) => t.class,
            VmExit::Error(e) => format!("error:{e:?}"),
        }
    }

    #[test]
    fn rw_round_trip_big_endian_and_seek() {
        let mut vm = vm();
        let f = new_raf(&mut vm, "/data/raf/a.bin", "rw").unwrap();
        call(&mut vm, f, "writeInt", "(I)V", &[Value::Int(0x0102_0304)]).unwrap();
        call(&mut vm, f, "writeLong", "(J)V", &[Value::Long(-2)]).unwrap();
        assert_eq!(
            call(&mut vm, f, "length", "()J", &[]).unwrap(),
            Some(Value::Long(12))
        );
        assert_eq!(
            call(&mut vm, f, "getFilePointer", "()J", &[]).unwrap(),
            Some(Value::Long(12))
        );
        call(&mut vm, f, "seek", "(J)V", &[Value::Long(0)]).unwrap();
        assert_eq!(
            call(&mut vm, f, "readInt", "()I", &[]).unwrap(),
            Some(Value::Int(0x0102_0304))
        );
        assert_eq!(
            call(&mut vm, f, "readLong", "()J", &[]).unwrap(),
            Some(Value::Long(-2))
        );
        // Fim de arquivo em readInt: EOFException, como em DataInput.
        let e = call(&mut vm, f, "readInt", "()I", &[]).unwrap_err();
        assert_eq!(exc_class(e), "Ljava/io/EOFException;");
        call(&mut vm, f, "close", "()V", &[]).unwrap();
        let e = call(&mut vm, f, "readInt", "()I", &[]).unwrap_err();
        assert_eq!(
            exc_class(e),
            "Ljava/io/IOException;",
            "após close: Stream Closed"
        );
    }

    #[test]
    fn read_only_missing_file_is_file_not_found() {
        let mut vm = vm();
        let e = new_raf(&mut vm, "/data/raf/nao_existe.bin", "r")
            .err()
            .unwrap();
        assert_eq!(exc_class(e), "Ljava/io/FileNotFoundException;");
    }

    #[test]
    fn invalid_mode_is_illegal_argument() {
        let mut vm = vm();
        let e = new_raf(&mut vm, "/data/raf/x.bin", "x").err().unwrap();
        assert_eq!(exc_class(e), "Ljava/lang/IllegalArgumentException;");
    }

    #[test]
    fn set_length_truncates_and_read_returns_eof_minus_one() {
        let mut vm = vm();
        let f = new_raf(&mut vm, "/data/raf/t.bin", "rw").unwrap();
        call(&mut vm, f, "writeLong", "(J)V", &[Value::Long(7)]).unwrap();
        call(&mut vm, f, "setLength", "(J)V", &[Value::Long(3)]).unwrap();
        assert_eq!(
            call(&mut vm, f, "length", "()J", &[]).unwrap(),
            Some(Value::Long(3))
        );
        call(&mut vm, f, "seek", "(J)V", &[Value::Long(3)]).unwrap();
        assert_eq!(
            call(&mut vm, f, "read", "()I", &[]).unwrap(),
            Some(Value::Int(-1))
        );
    }

    #[test]
    fn negative_seek_is_io_exception() {
        let mut vm = vm();
        let f = new_raf(&mut vm, "/data/raf/s.bin", "rw").unwrap();
        let e = call(&mut vm, f, "seek", "(J)V", &[Value::Long(-1)]).unwrap_err();
        assert_eq!(exc_class(e), "Ljava/io/IOException;");
    }
}
