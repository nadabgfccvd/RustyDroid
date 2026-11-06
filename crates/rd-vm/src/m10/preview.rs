//! `SurfaceView` de preview (CameraX `PreviewView`).
//!
//! Cada `SurfaceView` ganha um `SurfaceHolder` host (`Landroid/view/SurfaceViewHolder;`)
//! e uma `Surface` própria. `addCallback` entrega `surfaceCreated` e
//! `surfaceChanged(format 1, w, h)` de imediato, como o framework faz quando a
//! superfície já existe. O tamanho entregue é o de `setFixedSize` (CameraX pede a
//! resolução do `SurfaceRequest`) ou, sem ele, o da view.
//!
//! Frames da câmera chegam à `Surface` e viram luminância de `PREVIEW_WIDTH` px de
//! largura com a proporção do sensor. Limites documentados: sem recorte, sem
//! rotação e sem escala de hardware (vizinho mais próximo); JPEG não é decodificado.

use std::collections::BTreeMap;

use crate::engine::Engine;
use crate::err::VmExit;
use crate::framework::HostObj;
use crate::heap::ObjRef;
use crate::value::Value;

pub const SURFACE_VIEW: &str = "Landroid/view/SurfaceView;";
pub const SURFACE_HOLDER: &str = "Landroid/view/SurfaceHolder;";
/// Holder host: a implementação concreta devolvida por `SurfaceView.getHolder()`.
pub const SURFACE_HOLDER_HOST: &str = "Landroid/view/SurfaceViewHolder;";
const SURFACE: &str = "Landroid/view/Surface;";

/// Largura do quadro de preview em px (a altura segue a proporção do sensor).
pub const PREVIEW_WIDTH: i32 = 256;
/// Limite de pixels de um frame de entrada (guarda contra buffers hostis).
const MAX_SRC_PIXELS: i64 = 4_000_000;

/// Luminância `w x h`, linha a linha.
#[derive(Clone, Debug)]
pub struct PreviewPixels {
    pub w: usize,
    pub h: usize,
    pub gray: Vec<u8>,
}

/// Estado de uma `SurfaceView`.
#[derive(Clone, Debug)]
pub struct PreviewSurfaceState {
    pub holder: ObjRef,
    pub surface: ObjRef,
    pub callbacks: Vec<ObjRef>,
    pub frame: Option<PreviewPixels>,
    /// Buffer pedido por `SurfaceHolder.setFixedSize`. `None` = tamanho da view.
    pub fixed: Option<(i32, i32)>,
}

#[derive(Clone, Default, Debug)]
pub struct PreviewState {
    /// `SurfaceView` → estado.
    pub views: BTreeMap<ObjRef, PreviewSurfaceState>,
    /// `SurfaceHolder` host → `SurfaceView` dona.
    pub holder_to_view: BTreeMap<ObjRef, ObjRef>,
    /// `Surface` → `SurfaceView` dona.
    pub surface_to_view: BTreeMap<ObjRef, ObjRef>,
}

impl PreviewState {
    /// Último frame de uma `SurfaceView` (para o desenho da UI).
    pub fn frame_of(&self, view: ObjRef) -> Option<&PreviewPixels> {
        self.views.get(&view)?.frame.as_ref()
    }

    /// Dimensões do quadro de preview para um sensor `cam_w x cam_h`.
    pub fn frame_size_for(cam_w: i32, cam_h: i32) -> (i32, i32) {
        if cam_w <= 0 || cam_h <= 0 {
            return (PREVIEW_WIDTH, PREVIEW_WIDTH * 3 / 4);
        }
        let h = (i64::from(PREVIEW_WIDTH) * i64::from(cam_h) / i64::from(cam_w)).max(1) as i32;
        (PREVIEW_WIDTH, h)
    }

    pub(crate) fn gc_keys(&self, out: &mut Vec<ObjRef>) {
        out.extend(self.views.keys().copied());
        out.extend(self.holder_to_view.keys().copied());
        out.extend(self.surface_to_view.keys().copied());
    }

