use std::env;

mod env_config;

fn main() {
    env::vars()
        .into_iter()
        .for_each(|t| {
            println!("{:?}", t)
        });
    println!("Hello, world!");
}
