//! `java.util.Timer` e `java.util.TimerTask` sobre o Looper principal.
//!
//! Modelo: cada agendamento vira uma mensagem do Looper com deadline no
//! relógio virtual (`fw.clock_ms`). Tarefa periódica re-enfileira um "tick"
//! host (`Ljava/util/Timer$Tick;`) até ser cancelada. Desvio documentado:
//! `run()` executa na thread principal (no Android roda na thread do Timer).
//! `purge()` devolve 0 (não remove entradas canceladas da fila).
//! `schedule(TimerTask, Date)` não é modelado.

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::ObjRef;
use crate::value::Value;

const TIMER: &str = "Ljava/util/Timer;";
const TIMER_TASK: &str = "Ljava/util/TimerTask;";
/// Runnable host enfileirado no Looper; não existe no classpath DEX.
pub(crate) const TIMER_TICK: &str = "Ljava/util/Timer$Tick;";

const F_CANCELLED: &str = "rd_cancelled";
const F_SCHEDULED: &str = "rd_scheduled";
const F_RAN: &str = "rd_ran";

fn exc(class: &str, msg: &str) -> VmExit {
    VmExit::Exception(Throwable::new(class, msg.to_string()))
}

fn flag(vm: &Engine, r: ObjRef, f: &str) -> bool {
    matches!(vm.heap.get_field(r, f), Ok(Value::Int(x)) if x != 0)
}

fn set_flag(vm: &mut Engine, r: ObjRef, f: &str, on: bool) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, Value::Int(on as i32)))
}

fn long_at(args: &[Value], i: usize) -> i64 {
    args.get(i).and_then(|v| v.as_long().ok()).unwrap_or(0)
}

fn obj_at(args: &[Value], i: usize) -> Result<ObjRef, VmExit> {
    match args.get(i) {
        Some(Value::Obj(o)) => Ok(*o),
        _ => Err(exc("Ljava/lang/NullPointerException;", "task")),
    }
}

/// Agenda uma execução de `task` em `when`. `period` 0 = uma vez;
/// `fixed_rate` define a base do próximo disparo (início previsto vs. agora).
fn enqueue_tick(
    vm: &mut Engine,
    timer: ObjRef,
    task: ObjRef,
    when: u64,
    period: i64,
    fixed_rate: bool,
) -> Result<(), VmExit> {
    let tick = vm
        .heap
        .alloc_instance(
            TIMER_TICK.to_string(),
            vec![
                ("rd_task".to_string(), Value::Obj(task)),
                ("rd_timer".to_string(), Value::Obj(timer)),
                ("rd_period".to_string(), Value::Long(period)),
                ("rd_fixed".to_string(), Value::Int(fixed_rate as i32)),
                ("rd_when".to_string(), Value::Long(when as i64)),
            ],
        )
        .map_err(crate::err::oom_throwable)?;
    crate::handler_msg::enqueue_runnable(vm, timer, tick, None, when)
}

fn schedule(
    vm: &mut Engine,
    timer: ObjRef,
    args: &[Value],
    period: i64,
    fixed_rate: bool,
    has_period: bool,
) -> Result<(), VmExit> {
    let task = obj_at(args, 0)?;
    let delay = long_at(args, 1);
    if flag(vm, timer, F_CANCELLED) {
        return Err(exc(
            "Ljava/lang/IllegalStateException;",
            "Timer already cancelled.",
        ));
    }
    if delay < 0 {
        return Err(exc(
            "Ljava/lang/IllegalArgumentException;",
            "Negative delay.",
        ));
    }
    if has_period && period <= 0 {
        return Err(exc(
            "Ljava/lang/IllegalArgumentException;",
            "Non-positive period.",
        ));
    }
    if flag(vm, task, F_SCHEDULED) || flag(vm, task, F_CANCELLED) {
        return Err(exc(
            "Ljava/lang/IllegalStateException;",
            "Task already scheduled or cancelled",
        ));
    }
    set_flag(vm, task, F_SCHEDULED, true)?;
    let when = vm.fw.clock_ms.saturating_add(delay as u64);
    enqueue_tick(
        vm,
        timer,
        task,
        when,
        if has_period { period } else { 0 },
        fixed_rate,
    )
}

