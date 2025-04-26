要将 Rust 代码提取为可供 C 调用的函数，我们需要使用 #[no_mangle] 和 extern "C" 来确保函数符合 C 的调用约定。以下是完整的实现方案：

1. 修改后的 Rust 代码 (src/lib.rs)
   rust
   use jni::{InitArgsBuilder, JavaVM, JNIVersion};
   use jni::objects::{JClass, JString, JObject};
   use std::ffi::{CStr, CString};
   use std::os::raw::c_char;

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

        JVM = match JavaVM::builder().from_init_args(args).and_then(|b| b.build()) {
            Ok(jvm) => Some(jvm),
            Err(_) => None,
        };

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
                    Ok(val) => val.l(),
                    Err(_) => Err(jni::errors::Error::MethodNotFound),
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
2. 需要修改的 Cargo.toml
   toml
   [lib]
   name = "jni_decrypt"
   crate-type = ["cdylib"]  # 生成C兼容的动态库

[dependencies]
jni = { version = "0.21", features = ["invocation"] }
libc = "0.2"
3. C 语言调用示例 (main.c)
   c
   #include <stdio.h>
   #include <stdlib.h>

// 声明Rust函数
extern bool init_jvm(void);
extern char* decrypt_password(const char* password);
extern void free_string(char* s);

int main() {
if (!init_jvm()) {
printf("Failed to initialize JVM\n");
return 1;
}

    const char* password = "secret";
    char* result = decrypt_password(password);
    
    if (result) {
        printf("Decrypted: %s\n", result);
        free_string(result);
    } else {
        printf("Decryption failed\n");
    }

    return 0;
}
4. 编译和运行步骤
   编译 Rust 库：

bash
cargo build --release
编译 C 程序 (Linux/macOS)：

bash
gcc main.c -o main -L./target/release -ljni_decrypt -Wl,-rpath,./target/release
运行：

bash
./main
关键设计要点
全局 JVM 管理：

使用 static mut 存储 JVM 实例

提供初始化函数 init_jvm()

内存安全：

使用 CString 管理跨语言字符串

提供 free_string() 释放内存

错误处理：

所有可能失败的操作都有错误检查

返回空指针表示失败

线程安全：

每次调用都重新 attach 线程

正确处理 JNI 环境

资源清理：

及时删除本地引用

防止内存泄漏

这个实现提供了完整的 C 语言接口，可以安全地在 C 程序中调用 Java 的解密功能。