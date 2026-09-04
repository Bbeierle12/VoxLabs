//! The Android share-sheet entry: "Share → VoxLabs" from Drive, a file
//! manager, a voice recorder — any app that sends an `audio/*` stream.
//!
//! The manifest (Cargo.toml `[package.metadata.android]`) registers the
//! activity for `ACTION_SEND` and `ACTION_VIEW` with `audio/*`. A share
//! creates a new instance of the activity whose `Intent` carries the
//! content URI; NativeActivity gives us no `onActivityResult`/`onNewIntent`,
//! so the intent is read once, here, at `android_main` time, before the UI
//! exists. The shared stream is copied into the import folder (content
//! URIs are not paths — the only portable way to read one is through the
//! `ContentResolver`), and the path is handed to the UI to analyze on its
//! first frame.
//!
//! All JNI, all best-effort: any failure logs and yields `None`, and the
//! app starts normally.

use crate::audio_file;
use jni::objects::{JObject, JString, JValue};
use jni::{Env, jni_sig, jni_str};
use std::fs::File;
use std::os::fd::FromRawFd;
use std::path::{Path, PathBuf};

const ACTION_SEND: &str = "android.intent.action.SEND";
const ACTION_VIEW: &str = "android.intent.action.VIEW";
const EXTRA_STREAM: &str = "android.intent.extra.STREAM";
const DISPLAY_NAME_COLUMN: &str = "_display_name";

/// Error carrier that satisfies `E: From<jni::errors::Error>` for the
/// attach callback while staying printable.
struct JErr(String);
impl From<jni::errors::Error> for JErr {
    fn from(e: jni::errors::Error) -> Self {
        JErr(format!("{e}"))
    }
}
impl From<std::io::Error> for JErr {
    fn from(e: std::io::Error) -> Self {
        JErr(format!("{e}"))
    }
}

/// If this activity was started by a share/open of an audio stream, copy
/// it into `import_dir` and return the new path.
pub fn take_shared_audio(import_dir: &Path) -> Option<PathBuf> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };
    let activity_ptr = ctx.context();
    let result: Result<Option<PathBuf>, JErr> = vm.attach_current_thread(|env| {
        let activity = unsafe { JObject::from_raw(env, activity_ptr as jni::sys::jobject) };
        let r = take_in_env(env, &activity, import_dir);
        if env.exception_check() {
            env.exception_clear();
        }
        r
    });
    match result {
        Ok(Some(p)) => {
            log::info!("share intent: imported {}", p.display());
            Some(p)
        }
        Ok(None) => None,
        Err(JErr(e)) => {
            log::warn!("share intent: could not import the shared file: {e}");
            None
        }
    }
}

fn get_string(env: &mut Env, obj: JObject) -> Option<String> {
    if obj.is_null() {
        return None;
    }
    // Every call site got this from an API returning java.lang.String.
    let js = unsafe { JString::from_raw(env, obj.into_raw()) };
    js.try_to_string(env).ok()
}

fn take_in_env(
    env: &mut Env,
    activity: &JObject,
    import_dir: &Path,
) -> Result<Option<PathBuf>, JErr> {
    let intent = env
        .call_method(
            activity,
            jni_str!("getIntent"),
            jni_sig!("()Landroid/content/Intent;"),
            &[],
        )?
        .l()?;
    if intent.is_null() {
        return Ok(None);
    }
    let action = env
        .call_method(
            &intent,
            jni_str!("getAction"),
            jni_sig!("()Ljava/lang/String;"),
            &[],
        )?
        .l()?;
    let action = get_string(env, action).unwrap_or_default();
    let uri = match action.as_str() {
        ACTION_SEND => {
            let key = env.new_string(EXTRA_STREAM)?;
            env.call_method(
                &intent,
                jni_str!("getParcelableExtra"),
                jni_sig!("(Ljava/lang/String;)Landroid/os/Parcelable;"),
                &[JValue::Object(&key)],
            )?
            .l()?
        }
        ACTION_VIEW => env
            .call_method(
                &intent,
                jni_str!("getData"),
                jni_sig!("()Landroid/net/Uri;"),
                &[],
            )?
            .l()?,
        // A plain launch.
        _ => return Ok(None),
    };
    if uri.is_null() {
        return Ok(None);
    }
    let mime = env
        .call_method(
            &intent,
            jni_str!("getType"),
            jni_sig!("()Ljava/lang/String;"),
            &[],
        )
        .ok()
        .and_then(|v| v.l().ok())
        .and_then(|o| get_string(env, o));

    let resolver = env
        .call_method(
            activity,
            jni_str!("getContentResolver"),
            jni_sig!("()Landroid/content/ContentResolver;"),
            &[],
        )?
        .l()?;

    let name = display_name(env, &resolver, &uri)
        .or_else(|| last_path_segment(env, &uri))
        .unwrap_or_else(|| "shared-audio".to_string());
    let name = file_name_for(&name, mime.as_deref());

    // Open through the resolver and take the fd for ourselves.
    let mode = env.new_string("r")?;
    let pfd = env
        .call_method(
            &resolver,
            jni_str!("openFileDescriptor"),
            jni_sig!("(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;"),
            &[JValue::Object(&uri), JValue::Object(&mode)],
        )?
        .l()?;
    if pfd.is_null() {
        return Err(JErr("openFileDescriptor returned null".into()));
    }
    let fd = env
        .call_method(&pfd, jni_str!("detachFd"), jni_sig!("()I"), &[])?
        .i()?;
    if fd < 0 {
        return Err(JErr("detachFd returned an invalid descriptor".into()));
    }
    // SAFETY: `detachFd` transferred ownership of a live descriptor to us;
    // nothing on the Java side closes it now.
    let mut src = unsafe { File::from_raw_fd(fd) };

    std::fs::create_dir_all(import_dir)?;
    let dest = audio_file::unique_path(import_dir, &name);
    let mut out = File::create(&dest)?;
    let bytes = std::io::copy(&mut src, &mut out)?;
    if bytes == 0 {
        let _ = std::fs::remove_file(&dest);
        return Err(JErr("the shared stream was empty".into()));
    }
    Ok(Some(dest))
}

