use std::env;

pub(crate) fn get_args() -> Vec<String> {
    let args: Vec<String> = env::args().skip(1).collect();
    return args;
}
