pub fn inject_builtins(src: &mut String) {
    inject_vec(src)
}

fn inject_vec(src: &mut String) {
    let vec_src = include_str!("vec.src");

    // src.push_str(vec_src);
    src.insert_str(0, vec_src); // FIXME this fucks up the line numbers
}