/// Executa um tick enfileirado. Chamado pelo Looper quando a `Runnable` não
/// tem `run()` DEX (ver `framework::run_looper_message`).
pub(crate) fn run_tick(vm: &mut Engine, tick: ObjRef) -> Result<(), VmExit> {
    let task = match vm.heap.get_field(tick, "rd_task") {
        Ok(Value::Obj(o)) => o,
        _ => return Ok(()),
    };
    let timer = match vm.heap.get_field(tick, "rd_timer") {
        Ok(Value::Obj(o)) => o,
        _ => return Ok(()),
    };
    if flag(vm, timer, F_CANCELLED) || flag(vm, task, F_CANCELLED) {
        return Ok(());
    }
    let period = match vm.heap.get_field(tick, "rd_period") {
        Ok(Value::Long(p)) => p,
        _ => 0,
    };
    let fixed_rate = flag(vm, tick, "rd_fixed");
    let when = match vm.heap.get_field(tick, "rd_when") {
        Ok(Value::Long(w)) => w as u64,
        _ => vm.fw.clock_ms,
    };
    set_flag(vm, task, F_RAN, true)?;
    crate::jcollections::guest_call(vm, task, "run", "()V", &[])?;
    if period > 0 && !flag(vm, timer, F_CANCELLED) && !flag(vm, task, F_CANCELLED) {
        let next = if fixed_rate {
            when.saturating_add(period as u64)
        } else {
            vm.fw.clock_ms.saturating_add(period as u64)
        };
        enqueue_tick(vm, timer, task, next, period, fixed_rate)?;
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
    match recv_class {
        TIMER => {
            let v = match (name, sig) {
                ("<init>", "()V" | "(Z)V" | "(Ljava/lang/String;)V" | "(Ljava/lang/String;Z)V") => {
                    Value::Null
                }
                ("schedule", "(Ljava/util/TimerTask;J)V") => {
                    schedule(vm, recv, args, 0, false, false)?;
                    Value::Null
                }
                ("schedule", "(Ljava/util/TimerTask;JJ)V") => {
                    schedule(vm, recv, args, long_at(args, 2), false, true)?;
                    Value::Null
                }
                ("scheduleAtFixedRate", "(Ljava/util/TimerTask;JJ)V") => {
                    schedule(vm, recv, args, long_at(args, 2), true, true)?;
                    Value::Null
                }
                ("cancel", "()V") => {
                    set_flag(vm, recv, F_CANCELLED, true)?;
                    Value::Null
                }
                ("purge", "()I") => Value::Int(0),
                _ => return Ok(None),
            };
            Ok(Some(v))
        }
        TIMER_TASK => {
            let v = match (name, sig) {
                ("<init>", "()V") => Value::Null,
                // Java: true se estava agendada e ainda não rodou (uma vez).
                ("cancel", "()Z") => {
                    let pending = flag(vm, recv, F_SCHEDULED)
                        && !flag(vm, recv, F_CANCELLED)
                        && !flag(vm, recv, F_RAN);
                    set_flag(vm, recv, F_CANCELLED, true)?;
                    Value::Int(pending as i32)
                }
                _ => return Ok(None),
            };
            Ok(Some(v))
        }
        TIMER_TICK if name == "run" && sig == "()V" => {
            run_tick(vm, recv)?;
            Ok(Some(Value::Null))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::VmConfig;

    fn vm() -> Engine {
        Engine::new(Vec::new(), VmConfig::default())
    }

    fn new_timer(vm: &mut Engine) -> ObjRef {
        let t = vm.heap.alloc_instance(TIMER.to_string(), vec![]).unwrap();
        instance_intrinsic(vm, TIMER, "<init>", "()V", t, &[]).unwrap();
        t
    }

    fn new_task(vm: &mut Engine) -> ObjRef {
        vm.heap
            .alloc_instance(TIMER_TASK.to_string(), vec![])
            .unwrap()
    }

    fn exc_class(e: VmExit) -> String {
        match e {
            VmExit::Exception(t) => t.class,
            VmExit::Error(e) => format!("error:{e:?}"),
        }
    }

    #[test]
    fn negative_delay_and_non_positive_period_are_rejected() {
        let mut vm = vm();
        let t = new_timer(&mut vm);
        let task = new_task(&mut vm);
        let e = instance_intrinsic(
            &mut vm,
            TIMER,
            "schedule",
            "(Ljava/util/TimerTask;J)V",
            t,
            &[Value::Obj(task), Value::Long(-5)],
        )
        .unwrap_err();
        assert_eq!(exc_class(e), "Ljava/lang/IllegalArgumentException;");
        let task2 = new_task(&mut vm);
        let e = instance_intrinsic(
            &mut vm,
            TIMER,
            "scheduleAtFixedRate",
            "(Ljava/util/TimerTask;JJ)V",
            t,
            &[Value::Obj(task2), Value::Long(0), Value::Long(0)],
        )
        .unwrap_err();
        assert_eq!(exc_class(e), "Ljava/lang/IllegalArgumentException;");
    }

    #[test]
    fn schedule_after_cancel_is_illegal_state() {
        let mut vm = vm();
        let t = new_timer(&mut vm);
        instance_intrinsic(&mut vm, TIMER, "cancel", "()V", t, &[]).unwrap();
        let task = new_task(&mut vm);
        let e = instance_intrinsic(
            &mut vm,
            TIMER,
            "schedule",
            "(Ljava/util/TimerTask;J)V",
            t,
            &[Value::Obj(task), Value::Long(0)],
        )
        .unwrap_err();
        assert_eq!(exc_class(e), "Ljava/lang/IllegalStateException;");
    }

    #[test]
    fn task_cancel_reports_whether_it_was_pending() {
        let mut vm = vm();
        let task = new_task(&mut vm);
        let r = instance_intrinsic(&mut vm, TIMER_TASK, "cancel", "()Z", task, &[]).unwrap();
        assert_eq!(r, Some(Value::Int(0)), "nunca agendada: false");
        let t = new_timer(&mut vm);
        let task2 = new_task(&mut vm);
        instance_intrinsic(
            &mut vm,
            TIMER,
            "schedule",
            "(Ljava/util/TimerTask;J)V",
            t,
            &[Value::Obj(task2), Value::Long(10)],
        )
        .unwrap();
        let r = instance_intrinsic(&mut vm, TIMER_TASK, "cancel", "()Z", task2, &[]).unwrap();
        assert_eq!(r, Some(Value::Int(1)), "agendada e ainda não rodou: true");
    }
}
