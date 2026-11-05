//! Apêndice D (S10) — photo picker com resultado programável pelo agente
//! (`androidx-photopicker`, contratos `PickVisualMedia`/`PickMultipleVisualMedia`).
//!
//! O picker do sistema não existe no sandbox. Em vez de inventar uma seleção,
//! o agente programa o próximo resultado (`enqueue`): uma URI ou o
//! cancelamento. Sem resultado programado, o picker cancela, como o AOSP faz
//! quando o usuário fecha a tela.
//!
//! Caminho modelado: `Activity.startActivityForResult(Intent, int)` com uma
//! ação de picker (`ACTION_PICK_IMAGES`, `ACTION_GET_CONTENT`,
//! `ACTION_OPEN_DOCUMENT`) consome um resultado da fila e chama
//! `onActivityResult(requestCode, resultCode, data)` no DEX da activity.
//! O Intent de resultado leva a URI em `data` (o mesmo `IntentMeta` que
//! `getData()` lê em `bundle.rs`).
//!
//! Desvios documentados (não silenciosos):
//! - A entrega é síncrona, dentro de `startActivityForResult`. No AOSP ela
//!   chega depois que o picker fecha.
//! - Seleção múltipla devolve uma única URI (sem `ClipData`).
//! - `Uri` é opaco: `parse`, `toString` e o acesso ao conteúdo não são
//!   modelados. `ContentResolver` não lê `content://` do sandbox.
//! - Os contratos AndroidX (`ActivityResultContracts$PickVisualMedia`) rodam
//!   como DEX do APK. Não há APK real com `androidx.activity` nos testes.
//! - Outras ações implícitas continuam com `NOT_IMPLEMENTED`.

use std::collections::VecDeque;

use crate::android_sysservices::alloc_obj;
use crate::engine::Engine;
use crate::err::{put_field_result, VmExit};
use crate::framework::INTENT;
use crate::heap::ObjRef;
use crate::intrinsics;
use crate::m8::call_dex;
use crate::value::Value;

pub const ACTION_PICK_IMAGES: &str = "android.media.action.PICK_IMAGES";
pub const ACTION_GET_CONTENT: &str = "android.intent.action.GET_CONTENT";
pub const ACTION_OPEN_DOCUMENT: &str = "android.intent.action.OPEN_DOCUMENT";

/// `Activity.RESULT_OK` e `Activity.RESULT_CANCELED`.
pub const RESULT_OK: i32 = -1;
pub const RESULT_CANCELED: i32 = 0;

pub const URI_CLASS: &str = "Landroid/net/Uri;";
const URI_TEXT: &str = "rd_uri_text";
const ON_ACTIVITY_RESULT_SIG: &str = "(IILandroid/content/Intent;)V";

/// Próximo resultado do picker, programado pelo agente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerAnswer {
    Uri(String),
    Cancel,
}

/// Fila FIFO de resultados programados. Vazia = o picker cancela.
#[derive(Debug, Default)]
pub struct PickerState {
    pub answers: VecDeque<PickerAnswer>,
}

/// `true` para as ações que o host trata como picker.
pub fn is_picker_action(action: &str) -> bool {
    matches!(
        action,
        ACTION_PICK_IMAGES | ACTION_GET_CONTENT | ACTION_OPEN_DOCUMENT
    )
}

/// Programa o próximo resultado do picker.
pub fn enqueue(vm: &mut Engine, answer: PickerAnswer) {
    vm.fw.photopicker.answers.push_back(answer);
}

/// Quantos resultados ainda estão programados.
pub fn pending(vm: &Engine) -> usize {
    vm.fw.photopicker.answers.len()
}

/// Ação do Intent (`new Intent(String)` / `setAction`), lida do modelo
/// `IntentMeta` de `bundle.rs`.
pub fn intent_action(vm: &Engine, intent: ObjRef) -> Option<String> {
    match vm.intent_meta.get(&intent).and_then(|m| m.action.clone()) {
        Some(Value::Obj(s)) => vm.heap.as_str(s).ok().map(str::to_string),
        _ => None,
    }
}

/// Cria um `Uri` opaco com a string dada.
pub fn uri_obj(vm: &mut Engine, text: &str) -> Result<ObjRef, VmExit> {
    let uri = alloc_obj(vm, URI_CLASS, Vec::new())?;
    let s = intrinsics::alloc_string(vm, text.to_string())?;
    put_field_result(vm.heap.put_field(uri, URI_TEXT, Value::Obj(s)))?;
    Ok(uri)
}

/// Texto de um `Uri` criado por `uri_obj`, se houver.
pub fn uri_text(vm: &Engine, uri: ObjRef) -> Option<String> {
    match vm.heap.get_field(uri, URI_TEXT) {
        Ok(Value::Obj(s)) => vm.heap.as_str(s).ok().map(str::to_string),
        _ => None,
    }
}

/// Entrega o próximo resultado ao `onActivityResult` da activity.
/// Sem resultado programado, entrega `RESULT_CANCELED` com data nula.
pub fn deliver(vm: &mut Engine, activity: ObjRef, request_code: i32) -> Result<(), VmExit> {
    let answer = vm
        .fw
        .photopicker
        .answers
        .pop_front()
        .unwrap_or(PickerAnswer::Cancel);
    let (result_code, data) = match answer {
        PickerAnswer::Uri(text) => {
            let uri = uri_obj(vm, &text)?;
            let intent = alloc_obj(vm, INTENT, Vec::new())?;
            vm.intent_meta.entry(intent).or_default().data = Some(Value::Obj(uri));
            (RESULT_OK, Value::Obj(intent))
        }
        PickerAnswer::Cancel => (RESULT_CANCELED, Value::Null),
    };
    // Sem `onActivityResult` no DEX, o padrão do AOSP é não fazer nada.
    call_dex(
        vm,
        activity,
        "onActivityResult",
        ON_ACTIVITY_RESULT_SIG,
        vec![Value::Int(request_code), Value::Int(result_code), data],
    )?;
    Ok(())
}
