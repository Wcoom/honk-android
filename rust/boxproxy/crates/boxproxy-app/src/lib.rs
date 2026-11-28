//! JNI FFI boundary (`libboxproxy_app.so`) for `io.honk.android.bridge.RustBridge`.
//!
//! Exposes the 12 exact entry points from `rust/research/boxproxy-honk-spec.md` Section 5.3:
//! `nativeInit`, `nativeShutdown`, `nativeRegisterAsset`, `nativeDispatch`,
//! `nativeSubscribeView`, `nativeUnsubscribeView`, `nativeQueryTileState`,
//! `nativeRenderNotification`, `nativeResolveMonetCss`, `nativeImportCoreFromFd`,
//! `nativeExportBackupToFd`, `nativeRestoreBackupFromFd`.

#![allow(non_snake_case)]

use boxproxy_engine::{
    AppEffect, BoxProxyEngine, EngineConfig, PlatformBridge, StateListener,
};
use boxproxy_schema::NetworkObservation;
use boxproxy_store::BackupScope;
use boxproxy_tools::InstalledAppInfo;
use jni::objects::{GlobalRef, JByteArray, JClass, JObject, JString, JValue};
use jni::sys::{jboolean, jbyteArray, jint, jlong, jstring};
use jni::{JNIEnv, JavaVM};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

static ENGINES: OnceLock<RwLock<BTreeMap<i64, Arc<BoxProxyEngine>>>> = OnceLock::new();
static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);

fn registry() -> &'static RwLock<BTreeMap<i64, Arc<BoxProxyEngine>>> {
    ENGINES.get_or_init(|| RwLock::new(BTreeMap::new()))
}

fn get_engine(handle: i64) -> Option<Arc<BoxProxyEngine>> {
    registry().read().ok()?.get(&handle).cloned()
}

struct JniPlatformBridge {
    vm: JavaVM,
    callback: GlobalRef,
}

impl PlatformBridge for JniPlatformBridge {
    fn on_effect(&self, effect: &AppEffect) {
        let Ok(bytes) = serde_json::to_vec(effect) else {
            return;
        };
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return;
        };
        if let Ok(arr) = env.byte_array_from_slice(&bytes) {
            let obj = JObject::from(arr);
            let _ = env.call_method(
                self.callback.as_obj(),
                "onEffect",
                "([B)V",
                &[JValue::Object(&obj)],
            );
        }
    }

    fn set_component_alias(&self, alias_name: &str, enabled: bool) {
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return;
        };
        if let Ok(name) = env.new_string(alias_name) {
            let obj = JObject::from(name);
            let _ = env.call_method(
                self.callback.as_obj(),
                "setComponentAlias",
                "(Ljava/lang/String;Z)V",
                &[JValue::Object(&obj), JValue::Bool(u8::from(enabled))],
            );
        }
    }

    fn query_installed_apps(&self) -> Vec<InstalledAppInfo> {
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return Vec::new();
        };
        let Ok(val) = env.call_method(self.callback.as_obj(), "queryInstalledApps", "()[B", &[])
        else {
            return Vec::new();
        };
        let Ok(obj) = val.l() else {
            return Vec::new();
        };
        if obj.is_null() {
            return Vec::new();
        }
        let arr = JByteArray::from(obj);
        env.convert_byte_array(arr)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn query_network_snapshot(&self) -> NetworkObservation {
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return NetworkObservation::default();
        };
        let Ok(val) = env.call_method(self.callback.as_obj(), "queryNetworkSnapshot", "()[B", &[])
        else {
            return NetworkObservation::default();
        };
        let Ok(obj) = val.l() else {
            return NetworkObservation::default();
        };
        if obj.is_null() {
            return NetworkObservation::default();
        }
        let arr = JByteArray::from(obj);
        env.convert_byte_array(arr)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn has_permission(&self, permission: &str) -> bool {
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return false;
        };
        let Ok(perm) = env.new_string(permission) else {
            return false;
        };
        let obj = JObject::from(perm);
        env.call_method(
            self.callback.as_obj(),
            "hasPermission",
            "(Ljava/lang/String;)Z",
            &[JValue::Object(&obj)],
        )
        .ok()
        .and_then(|v| v.z().ok())
        .unwrap_or(false)
    }

    fn read_asset_bytes(&self, asset_path: &str) -> Option<Vec<u8>> {
        let mut env = self.vm.attach_current_thread().ok()?;
        let p = env.new_string(asset_path).ok()?;
        let obj = JObject::from(p);
        let val = env
            .call_method(
                self.callback.as_obj(),
                "readAssetBytes",
                "(Ljava/lang/String;)[B",
                &[JValue::Object(&obj)],
            )
            .ok()?;
        let res_obj = val.l().ok()?;
        if res_obj.is_null() {
            return None;
        }
        let arr = JByteArray::from(res_obj);
        env.convert_byte_array(arr).ok()
    }

    fn open_external_url(&self, url: &str) {
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return;
        };
        if let Ok(u) = env.new_string(url) {
            let obj = JObject::from(u);
            let _ = env.call_method(
                self.callback.as_obj(),
                "openExternalUrl",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&obj)],
            );
        }
    }
}

