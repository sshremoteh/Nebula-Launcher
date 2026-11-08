//! NativeActivity loads its library with dlopen, outside the Java class loader.
//! Bind the result callback explicitly instead of relying on JNI symbol discovery.
use jni::{
    JNIEnv, NativeMethod,
    objects::{JObject, JString},
};

pub type Callback = extern "system" fn(JNIEnv<'_>, JObject<'_>, i32, i32, JString<'_>);

pub fn register(
    env: &mut JNIEnv<'_>,
    activity: &JObject<'_>,
    callback: Callback,
) -> jni::errors::Result<()> {
    let class = env.get_object_class(activity)?;
    env.register_native_methods(
        class,
        &[
            NativeMethod {
                name: "nativeDocumentResult".into(),
                sig: "(IILjava/lang/String;)V".into(),
                fn_ptr: callback as *mut std::ffi::c_void,
            },
            NativeMethod {
                name: "nativeImportUri".into(),
                sig: "(Ljava/lang/String;)V".into(),
                fn_ptr: crate::documents::import_external_uri as *mut std::ffi::c_void,
            },
        ],
    )
}

/// Open the granted document on a worker. No filesystem path is inferred from its URI.
pub fn open(
    env: &mut JNIEnv<'_>,
    activity: &JObject<'_>,
    uri: &str,
    fallback: &str,
) -> Result<(std::fs::File, String), Box<dyn std::error::Error + Send + Sync>> {
    use jni::objects::JValue;
    use std::os::fd::FromRawFd;

    // Direct file:// URI support
    if let Some(path_str) = uri.strip_prefix("file://") {
        let path = std::path::Path::new(path_str);
        if path.is_file() {
            let file = std::fs::File::open(path)?;
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| fallback.to_string());
            return Ok((file, name));
        }
    }

    env.with_local_frame(24, |env| {
        let resolver = env.call_method(
            activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )?.l()?;
        let text = JObject::from(env.new_string(uri)?);
        let uri_obj = env.call_static_method(
            "android/net/Uri",
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&text)],
        )?.l()?;

        // Try getting display name safely
        let mut name: Option<String> = None;
        let projection = env.new_object_array(1, "java/lang/String", JObject::null())?;
        env.set_object_array_element(&projection, 0, env.new_string("_display_name")?)?;
        let null = JObject::null();
        let query_res = env.call_method(
            &resolver,
            "query",
            "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            &[
                JValue::Object(&uri_obj),
                JValue::Object(projection.as_ref()),
                JValue::Object(&null),
                JValue::Object(&null),
                JValue::Object(&null),
            ],
        );
        if let Ok(cursor) = query_res.and_then(|v| v.l()) {
            if !cursor.is_null() {
                let _ = (|| -> jni::errors::Result<()> {
                    if env.call_method(&cursor, "moveToFirst", "()Z", &[])?.z()? {
                        let value = env.call_method(
                            &cursor,
                            "getString",
                            "(I)Ljava/lang/String;",
                            &[JValue::Int(0)],
                        )?.l()?;
                        if !value.is_null() {
                            name = Some(env.get_string((&value).into())?.into());
                        }
                    }
                    Ok(())
                })();
                let _ = env.exception_clear();
                let _ = env.call_method(&cursor, "close", "()V", &[]);
                let _ = env.exception_clear();
            }
        } else {
            let _ = env.exception_clear();
        }

        let name = name.unwrap_or_else(|| {
            if let Some(pos) = uri.rfind('/') {
                let sub = &uri[pos + 1..];
                if !sub.is_empty() && !sub.contains('?') {
                    return sub.to_string();
                }
            }
            fallback.into()
        });

        let mode = JObject::from(env.new_string("r")?);
        let fd = env.call_method(
            &resolver,
            "openFileDescriptor",
            "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
            &[JValue::Object(&uri_obj), JValue::Object(&mode)],
        )?.l()?;
        if fd.is_null() {
            return Err("Document provider returned no file".into());
        }
        let raw = env.call_method(&fd, "detachFd", "()I", &[])?.i()?;
        if raw < 0 {
            return Err("Document provider returned an invalid descriptor".into());
        }
        // detachFd transferred ownership to Rust.
        let file = unsafe { std::fs::File::from_raw_fd(raw) };
        let _ = env.call_method(&fd, "close", "()V", &[]);
        let _ = env.exception_clear();
        Ok((file, name))
    })
}
