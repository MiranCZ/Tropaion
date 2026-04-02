pub fn inject_builtins(src: &mut String) {
    inject_vec(src);
    inject_direction(src);
}

fn inject_vec(src: &mut String) {
    let vec_src = include_str!("vec.src");

    src.push_str(vec_src);
}

fn inject_direction(src: &mut String) {
    let dir_str = include_str!("direction.src");

    src.push_str(dir_str);
}