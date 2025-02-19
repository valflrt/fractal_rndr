use std::env;

pub fn get_args_and_options() -> (Vec<String>, Vec<(String, Option<String>)>) {
    let mut args = Vec::new();
    let mut options = Vec::new();

    let raw_args = env::args().collect::<Vec<_>>();
    let l = raw_args.len();

    let mut i = 0;
    while i < l {
        let arg = raw_args[i].to_owned();
        if arg.starts_with("--") {
            options.push((arg, raw_args.get(i + 1).map(|s| s.to_owned())));
            i += 1;
        } else if arg.starts_with("-") {
            options.push((arg, None));
        } else {
            args.push(arg);
        }
        i += 1;
    }

    (args, options)
}
