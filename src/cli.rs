use std::{collections::HashMap, env};

pub fn get_args_and_options() -> (Vec<String>, HashMap<String, Option<String>>) {
    let mut args = Vec::new();
    let mut options = HashMap::new();

    let raw_args = env::args().collect::<Vec<_>>();
    let l = raw_args.len();

    let mut i = 0;
    while i < l {
        let arg = raw_args[i].to_string();
        if let Some(stripped) = arg.strip_prefix("--") {
            options.insert(
                stripped.to_string(),
                raw_args.get(i + 1).map(|s| s.to_owned()),
            );
            i += 1;
        } else if let Some(stripped) = arg.strip_prefix("-") {
            options.insert(stripped.to_string(), None);
        } else {
            args.push(arg);
        }
        i += 1;
    }

    (args, options)
}
