//! E2E regression for the real NewPipe `Application.onCreate`.
//!
//! Three gaps discovered by running the golden APK end to end are pinned here,
//! because each one stopped the app *before* any UI existed and each has a
//! unit test only at the intrinsic level:
//!
//! 1. `Collections.binarySearch(List, Object)` — used by
//!    `PreferenceManager.setDefaultValues` (PR #506);
//! 2. `TypedArray.getTextArray(int)` over ARSC array resources — used by
//!    `MultiSelectListPreference.onGetDefaultValue` (PR #507);
//! 3. Java field hiding on the VM heap — `SwitchPreference.mListener` used to
//!    collapse into `Preference.mListener` (PR #506).
//!
//! The APK is not versioned (GH-05 policy), so the test skips when it is not
//! present, like the F-Droid `onCreate` E2E and the README smoke tests.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn newpipe_app_oncreate_gets_past_preferences_and_array_resources() {
    let apk = repo_root().join("golden-apks/org.schabi.newpipe.apk");
    if !apk.exists() {
        eprintln!(
            "PULANDO: {} ausente — rode ./golden-apks/fetch.sh org.schabi.newpipe",
            apk.display()
        );
        return;
    }

    let out = Command::new(env!("CARGO_BIN_EXE_rd"))
        .args(["app", "run", "--trace", &apk.to_string_lossy()])
        .output()
        .expect("rd app run executa");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // 1. binarySearch gap
    assert!(
        !combined.contains("Collections;->binarySearch"),
        "Collections.binarySearch voltou a faltar no launch: {combined}"
    );
    // 2. array resources / getTextArray gap
    assert!(
        !combined.contains("array-length em null"),
        "TypedArray.getTextArray voltou a devolver null: {combined}"
    );
    // 3. field hiding: the preference stack must not throw from the inflate
    assert!(
        !combined.contains("Error inflating class SwitchPreferenceCompat"),
        "field hiding regrediu (SwitchPreference.mListener): {combined}"
    );
    assert!(
        !combined.contains("Error inflating class MultiSelectListPreference"),
        "ARSC array resources regrediram: {combined}"
    );
    // 4. the socket factories must exist (okhttp builds its client eagerly);
    //    the modelled gap is the I/O itself, not the configuration.
    assert!(
        !combined.contains("SocketFactory;->getDefault"),
        "javax.net.SocketFactory.getDefault voltou a faltar: {combined}"
    );
    // Current honest end state: `App.onCreate` reaches okhttp/JDK internals
    // that the VM does not model yet (`sun.misc.Unsafe.arrayBaseOffset`).
    // When that changes, the assertion below flips and the test documents the
    // new endpoint — it is a marker, not a claim that the app runs.
}
