//! Where the bundle goes and what the bundle says about the build/device.
//!
//! Android: the file is inserted into `MediaStore.Downloads` under
//! `Downloads/VoxLabs` (no storage permission needed on API 29+), then
//! offered to the share sheet — the source app's `exportBundle` path. App
//! and device facts come from `PackageManager` and `android.os.Build`.
//!
//! Desktop: the file is written to `~/Downloads/VoxLabs/`; there is no
//! share sheet, so the path is the "URI".

use serde_json::{Value, json};

use super::store::{DOWNLOAD_SUBDIR, Export};

#[cfg(not(target_os = "android"))]
pub fn write_download(name: &str, bytes: &[u8]) -> Result<Export, String> {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or("no home directory")?;
    let dir = home.join("Downloads").join(DOWNLOAD_SUBDIR);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(name);
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(Export {
        display_name: name.to_string(),
        uri: path.display().to_string(),
        bytes: bytes.len(),
    })
}

/// Desktop has no share sheet; the file is already where the user can find it.
#[cfg(not(target_os = "android"))]
pub fn share(_export: &Export) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn app_json() -> Value {
    json!({
        "package": env!("CARGO_PKG_NAME"),
        "version_name": env!("CARGO_PKG_VERSION"),
        "version_code": 0,
    })
}

#[cfg(not(target_os = "android"))]
pub fn device_json() -> Value {
    json!({
        "manufacturer": "desktop",
        "model": std::env::consts::OS,
        "android_api": 0,
        "android_release": "n/a",
        "supported_abis": [std::env::consts::ARCH],
    })
}

#[cfg(target_os = "android")]
pub use android::{app_json, device_json, share, write_download};

#[cfg(target_os = "android")]
mod android {
    use super::*;
    use crate::permission::{JErr, with_activity};
    use jni::objects::{JObject, JObjectArray, JString, JValue};
    use jni::strings::JNIStr;
    use jni::{Env, jni_sig, jni_str};

    /// `Intent.FLAG_GRANT_READ_URI_PERMISSION`.
    const FLAG_GRANT_READ_URI_PERMISSION: i32 = 1;
    const MIME: &str = "application/json";

    fn get_string(env: &mut Env, obj: JObject) -> Option<String> {
        if obj.is_null() {
            return None;
        }
        // Every call site got this from an API returning java.lang.String.
        let js = unsafe { JString::from_raw(env, obj.into_raw()) };
        js.try_to_string(env).ok()
    }

    fn static_string(env: &mut Env, class: &JNIStr, field: &JNIStr) -> Option<String> {
        let v = env
            .get_static_field(class, field, jni_sig!("Ljava/lang/String;"))
            .ok()?
            .l()
            .ok()?;
        get_string(env, v)
    }

    fn put_string(env: &mut Env, values: &JObject, key: &str, value: &str) -> Result<(), JErr> {
        let k = env.new_string(key)?;
        let v = env.new_string(value)?;
        env.call_method(
            values,
            jni_str!("put"),
            jni_sig!("(Ljava/lang/String;Ljava/lang/String;)V"),
            &[JValue::Object(&k), JValue::Object(&v)],
        )?;
        Ok(())
    }

    fn put_int(env: &mut Env, values: &JObject, key: &str, value: i32) -> Result<(), JErr> {
        let k = env.new_string(key)?;
        let boxed = env
            .call_static_method(
                jni_str!("java/lang/Integer"),
                jni_str!("valueOf"),
                jni_sig!("(I)Ljava/lang/Integer;"),
                &[JValue::Int(value)],
            )?
            .l()?;
        env.call_method(
            values,
            jni_str!("put"),
            jni_sig!("(Ljava/lang/String;Ljava/lang/Integer;)V"),
            &[JValue::Object(&k), JValue::Object(&boxed)],
        )?;
        Ok(())
    }

