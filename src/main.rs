use jni::{InitArgsBuilder, JavaVM, JNIVersion};
use jni::objects::{JClass, JString, JObject};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::result;

// 全局静态变量存储JVM实例
static mut JVM: Option<JavaVM> = None;

#[no_mangle]
pub extern "C" fn init_jvm() -> bool {
    unsafe {
        if JVM.is_some() {
            return true; // 已经初始化过
        }

        let args = match InitArgsBuilder::new()
            .version(JNIVersion::V8)
            .option("-Djava.class.path=.")
            .build() 
        {
            Ok(args) => args,
            Err(_) => return false,
        };

        let jvm = JavaVM::new(args).unwrap();  // 新版本
        JVM.is_some()
    }
}

#[no_mangle]
pub extern "C" fn decrypt_password(password: *const c_char) -> *mut c_char {
    unsafe {
        if JVM.is_none() && !init_jvm() {
            return std::ptr::null_mut();
        }

        let env = match JVM.as_ref().unwrap().attach_current_thread() {
            Ok(env) => env,
            Err(_) => return std::ptr::null_mut(),
        };

        // 将C字符串转换为Rust字符串
        let c_str = unsafe { CStr::from_ptr(password) };
        let password_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        // 调用Java方法
        let result = match env.new_string(password_str) {
            Ok(jstr) => {
                let class = match env.find_class("HelloWorld") {
                    Ok(c) => c,
                    Err(_) => {
                        env.delete_local_ref(jstr).ok();
                        return std::ptr::null_mut();
                    }
                };

                let res = env.call_static_method(
                    class,
                    "decrypt",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    &[jni::objects::JValue::from(&jstr)],
                );

                env.delete_local_ref(jstr).ok();

                match res {
                    Ok(val) => match val.l() {
                        Ok(obj) => Ok(obj),
                        Err(_) => return std::ptr::null_mut(),
                    },
                    Err(_) => return std::ptr::null_mut(),
                }
            }
            Err(_) => return std::ptr::null_mut(),
        };

        // 处理返回结果
        match result {
            Ok(obj) => {
                let jstr = JString::from(obj);
                let rust_str = match env.get_string(&jstr) {
                    Ok(s) => s,
                    Err(_) => {
                        env.delete_local_ref(obj).ok();
                        return std::ptr::null_mut();
                    }
                };

                let c_str = match CString::new(rust_str.to_string_lossy().into_owned()) {
                    Ok(s) => s,
                    Err(_) => {
                        env.delete_local_ref(obj).ok();
                        return std::ptr::null_mut();
                    }
                };

                env.delete_local_ref(obj).ok();
                c_str.into_raw()
            }
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            let _ = CString::from_raw(s);
        }
    }
}