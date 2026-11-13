//! `android.media` audio surface for headless runs: `SoundPool`,
//! `AudioAttributes` (+ `Builder`s) and their constants.
//!
//! The VM has no audio device. Games and apps that touch these classes must
//! keep running, so this module models the *contract* and renders nothing:
//!
//! - `SoundPool.load*` hands out increasing, non-zero sound ids (what the app
//!   stores and later passes to `play`), and queues an
//!   `OnLoadCompleteListener.onLoadComplete(pool, id, 0)` callback. The queue
//!   is drained on the main Looper (`drain_looper`), as Android delivers it
//!   asynchronously; the listener is read at delivery time, so one set after
//!   `load` still receives the callback.
//! - `SoundPool.play` accepts the request and returns a positive stream id
//!   (the app's "is playing" state stays consistent); no samples are produced.
//! - `pause`/`resume`/`stop`/`setVolume`/`setRate`/`setLoop`/`release` are
//!   accepted and do nothing. Result: the app runs, and its audio is silent.
//! - `AudioAttributes.Builder` stores usage/content type/flags, so the app's
//!   own reads of `getUsage()`/`getContentType()` round-trip correctly.
//!
//! The constant values mirror AOSP `AudioAttributes` / `AudioManager`.

use crate::engine::Engine;
use crate::err::{put_field_result, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

pub(crate) const SOUND_POOL: &str = "Landroid/media/SoundPool;";
pub(crate) const SOUND_POOL_BUILDER: &str = "Landroid/media/SoundPool$Builder;";
pub(crate) const AUDIO_ATTRIBUTES: &str = "Landroid/media/AudioAttributes;";
pub(crate) const AUDIO_ATTRIBUTES_BUILDER: &str = "Landroid/media/AudioAttributes$Builder;";

/// Instance field holding the app's `OnLoadCompleteListener` (a DEX object).
const ON_LOAD_LISTENER: &str = "rd_on_load_listener";
const ON_LOAD_COMPLETE_SIG: &str = "(Landroid/media/SoundPool;II)V";

/// A finished `SoundPool.load*`, waiting for the Looper to deliver it.
/// Kept in `Engine::pending_sound_loads`, which is a GC root.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingSoundLoad {
    pub pool: ObjRef,
    pub sample_id: i32,
    /// 0 = success (the only status the headless loader reports).
    pub status: i32,
}

/// Delivers queued load completions in FIFO order. Returns how many callbacks
/// ran; a pool whose listener is unset (or a non-listener value) is skipped.
pub(crate) fn deliver_load_completions(vm: &mut Engine) -> Result<usize, VmExit> {
    let pending = std::mem::take(&mut vm.pending_sound_loads);
    let mut delivered = 0usize;
    for p in pending {
        let Ok(Value::Obj(listener)) = vm.heap.get_field(p.pool, ON_LOAD_LISTENER) else {
            continue;
        };
        vm.invoke_instance(
            listener,
            "onLoadComplete",
            ON_LOAD_COMPLETE_SIG,
            &[
                Value::Obj(p.pool),
                Value::Int(p.sample_id),
                Value::Int(p.status),
            ],
        )?;
        delivered += 1;
    }
    Ok(delivered)
}