    pub fn write_download(name: &str, bytes: &[u8]) -> Result<Export, String> {
        let len = bytes.len();
        with_activity(|env, activity| {
            let downloads_dir = env
                .get_static_field(
                    jni_str!("android/os/Environment"),
                    jni_str!("DIRECTORY_DOWNLOADS"),
                    jni_sig!("Ljava/lang/String;"),
                )?
                .l()?;
            let downloads_dir = get_string(env, downloads_dir).unwrap_or_else(|| "Download".into());
            let values = env.new_object(
                jni_str!("android/content/ContentValues"),
                jni_sig!("()V"),
                &[],
            )?;
            put_string(env, &values, "_display_name", name)?;
            put_string(env, &values, "mime_type", MIME)?;
            put_string(
                env,
                &values,
                "relative_path",
                &format!("{downloads_dir}/{DOWNLOAD_SUBDIR}"),
            )?;
            put_int(env, &values, "is_pending", 1)?;

            let resolver = env
                .call_method(
                    activity,
                    jni_str!("getContentResolver"),
                    jni_sig!("()Landroid/content/ContentResolver;"),
                    &[],
                )?
                .l()?;
            let collection = env
                .get_static_field(
                    jni_str!("android/provider/MediaStore$Downloads"),
                    jni_str!("EXTERNAL_CONTENT_URI"),
                    jni_sig!("Landroid/net/Uri;"),
                )?
                .l()?;
            let uri = env
                .call_method(
                    &resolver,
                    jni_str!("insert"),
                    jni_sig!("(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;"),
                    &[JValue::Object(&collection), JValue::Object(&values)],
                )?
                .l()?;
            if uri.is_null() {
                return Err(JErr(
                    "Android could not create the diagnostics download".into(),
                ));
            }
            let written = (|| -> Result<(), JErr> {
                let mode = env.new_string("w")?;
                let stream = env
                    .call_method(
                        &resolver,
                        jni_str!("openOutputStream"),
                        jni_sig!("(Landroid/net/Uri;Ljava/lang/String;)Ljava/io/OutputStream;"),
                        &[JValue::Object(&uri), JValue::Object(&mode)],
                    )?
                    .l()?;
                if stream.is_null() {
                    return Err(JErr("openOutputStream returned null".into()));
                }
                let array = env.byte_array_from_slice(bytes)?;
                env.call_method(
                    &stream,
                    jni_str!("write"),
                    jni_sig!("([B)V"),
                    &[JValue::Object(array.as_ref())],
                )?;
                env.call_method(&stream, jni_str!("close"), jni_sig!("()V"), &[])?;
                env.call_method(&values, jni_str!("clear"), jni_sig!("()V"), &[])?;
                put_int(env, &values, "is_pending", 0)?;
                env.call_method(
                    &resolver,
                    jni_str!("update"),
                    jni_sig!(
                        "(Landroid/net/Uri;Landroid/content/ContentValues;Ljava/lang/String;[Ljava/lang/String;)I"
                    ),
                    &[
                        JValue::Object(&uri),
                        JValue::Object(&values),
                        JValue::Object(&JObject::null()),
                        JValue::Object(&JObject::null()),
                    ],
                )?;
                Ok(())
            })();
            if let Err(e) = written {
                if env.exception_check() {
                    env.exception_clear();
                }
                let _ = env.call_method(
                    &resolver,
                    jni_str!("delete"),
                    jni_sig!("(Landroid/net/Uri;Ljava/lang/String;[Ljava/lang/String;)I"),
                    &[
                        JValue::Object(&uri),
                        JValue::Object(&JObject::null()),
                        JValue::Object(&JObject::null()),
                    ],
                );
                return Err(e);
            }
            let uri_text = env
                .call_method(
                    &uri,
                    jni_str!("toString"),
                    jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?;
            let uri_text = get_string(env, uri_text).ok_or(JErr("Uri.toString failed".into()))?;
            Ok(Export {
                display_name: name.to_string(),
                uri: uri_text,
                bytes: len,
            })
        })
    }

    /// `Intent.createChooser(ACTION_SEND application/json)` with the bundle's URI.
    pub fn share(export: &Export) -> Result<(), String> {
        let uri_text = export.uri.clone();
        with_activity(move |env, activity| {
            let uri_js = env.new_string(&uri_text)?;
            let uri = env
                .call_static_method(
                    jni_str!("android/net/Uri"),
                    jni_str!("parse"),
                    jni_sig!("(Ljava/lang/String;)Landroid/net/Uri;"),
                    &[JValue::Object(&uri_js)],
                )?
                .l()?;
            let action = env.new_string("android.intent.action.SEND")?;
            let intent = env.new_object(
                jni_str!("android/content/Intent"),
                jni_sig!("(Ljava/lang/String;)V"),
                &[JValue::Object(&action)],
            )?;
            let mime = env.new_string(MIME)?;
            env.call_method(
                &intent,
                jni_str!("setType"),
                jni_sig!("(Ljava/lang/String;)Landroid/content/Intent;"),
                &[JValue::Object(&mime)],
            )?;
            let stream_key = env.new_string("android.intent.extra.STREAM")?;
            env.call_method(
                &intent,
                jni_str!("putExtra"),
                jni_sig!("(Ljava/lang/String;Landroid/os/Parcelable;)Landroid/content/Intent;"),
                &[JValue::Object(&stream_key), JValue::Object(&uri)],
            )?;
            let subject_key = env.new_string("android.intent.extra.SUBJECT")?;
            let subject = env.new_string("VoxLabs diagnostic bundle")?;
            env.call_method(
                &intent,
                jni_str!("putExtra"),
                jni_sig!("(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;"),
                &[JValue::Object(&subject_key), JValue::Object(&subject)],
            )?;
            env.call_method(
                &intent,
                jni_str!("addFlags"),
                jni_sig!("(I)Landroid/content/Intent;"),
                &[JValue::Int(FLAG_GRANT_READ_URI_PERMISSION)],
            )?;
            let title = env.new_string("Share diagnostic bundle")?;
            let chooser = env
                .call_static_method(
                    jni_str!("android/content/Intent"),
                    jni_str!("createChooser"),
                    jni_sig!(
                        "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;"
                    ),
                    &[JValue::Object(&intent), JValue::Object(&title)],
                )?
                .l()?;
            env.call_method(
                activity,
                jni_str!("startActivity"),
                jni_sig!("(Landroid/content/Intent;)V"),
                &[JValue::Object(&chooser)],
            )?;
            Ok(())
        })
    }

    pub fn app_json() -> Value {
        with_activity(|env, activity| {
            let package = env
                .call_method(
                    activity,
                    jni_str!("getPackageName"),
                    jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?;
            let package = get_string(env, package).unwrap_or_default();
            let package_js = env.new_string(&package)?;
            let pm = env
                .call_method(
                    activity,
                    jni_str!("getPackageManager"),
                    jni_sig!("()Landroid/content/pm/PackageManager;"),
                    &[],
                )?
                .l()?;
            let info = env
                .call_method(
                    &pm,
                    jni_str!("getPackageInfo"),
                    jni_sig!("(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;"),
                    &[JValue::Object(&package_js), JValue::Int(0)],
                )?
                .l()?;
            let version_name = env
                .get_field(
                    &info,
                    jni_str!("versionName"),
                    jni_sig!("Ljava/lang/String;"),
                )?
                .l()?;
            let version_name = get_string(env, version_name).unwrap_or_else(|| "unknown".into());
            let version_code = env
                .call_method(&info, jni_str!("getLongVersionCode"), jni_sig!("()J"), &[])?
                .j()?;
            Ok(json!({
                "package": package,
                "version_name": version_name,
                "version_code": version_code,
            }))
        })
        .unwrap_or_else(|e| json!({ "package": "unknown", "error": e }))
    }

    pub fn device_json() -> Value {
        with_activity(|env, _activity| {
            let manufacturer =
                static_string(env, jni_str!("android/os/Build"), jni_str!("MANUFACTURER"))
                    .unwrap_or_default();
            let model = static_string(env, jni_str!("android/os/Build"), jni_str!("MODEL"))
                .unwrap_or_default();
            let release = static_string(
                env,
                jni_str!("android/os/Build$VERSION"),
                jni_str!("RELEASE"),
            )
            .unwrap_or_default();
            let api = env
                .get_static_field(
                    jni_str!("android/os/Build$VERSION"),
                    jni_str!("SDK_INT"),
                    jni_sig!("I"),
                )?
                .i()?;
            let abis_obj = env
                .get_static_field(
                    jni_str!("android/os/Build"),
                    jni_str!("SUPPORTED_ABIS"),
                    jni_sig!("[Ljava/lang/String;"),
                )?
                .l()?;
            let mut abis = Vec::new();
            if !abis_obj.is_null() {
                // Declared `String[]` in the platform API.
                let array: JObjectArray<JObject> =
                    unsafe { JObjectArray::<JObject>::from_raw(env, abis_obj.into_raw()) };
                let n = array.len(env)?;
                for i in 0..n {
                    let item = array.get_element(env, i)?;
                    if let Some(s) = get_string(env, item) {
                        abis.push(s);
                    }
                }
            }
            Ok(json!({
                "manufacturer": manufacturer,
                "model": model,
                "android_api": api,
                "android_release": release,
                "supported_abis": abis,
            }))
        })
        .unwrap_or_else(|e| json!({ "error": e }))
    }
}