/// `OpenableColumns.DISPLAY_NAME` via a one-row query; the standard way to
/// name a content URI.
fn display_name(env: &mut Env, resolver: &JObject, uri: &JObject) -> Option<String> {
    let cursor = env
        .call_method(
            resolver,
            jni_str!("query"),
            jni_sig!(
                "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;"
            ),
            &[
                JValue::Object(uri),
                JValue::Object(&JObject::null()),
                JValue::Object(&JObject::null()),
                JValue::Object(&JObject::null()),
                JValue::Object(&JObject::null()),
            ],
        )
        .ok()?
        .l()
        .ok()?;
    if cursor.is_null() {
        return None;
    }
    let name = (|| -> Result<Option<String>, JErr> {
        let first = env
            .call_method(&cursor, jni_str!("moveToFirst"), jni_sig!("()Z"), &[])?
            .z()?;
        if !first {
            return Ok(None);
        }
        let col = env.new_string(DISPLAY_NAME_COLUMN)?;
        let idx = env
            .call_method(
                &cursor,
                jni_str!("getColumnIndex"),
                jni_sig!("(Ljava/lang/String;)I"),
                &[JValue::Object(&col)],
            )?
            .i()?;
        if idx < 0 {
            return Ok(None);
        }
        let s = env
            .call_method(
                &cursor,
                jni_str!("getString"),
                jni_sig!("(I)Ljava/lang/String;"),
                &[JValue::Int(idx)],
            )?
            .l()?;
        Ok(get_string(env, s))
    })();
    let _ = env.call_method(&cursor, jni_str!("close"), jni_sig!("()V"), &[]);
    if env.exception_check() {
        env.exception_clear();
    }
    name.ok().flatten().filter(|n| !n.trim().is_empty())
}

fn last_path_segment(env: &mut Env, uri: &JObject) -> Option<String> {
    let s = env
        .call_method(
            uri,
            jni_str!("getLastPathSegment"),
            jni_sig!("()Ljava/lang/String;"),
            &[],
        )
        .ok()?
        .l()
        .ok()?;
    get_string(env, s).filter(|n| !n.trim().is_empty())
}

/// A safe file name for the import folder: no path separators, and an
/// audio extension (guessed from the MIME type when the name has none, so
/// the folder listing and the decoder's format hint both work).
fn file_name_for(name: &str, mime: Option<&str>) -> String {
    let mut n: String = name
        .chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == '\0' {
                '_'
            } else {
                c
            }
        })
        .collect();
    if !audio_file::is_audio_file(Path::new(&n)) {
        let ext = match mime.map(|m| m.trim().to_ascii_lowercase()).as_deref() {
            Some("audio/mpeg" | "audio/mp3" | "audio/mpeg3") => "mp3",
            Some("audio/wav" | "audio/x-wav" | "audio/wave" | "audio/vnd.wave") => "wav",
            Some("audio/flac" | "audio/x-flac") => "flac",
            Some("audio/mp4" | "audio/x-m4a" | "audio/m4a" | "audio/mp4a-latm") => "m4a",
            Some("audio/aac" | "audio/aacp") => "aac",
            Some("audio/ogg" | "application/ogg" | "audio/vorbis") => "ogg",
            _ => return n,
        };
        n.push('.');
        n.push_str(ext);
    }
    n
}

#[cfg(test)]
mod tests {
    use super::file_name_for;

    #[test]
    fn names_are_sanitized_and_get_an_extension_from_the_mime() {
        assert_eq!(file_name_for("take one.wav", None), "take one.wav");
        assert_eq!(file_name_for("a/b\\c.mp3", None), "a_b_c.mp3");
        assert_eq!(file_name_for("12345", Some("audio/mpeg")), "12345.mp3");
        assert_eq!(file_name_for("rec", Some("audio/x-wav")), "rec.wav");
        assert_eq!(file_name_for("rec", Some("application/ogg")), "rec.ogg");
        assert_eq!(file_name_for("rec.txt", Some("audio/mp4")), "rec.txt.m4a");
        assert_eq!(
            file_name_for("rec", Some("application/octet-stream")),
            "rec"
        );
    }
}