/// Static constants of the media classes modeled here.
pub(crate) fn host_static(
    vm: &mut Engine,
    owner: &str,
    field: &str,
) -> Result<Option<Value>, VmExit> {
    let _ = vm;
    let v = match (owner, field) {
        (AUDIO_ATTRIBUTES, "USAGE_UNKNOWN") => 0,
        (AUDIO_ATTRIBUTES, "USAGE_MEDIA") => 1,
        (AUDIO_ATTRIBUTES, "USAGE_VOICE_COMMUNICATION") => 2,
        (AUDIO_ATTRIBUTES, "USAGE_VOICE_COMMUNICATION_SIGNALLING") => 3,
        (AUDIO_ATTRIBUTES, "USAGE_ALARM") => 4,
        (AUDIO_ATTRIBUTES, "USAGE_NOTIFICATION") => 5,
        (AUDIO_ATTRIBUTES, "USAGE_NOTIFICATION_RINGTONE") => 6,
        (AUDIO_ATTRIBUTES, "USAGE_NOTIFICATION_COMMUNICATION_REQUEST") => 7,
        (AUDIO_ATTRIBUTES, "USAGE_NOTIFICATION_COMMUNICATION_INSTANT") => 8,
        (AUDIO_ATTRIBUTES, "USAGE_NOTIFICATION_COMMUNICATION_DELAYED") => 9,
        (AUDIO_ATTRIBUTES, "USAGE_NOTIFICATION_EVENT") => 10,
        (AUDIO_ATTRIBUTES, "USAGE_ASSISTANCE_ACCESSIBILITY") => 11,
        (AUDIO_ATTRIBUTES, "USAGE_ASSISTANCE_NAVIGATION_GUIDANCE") => 12,
        (AUDIO_ATTRIBUTES, "USAGE_ASSISTANCE_SONIFICATION") => 13,
        (AUDIO_ATTRIBUTES, "USAGE_GAME") => 14,
        (AUDIO_ATTRIBUTES, "USAGE_ASSISTANT") => 16,
        (AUDIO_ATTRIBUTES, "CONTENT_TYPE_UNKNOWN") => 0,
        (AUDIO_ATTRIBUTES, "CONTENT_TYPE_SPEECH") => 1,
        (AUDIO_ATTRIBUTES, "CONTENT_TYPE_MUSIC") => 2,
        (AUDIO_ATTRIBUTES, "CONTENT_TYPE_MOVIE") => 3,
        (AUDIO_ATTRIBUTES, "CONTENT_TYPE_SONIFICATION") => 4,
        (AUDIO_ATTRIBUTES, "FLAG_AUDIBILITY_ENFORCED") => 0x1,
        (AUDIO_ATTRIBUTES, "FLAG_HW_AV_SYNC") => 0x10,
        (AUDIO_ATTRIBUTES, "FLAG_LOW_LATENCY") => 0x100,
        ("Landroid/media/AudioManager;", "STREAM_VOICE_CALL") => 0,
        ("Landroid/media/AudioManager;", "STREAM_SYSTEM") => 1,
        ("Landroid/media/AudioManager;", "STREAM_RING") => 2,
        ("Landroid/media/AudioManager;", "STREAM_MUSIC") => 3,
        ("Landroid/media/AudioManager;", "STREAM_ALARM") => 4,
        ("Landroid/media/AudioManager;", "STREAM_NOTIFICATION") => 5,
        _ => return Ok(None),
    };
    Ok(Some(Value::Int(v)))
}

fn field_int(vm: &Engine, obj: ObjRef, field: &str) -> i32 {
    vm.heap
        .get_field(obj, field)
        .ok()
        .and_then(|v| v.as_int().ok())
        .unwrap_or(0)
}

fn set_int(vm: &mut Engine, obj: ObjRef, field: &str, v: i32) -> Result<(), VmExit> {
    put_field_result(vm.heap.put_field(obj, field, Value::Int(v)))
}

fn arg_int(args: &[Value], i: usize) -> i32 {
    args.get(i).and_then(|v| v.as_int().ok()).unwrap_or(0)
}

/// Instance methods of the modeled classes. `Ok(None)` = not ours.
pub(crate) fn instance(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match recv_class {
        SOUND_POOL => sound_pool(vm, name, sig, recv, args),
        SOUND_POOL_BUILDER => sound_pool_builder(vm, name, sig, recv, args),
        AUDIO_ATTRIBUTES => audio_attributes(vm, name, sig, recv),
        AUDIO_ATTRIBUTES_BUILDER => audio_attributes_builder(vm, name, sig, recv, args),
        _ => Ok(None),
    }
}

fn sound_pool(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let v = match (name, sig) {
        ("<init>", "(III)V") => {
            set_int(vm, recv, "rd_max_streams", arg_int(args, 0).max(1))?;
            set_int(vm, recv, "rd_next_sound", 1)?;
            set_int(vm, recv, "rd_next_stream", 1)?;
            Value::Null
        }
        ("load", _)
            if sig.starts_with("(Landroid/content/Context;I")
                || sig.starts_with("(Ljava/io/FileDescriptor;")
                || sig.starts_with("(Landroid/content/res/AssetFileDescriptor;")
                || sig.starts_with("(Ljava/lang/String;") =>
        {
            let id = field_int(vm, recv, "rd_next_sound").max(1);
            set_int(vm, recv, "rd_next_sound", id + 1)?;
            vm.pending_sound_loads.push(PendingSoundLoad {
                pool: recv,
                sample_id: id,
                status: 0,
            });
            Value::Int(id)
        }
        ("play", "(IFFIIF)I") => {
            // Positive stream id: the app's playing state stays consistent.
            // Silent: no audio device, nothing is rendered.
            let sid = field_int(vm, recv, "rd_next_stream").max(1);
            set_int(vm, recv, "rd_next_stream", sid + 1)?;
            Value::Int(sid)
        }
        ("unload", "(I)Z") => Value::Int(1),
        ("pause", "(I)V")
        | ("resume", "(I)V")
        | ("stop", "(I)V")
        | ("autoPause", "()V")
        | ("autoResume", "()V")
        | ("release", "()V") => Value::Null,
        ("setVolume", "(IFF)V") | ("setLoop", "(II)V") | ("setPriority", "(II)V") => Value::Null,
        ("setRate", "(IF)V") => Value::Null,
        ("setOnLoadCompleteListener", _) => {
            let listener = args.first().cloned().unwrap_or(Value::Null);
            put_field_result(vm.heap.put_field(recv, ON_LOAD_LISTENER, listener))?;
            Value::Null
        }
        _ => return Ok(None),
    };
    Ok(Some(v))
}

