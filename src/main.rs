use std::env;


fn main() {
    let args: Vec<String> = env::args().collect();
    let builder = &args[1];
    //dbg!(args);
    if &args[1].to_lowercase() == "build" {
        println!("YES")
    }
    println!("{}",builder)
}
