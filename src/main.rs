use std::env;


fn main() {
    let args: Vec<String> = env::args().collect();

    //dbg!(args);

    if args.len() > 1 {
        // This is the main build command. 
    if &args[1].to_lowercase() == "build" {
        if args.len() > 2 {
            match args[2].as_str() {
            "test" => println!("Nah"),
            _ => println!("ok")
        }
        }
        
        helper(1)
    }

    }
    
}



fn helper(id:u8){

    if id == 1 {
        println!("This is a helper")
    }

}