fn sound_pool_builder(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let v = match (name, sig) {
        ("<init>", "()V") => {
            set_int(vm, recv, "rd_max_streams", 1)?;
            Value::Null
        }
        ("setMaxStreams", "(I)Landroid/media/SoundPool$Builder;") => {
            set_int(vm, recv, "rd_max_streams", arg_int(args, 0).max(1))?;
            Value::Obj(recv)
        }
        (
            "setAudioAttributes",
            "(Landroid/media/AudioAttributes;)Landroid/media/SoundPool$Builder;",
        ) => {
            if let Some(Value::Obj(attrs)) = args.first() {
                put_field_result(vm.heap.put_field(recv, "rd_attributes", Value::Obj(*attrs)))?;
            }
            Value::Obj(recv)
        }
        ("build", "()Landroid/media/SoundPool;") => {
            let pool = vm.new_object(SOUND_POOL)?;
            let max = field_int(vm, recv, "rd_max_streams").max(1);
            vm.invoke_instance(
                pool,
                "<init>",
                "(III)V",
                &[Value::Int(max), Value::Int(3), Value::Int(0)],
            )?;
            Value::Obj(pool)
        }
        _ => return Ok(None),
    };
    Ok(Some(v))
}

fn audio_attributes(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
) -> Result<Option<Value>, VmExit> {
    let v = match (name, sig) {
        ("getUsage", "()I") => Value::Int(field_int(vm, recv, "rd_usage")),
        ("getContentType", "()I") => Value::Int(field_int(vm, recv, "rd_content_type")),
        ("getFlags", "()I") => Value::Int(field_int(vm, recv, "rd_flags")),
        ("getVolumeControlStream", "()I") => {
            // AOSP: STREAM_MUSIC for media/game/unknown usages, STREAM_ALARM etc. otherwise.
            let stream = match field_int(vm, recv, "rd_usage") {
                4 => 4,
                5..=10 => 5,
                2 | 3 => 0,
                13 => 5,
                _ => 3,
            };
            Value::Int(stream)
        }
        _ => return Ok(None),
    };
    Ok(Some(v))
}

fn audio_attributes_builder(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let v = match (name, sig) {
        ("<init>", "()V") => {
            set_int(vm, recv, "rd_usage", 0)?;
            set_int(vm, recv, "rd_content_type", 0)?;
            set_int(vm, recv, "rd_flags", 0)?;
            Value::Null
        }
        ("setUsage", "(I)Landroid/media/AudioAttributes$Builder;") => {
            set_int(vm, recv, "rd_usage", arg_int(args, 0))?;
            Value::Obj(recv)
        }
        ("setContentType", "(I)Landroid/media/AudioAttributes$Builder;") => {
            set_int(vm, recv, "rd_content_type", arg_int(args, 0))?;
            Value::Obj(recv)
        }
        ("setFlags", "(I)Landroid/media/AudioAttributes$Builder;") => {
            set_int(vm, recv, "rd_flags", arg_int(args, 0))?;
            Value::Obj(recv)
        }
        ("setLegacyStreamType", "(I)Landroid/media/AudioAttributes$Builder;")
        | ("setAllowedCapturePolicy", "(I)Landroid/media/AudioAttributes$Builder;") => {
            Value::Obj(recv)
        }
        ("build", "()Landroid/media/AudioAttributes;") => {
            let attrs = vm.new_object(AUDIO_ATTRIBUTES)?;
            let usage = field_int(vm, recv, "rd_usage");
            let content = field_int(vm, recv, "rd_content_type");
            let flags = field_int(vm, recv, "rd_flags");
            set_int(vm, attrs, "rd_usage", usage)?;
            set_int(vm, attrs, "rd_content_type", content)?;
            set_int(vm, attrs, "rd_flags", flags)?;
            Value::Obj(attrs)
        }
        _ => return Ok(None),
    };
    Ok(Some(v))
}
