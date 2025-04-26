use jni::{InitArgsBuilder, JavaVM, JNIVersion};
use jni::objects::{JClass, JString, JValue};
use jni::sys::jint;

fn main() -> Result<(), Box<dyn std::error::Error>> {
   // 1. 初始化 JVM 参数
   let args = InitArgsBuilder::new()
   .version(JNIVersion::V8)
   .option("-Djava.class.path=.")
   .build()?;

// 2. 创建 Java 虚拟机实例（兼容写法）
    let jvm = JavaVM::new(args)?;  // 新版本

    let mut env = jvm.attach_current_thread()?;

    // 3. 调用静态方法 printHello
    let class_hello = env.find_class("HelloWorld")?;
    // env.call_static_method(
    //     &class_hello,
    //     "printHello",
    //     "()V",
    //     &[],
    // )?;

    // // 4. 调用静态方法 addNumbers 并获取返回值
    // let result = env.call_static_method(
    //     &class_hello,
    //     "addNumbers",
    //     "(II)I",
    //     &[
    //         JValue::Int(5),
    //         JValue::Int(7),
    //     ],
    // )?;

    // let sum: jint = result.i()?;
    // println!("Java计算 5 + 7 = {}", sum);

   // 2. 准备参数
   let password = "myPassword";
   let jstr_password = env.new_string(password)?;

   // 3. 调用 decrypt 方法
   let class_hello = env.find_class("HelloWorld")?;
   let result = env.call_static_method(
       class_hello,
       "decrypt",
       "(Ljava/lang/String;)Ljava/lang/String;",
       &[jni::objects::JValue::from(&jstr_password)],
   )?;

   // 4. 处理返回的Java字符串（关键修正部分）
   let j_return_obj = result.l()?; // 获取返回的JObject
   let j_return_string = JString::from(j_return_obj); // 显式转换为JString
   let rust_string: String = env.get_string(&j_return_string)?.into();

   println!("Java返回的字符串: {}", rust_string);

   // 5. 释放本地引用
   env.delete_local_ref(jstr_password)?;
//    env.delete_local_ref(j_return_obj)?;

    Ok(())
}