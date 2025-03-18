use std::{collections::HashMap, env};

pub fn parse() -> (Vec<String>, HashMap<String, Option<String>>) {
    let raw_args = env::args().collect::<Vec<_>>();
    let l = raw_args.len();

    let mut args = Vec::new();
    let mut options = HashMap::new();

    let mut i = 0;
    while i < l {
        let arg = raw_args[i].to_string();
        if let Some(stripped) = arg.strip_prefix("--") {
            let param = raw_args
                .get(i + 1)
                .map(|s| s.to_owned())
                .filter(|s| !s.starts_with("-"));

            if param.is_some() {
                i += 1;
            }
            options.insert(stripped.to_string(), param);
        } else if let Some(stripped) = arg.strip_prefix("-") {
            options.insert(stripped.to_string(), None);
        } else {
            args.push(arg);
        }
        i += 1;
    }

    (args, options)
}
