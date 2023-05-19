use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};


#[no_mangle]
pub extern "system" fn Java_frequency_Native_hello<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JString<'local>,
) -> JString<'local> {
    let input: String = env
        .get_string(&input)
        .expect("Couldn't get java string!")
        .into();

    let output = env
        .new_string(format!("Hello, {}!", input))
        .expect("Couldn't create java string!");
    output
}


/// An optimization barrier / guard against garbage collection.
///
/// cbindgen:ignore
#[no_mangle]
pub extern "system" fn Java_frequency_Native_keepAlive<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
    _input: JObject<'local>,
) {
}
