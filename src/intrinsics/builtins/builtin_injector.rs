pub fn inject_builtins(src: &mut String) {
    src.push('\n');
    inject_vec(src);
    src.push('\n');
    inject_direction(src);
    src.push('\n');
    inject_default_panic(src);
}

fn inject_vec(src: &mut String) {
    let vec_src = include_str!("vec.src");

    src.push_str(vec_src);
}

fn inject_direction(src: &mut String) {
    let dir_str = include_str!("direction.src");

    src.push_str(dir_str);
}

fn inject_default_panic(src: &mut String) {
    src.push_str(r#"
        fn panic() {
            panic("Explicit panic!"); 
        }
    "#);   
}