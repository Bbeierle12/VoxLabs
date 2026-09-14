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

pub(crate) struct JErr(pub(crate) String);
impl From<jni::errors::Error> for JErr {
    fn from(e: jni::errors::Error) -> Self {
        JErr(format!("{e}"))
    }
}

pub(crate) fn with_activity<T>(
    f: impl FnOnce(&mut Env, &JObject) -> Result<T, JErr>,
) -> Result<T, String> {
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

/// `Intent.FLAG_ACTIVITY_NEW_TASK`.
const FLAG_ACTIVITY_NEW_TASK: i32 = 0x1000_0000;

/// Opens this app's page in the system Settings (Permissions → Microphone
/// lives there) — the way to grant the permission by hand on a phone
/// without `adb`, when the dialog could not be shown or was dismissed.
pub fn open_app_settings() -> bool {
    with_activity(|env, activity| {
        let package = env
            .call_method(
                activity,
                jni_str!("getPackageName"),
                jni_sig!("()Ljava/lang/String;"),
                &[],
            )?
            .l()?;
        let intent_class = env.find_class(jni_str!("android/content/Intent"))?;
        let action = env.new_string("android.settings.APPLICATION_DETAILS_SETTINGS")?;
        let intent = env.new_object(
            &intent_class,
            jni_sig!("(Ljava/lang/String;)V"),
            &[JValue::Object(&action)],
        )?;
        let uri_class = env.find_class(jni_str!("android/net/Uri"))?;
        let scheme = env.new_string("package")?;
        let null = JObject::null();
        let uri = env
            .call_static_method(
                &uri_class,
                jni_str!("fromParts"),
                jni_sig!(
                    "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Landroid/net/Uri;"
                ),
                &[
                    JValue::Object(&scheme),
                    JValue::Object(&package),
                    JValue::Object(&null),
                ],
            )?
            .l()?;
        env.call_method(
            &intent,
            jni_str!("setData"),
            jni_sig!("(Landroid/net/Uri;)Landroid/content/Intent;"),
            &[JValue::Object(&uri)],
        )?;
        env.call_method(
            &intent,
            jni_str!("addFlags"),
            jni_sig!("(I)Landroid/content/Intent;"),
            &[JValue::Int(FLAG_ACTIVITY_NEW_TASK)],
        )?;
        env.call_method(
            activity,
            jni_str!("startActivity"),
            jni_sig!("(Landroid/content/Intent;)V"),
            &[JValue::Object(&intent)],
        )?;
        Ok(())
    })
    .map(|()| true)
    .unwrap_or_else(|e| {
        log::warn!("could not open the app's Settings page: {e}");
        false
    })
}
