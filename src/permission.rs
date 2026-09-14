//! Runtime microphone permission on Android.
//!
//! `RECORD_AUDIO` is a runtime ("dangerous") permission: the manifest
//! declaration only makes it grantable. A plain `NativeActivity` gets no
//! `onRequestPermissionsResult`, so the app asks through JNI at launch and
//! then polls the grant state; `android_main` opens the audio engine the
//! moment the grant lands (see `android::start_audio_when_permitted`).
//! Before this, every fresh install sat in SEARCHING until someone ran
//! `pm grant` by hand.
//!
//! All JNI, all best-effort: a failure logs and reads as "not granted".

use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};

pub const RECORD_AUDIO: &str = "android.permission.RECORD_AUDIO";
/// `PackageManager.PERMISSION_GRANTED`.
const PERMISSION_GRANTED: i32 = 0;
/// Request code echoed to `onRequestPermissionsResult`, which NativeActivity
/// never delivers to us; any value works.
const REQUEST_CODE: i32 = 1;

struct JErr(String);
impl From<jni::errors::Error> for JErr {
    fn from(e: jni::errors::Error) -> Self {
        JErr(format!("{e}"))
    }
}

fn with_activity<T>(f: impl FnOnce(&mut Env, &JObject) -> Result<T, JErr>) -> Result<T, String> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };
    let activity_ptr = ctx.context();
    let result: Result<T, JErr> = vm.attach_current_thread(|env| {
        let activity = unsafe { JObject::from_raw(env, activity_ptr as jni::sys::jobject) };
        let r = f(env, &activity);
        if env.exception_check() {
            env.exception_clear();
        }
        r
    });
    result.map_err(|JErr(e)| e)
}

/// Whether `RECORD_AUDIO` is currently granted to this app.
pub fn has_record_audio() -> bool {
    with_activity(|env, activity| {
        let name = env.new_string(RECORD_AUDIO)?;
        let r = env
            .call_method(
                activity,
                jni_str!("checkSelfPermission"),
                jni_sig!("(Ljava/lang/String;)I"),
                &[JValue::Object(&name)],
            )?
            .i()?;
        Ok(r == PERMISSION_GRANTED)
    })
    .unwrap_or_else(|e| {
        log::warn!("checkSelfPermission failed: {e}");
        false
    })
}

/// Shows the system's microphone permission dialog. Returns false when
/// the request could not even be made.
pub fn request_record_audio() -> bool {
    with_activity(|env, activity| {
        let string_class = env.find_class(jni_str!("java/lang/String"))?;
        let name = env.new_string(RECORD_AUDIO)?;
        let names = env.new_object_array(1, &string_class, &name)?;
        env.call_method(
            activity,
            jni_str!("requestPermissions"),
            jni_sig!("([Ljava/lang/String;I)V"),
            &[JValue::Object(&names), JValue::Int(REQUEST_CODE)],
        )?;
        Ok(())
    })
    .map(|()| true)
    .unwrap_or_else(|e| {
        log::warn!("requestPermissions failed: {e}");
        false
    })
}