    pub(crate) fn gc_trace_entry(&self, r: ObjRef, out: &mut Vec<ObjRef>) {
        if let Some(st) = self.views.get(&r) {
            out.push(st.holder);
            out.push(st.surface);
            out.extend(st.callbacks.iter().copied());
        }
        if let Some(&view) = self.holder_to_view.get(&r) {
            out.push(view);
        }
        if let Some(&view) = self.surface_to_view.get(&r) {
            out.push(view);
        }
    }

    pub(crate) fn gc_purge(&mut self, alive: &impl Fn(&ObjRef) -> bool) {
        self.views.retain(|v, st| {
            alive(v) && alive(&st.holder) && alive(&st.surface) && st.callbacks.iter().all(alive)
        });
        self.holder_to_view.retain(|h, v| alive(h) && alive(v));
        self.surface_to_view.retain(|s, v| alive(s) && alive(v));
    }
}

/// Cria (uma vez) o holder e a `Surface` da view.
fn ensure_view(vm: &mut Engine, view: ObjRef) -> Result<(), VmExit> {
    if vm.fw.vdev.preview.views.contains_key(&view) {
        return Ok(());
    }
    let holder = vm
        .heap
        .alloc_instance(SURFACE_HOLDER_HOST.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    let surface = vm
        .heap
        .alloc_instance(SURFACE.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    let p = &mut vm.fw.vdev.preview;
    p.views.insert(
        view,
        PreviewSurfaceState {
            holder,
            surface,
            callbacks: Vec::new(),
            frame: None,
            fixed: None,
        },
    );
    p.holder_to_view.insert(holder, view);
    p.surface_to_view.insert(surface, view);
    Ok(())
}

/// Tamanho da `Surface`: o fixo (`setFixedSize`) ou o da view.
fn surface_size(vm: &Engine, view: ObjRef) -> (i32, i32) {
    let fixed = vm.fw.vdev.preview.views.get(&view).and_then(|st| st.fixed);
    let (w, h) = fixed.unwrap_or_else(|| match vm.fw.get(view) {
        Some(HostObj::View { w, h, .. }) => (*w, *h),
        _ => (0, 0),
    });
    (w.max(1), h.max(1))
}

fn surface_changed(
    vm: &mut Engine,
    cb: ObjRef,
    holder: ObjRef,
    w: i32,
    h: i32,
) -> Result<(), VmExit> {
    super::call_dex(
        vm,
        cb,
        "surfaceChanged",
        "(Landroid/view/SurfaceHolder;III)V",
        vec![
            Value::Obj(holder),
            Value::Int(1),
            Value::Int(w),
            Value::Int(h),
        ],
    )
    .map(|_| ())
}

/// `SurfaceHolder.addCallback`: entrega `surfaceCreated` e `surfaceChanged`.
fn add_callback(vm: &mut Engine, view: ObjRef, holder: ObjRef, cb: ObjRef) -> Result<(), VmExit> {
    let Some(st) = vm.fw.vdev.preview.views.get_mut(&view) else {
        return Ok(());
    };
    if st.callbacks.contains(&cb) {
        return Ok(());
    }
    st.callbacks.push(cb);
    let (w, h) = surface_size(vm, view);
    super::call_dex(
        vm,
        cb,
        "surfaceCreated",
        "(Landroid/view/SurfaceHolder;)V",
        vec![Value::Obj(holder)],
    )
    .map(|_| ())?;
    surface_changed(vm, cb, holder, w, h)
}

/// `SurfaceHolder.setFixedSize(w, h)`: com callbacks registrados, o framework
/// reentrega `surfaceChanged` com o novo tamanho.
fn set_fixed_size(vm: &mut Engine, view: ObjRef, w: i32, h: i32) -> Result<(), VmExit> {
    let Some(st) = vm.fw.vdev.preview.views.get_mut(&view) else {
        return Ok(());
    };
    if st.fixed == Some((w, h)) {
        return Ok(());
    }
    st.fixed = Some((w, h));
    let holder = st.holder;
    let callbacks = st.callbacks.clone();
    let (sw, sh) = surface_size(vm, view);
    for cb in callbacks {
        surface_changed(vm, cb, holder, sw, sh)?;
    }
    Ok(())
}

/// Dispatch host de `SurfaceView` / `SurfaceHolder`. `None` = não é desta
/// família (ou o receptor não é uma view/holder deste módulo).
pub fn instance_call(
    vm: &mut Engine,
    class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if class == SURFACE_VIEW {
        if name == "getHolder" && sig == "()Landroid/view/SurfaceHolder;" {
            if !matches!(vm.fw.get(recv), Some(HostObj::View { .. })) {
                return Ok(None);
            }
            ensure_view(vm, recv)?;
            let holder = vm.fw.vdev.preview.views[&recv].holder;
            return Ok(Some(Value::Obj(holder)));
        }
        return Ok(None);
    }
    if class != SURFACE_HOLDER && class != SURFACE_HOLDER_HOST {
        return Ok(None);
    }
    let Some(&view) = vm.fw.vdev.preview.holder_to_view.get(&recv) else {
        return Ok(None);
    };
    let value = match name {
        "addCallback" => {
            let Some(Value::Obj(cb)) = args.first().cloned() else {
                return Err(crate::err::vm_error(
                    "NULL_POINTER",
                    "SurfaceHolder.addCallback(null)".to_string(),
                )
                .into());
            };
            add_callback(vm, view, recv, cb)?;
            Value::Null
        }
        "removeCallback" => {
            if let Some(Value::Obj(cb)) = args.first() {
                if let Some(st) = vm.fw.vdev.preview.views.get_mut(&view) {
                    st.callbacks.retain(|c| c != cb);
                }
            }
            Value::Null
        }
        "getSurface" => Value::Obj(vm.fw.vdev.preview.views[&view].surface),
        "setFixedSize" => {
            let (w, h) = match (args.first(), args.get(1)) {
                (Some(Value::Int(w)), Some(Value::Int(h))) => (*w, *h),
                _ => return Ok(None),
            };
            set_fixed_size(vm, view, w, h)?;
            Value::Null
        }
        "setFormat" | "setType" | "setKeepScreenOn" | "setSizeFromLayout" => Value::Null,
        "isCreating" => Value::Int(0),
        _ => return Ok(None),
    };
    Ok(Some(value))
}

/// Entrega um frame da câmera à `Surface` de uma `SurfaceView`. `false` se a
/// surface não pertence a uma view de preview ou o formato não é suportado.
pub(crate) fn enqueue_preview_frame(
    vm: &mut Engine,
    surface: ObjRef,
    width: i32,
    height: i32,
    format: i32,
    bytes: &[u8],
) -> bool {
    let Some(&view) = vm.fw.vdev.preview.surface_to_view.get(&surface) else {
        return false;
    };
    let Some(frame) = to_gray(width, height, format, bytes) else {
        return false;
    };
    if let Some(st) = vm.fw.vdev.preview.views.get_mut(&view) {
        st.frame = Some(frame);
        return true;
    }
    false
}

/// Converte um frame para luminância no tamanho do quadro de preview.
/// RGBA (formato 1) usa o canal R (stride 4); YUV (35) e demais tratam o início
/// do buffer como plano Y. JPEG (256) não é decodificado.
fn to_gray(width: i32, height: i32, format: i32, bytes: &[u8]) -> Option<PreviewPixels> {
    if width <= 0 || height <= 0 || format == 256 {
        return None;
    }
    let (sw, sh) = (width as usize, height as usize);
    if i64::from(width) * i64::from(height) > MAX_SRC_PIXELS {
        return None;
    }
    let stride = if format == 1 { 4 } else { 1 };
    if bytes.len() < sw * sh * stride {
        return None;
    }
    let (ow, oh) = PreviewState::frame_size_for(width, height);
    let (ow, oh) = (ow as usize, oh as usize);
    let mut gray = Vec::with_capacity(ow * oh);
    for oy in 0..oh {
        let sy = oy * sh / oh;
        for ox in 0..ow {
            let sx = ox * sw / ow;
            gray.push(bytes[(sy * sw + sx) * stride]);
        }
    }
    Some(PreviewPixels { w: ow, h: oh, gray })
}