struct JniStateListener {
    vm: JavaVM,
    listener: GlobalRef,
}

impl StateListener for JniStateListener {
    fn on_state_snapshot(&self, version: u64, state_json: &[u8]) {
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return;
        };
        if let Ok(arr) = env.byte_array_from_slice(state_json) {
            let obj = JObject::from(arr);
            let _ = env.call_method(
                self.listener.as_obj(),
                "onStateSnapshot",
                "(J[B)V",
                &[JValue::Long(version as i64), JValue::Object(&obj)],
            );
        }
    }
}

fn jstr_to_string(env: &mut JNIEnv<'_>, s: JString<'_>) -> String {
    env.get_string(&s).map(String::from).unwrap_or_default()
}

fn to_jbyte_array(env: &mut JNIEnv<'_>, bytes: &[u8]) -> jbyteArray {
    env.byte_array_from_slice(bytes)
        .map(|a| a.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

fn json_error_bytes(msg: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({ "ok": false, "error": msg }))
        .unwrap_or_else(|_| b"{\"ok\":false}".to_vec())
}

#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeInit<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    files_dir: JString<'local>,
    cache_dir: JString<'local>,
    package_name: JString<'local>,
    app_version: JString<'local>,
    sdk_int: jint,
    initial_locale: JString<'local>,
    callback: JObject<'local>,
) -> jlong {
    let files_dir = PathBuf::from(jstr_to_string(&mut env, files_dir));
    let cache_dir = PathBuf::from(jstr_to_string(&mut env, cache_dir));
    let package_name = jstr_to_string(&mut env, package_name);
    let app_version = jstr_to_string(&mut env, app_version);
    let initial_locale = jstr_to_string(&mut env, initial_locale);

    let Ok(vm) = env.get_java_vm() else {
        return 0;
    };
    let Ok(global_cb) = env.new_global_ref(callback) else {
        return 0;
    };
    let bridge = Arc::new(JniPlatformBridge {
        vm,
        callback: global_cb,
    });

    let cfg = EngineConfig {
        files_dir,
        cache_dir,
        package_name,
        app_version,
        sdk_int,
        initial_locale,
    };

    let Ok(engine) = BoxProxyEngine::new(cfg, bridge, None) else {
        return 0;
    };

    let handle = NEXT_HANDLE.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut map) = registry().write() {
        map.insert(handle, Arc::new(engine));
    }
    handle
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeShutdown(
    _env: JNIEnv<'_>,
    _cls: JClass<'_>,
    handle: jlong,
) {
    if let Ok(mut map) = registry().write() {
        map.remove(&handle);
    }
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeRegisterAsset<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
    asset_path: JString<'local>,
    bytes: JByteArray<'local>,
) {
    let Some(engine) = get_engine(handle) else {
        return;
    };
    let path = jstr_to_string(&mut env, asset_path);
    if let Ok(raw) = env.convert_byte_array(bytes) {
        engine.register_asset(&path, raw);
    }
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeDispatch<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
    action_bytes: JByteArray<'local>,
) -> jbyteArray {
    let Some(engine) = get_engine(handle) else {
        return to_jbyte_array(&mut env, &json_error_bytes("invalid engine handle"));
    };
    let Ok(raw) = env.convert_byte_array(action_bytes) else {
        return to_jbyte_array(&mut env, &json_error_bytes("read action bytes failed"));
    };
    match engine.dispatch_json(&raw) {
        Ok(state_json) => to_jbyte_array(&mut env, &state_json),
        Err(err) => to_jbyte_array(&mut env, &json_error_bytes(&err)),
    }
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeSubscribeView<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
    view_id: JString<'local>,
    listener: JObject<'local>,
) -> jlong {
    let Some(engine) = get_engine(handle) else {
        return 0;
    };
    let view = jstr_to_string(&mut env, view_id);
    let Ok(vm) = env.get_java_vm() else {
        return 0;
    };
    let Ok(global_listener) = env.new_global_ref(listener) else {
        return 0;
    };
    engine.subscribe(
        view,
        Arc::new(JniStateListener {
            vm,
            listener: global_listener,
        }),
    ) as jlong
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeUnsubscribeView(
    _env: JNIEnv<'_>,
    _cls: JClass<'_>,
    handle: jlong,
    subscription_id: jlong,
) {
    if let Some(engine) = get_engine(handle) {
        engine.unsubscribe(subscription_id as u64);
    }
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeQueryTileState<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
) -> jbyteArray {
    let Some(engine) = get_engine(handle) else {
        return to_jbyte_array(&mut env, &json_error_bytes("invalid engine handle"));
    };
    let snap = engine.query_tile_state();
    let bytes = serde_json::to_vec(&snap).unwrap_or_else(|_| b"{}".to_vec());
    to_jbyte_array(&mut env, &bytes)
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeRenderNotification<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
    force_refresh: jboolean,
) -> jbyteArray {
    let Some(engine) = get_engine(handle) else {
        return to_jbyte_array(&mut env, &json_error_bytes("invalid engine handle"));
    };
    let snap = engine.render_notification(force_refresh != 0);
    let bytes = serde_json::to_vec(&snap).unwrap_or_else(|_| b"{}".to_vec());
    to_jbyte_array(&mut env, &bytes)
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeResolveMonetCss<'local>(
    env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
    palette_json: JByteArray<'local>,
) -> jstring {
    let raw = env.convert_byte_array(palette_json).unwrap_or_default();
    let css = if let Some(engine) = get_engine(handle) {
        engine.resolve_monet_css(&raw)
    } else {
        String::new()
    };
    env.new_string(css)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeImportCoreFromFd<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
    target_core: JString<'local>,
    fd: jint,
    display_name: JString<'local>,
) -> jbyteArray {
    let Some(engine) = get_engine(handle) else {
        return to_jbyte_array(&mut env, &json_error_bytes("invalid engine handle"));
    };
    let core = jstr_to_string(&mut env, target_core);
    let name = jstr_to_string(&mut env, display_name);
    #[cfg(unix)]
    let res = {
        use std::os::fd::FromRawFd;
        // SAFETY: Android's ParcelFileDescriptor.detachFd() transfers fd ownership to Rust.
        let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
        let mut buf = Vec::new();
        match file.read_to_end(&mut buf) {
            Ok(_) => engine.import_core_from_bytes(&core, &buf, &name),
            Err(e) => Err(format!("read fd failed: {e}")),
        }
    };
    #[cfg(not(unix))]
    let res: Result<usize, String> = Err("fd import requires unix".to_string());

    let payload = match res {
        Ok(size) => serde_json::to_vec(&serde_json::json!({ "ok": true, "bytes": size }))
            .unwrap_or_else(|_| b"{\"ok\":true}".to_vec()),
        Err(e) => json_error_bytes(&e),
    };
    to_jbyte_array(&mut env, &payload)
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeExportBackupToFd<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
    fd: jint,
    scope_json: JByteArray<'local>,
) -> jbyteArray {
    let Some(engine) = get_engine(handle) else {
        return to_jbyte_array(&mut env, &json_error_bytes("invalid engine handle"));
    };
    let scope: BackupScope = env
        .convert_byte_array(scope_json)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();

    #[cfg(unix)]
    let res = {
        use std::os::fd::FromRawFd;
        // SAFETY: fd ownership transferred via ParcelFileDescriptor.detachFd().
        let file = unsafe { std::fs::File::from_raw_fd(fd) };
        engine.export_backup_to_writer(file, scope)
    };
    #[cfg(not(unix))]
    let res: Result<boxproxy_store::BackupManifest, String> =
        Err("fd export requires unix".to_string());

    let payload = match res {
        Ok(manifest) => serde_json::to_vec(&serde_json::json!({ "ok": true, "manifest": manifest }))
            .unwrap_or_else(|_| b"{\"ok\":true}".to_vec()),
        Err(e) => json_error_bytes(&e),
    };
    to_jbyte_array(&mut env, &payload)
}

#[no_mangle]
pub extern "system" fn Java_io_honk_android_bridge_RustBridge_nativeRestoreBackupFromFd<'local>(
    mut env: JNIEnv<'local>,
    _cls: JClass<'local>,
    handle: jlong,
    fd: jint,
    scope_json: JByteArray<'local>,
) -> jbyteArray {
    let Some(engine) = get_engine(handle) else {
        return to_jbyte_array(&mut env, &json_error_bytes("invalid engine handle"));
    };
    let scope: BackupScope = env
        .convert_byte_array(scope_json)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();

    #[cfg(unix)]
    let res = {
        use std::os::fd::FromRawFd;
        // SAFETY: fd ownership transferred via ParcelFileDescriptor.detachFd().
        let file = unsafe { std::fs::File::from_raw_fd(fd) };
        engine.restore_backup_from_reader(file, scope)
    };
    #[cfg(not(unix))]
    let res: Result<boxproxy_store::BackupManifest, String> =
        Err("fd restore requires unix".to_string());

    let payload = match res {
        Ok(manifest) => serde_json::to_vec(&serde_json::json!({ "ok": true, "manifest": manifest }))
            .unwrap_or_else(|_| b"{\"ok\":true}".to_vec()),
        Err(e) => json_error_bytes(&e),
    };
    to_jbyte_array(&mut env, &payload)
}
