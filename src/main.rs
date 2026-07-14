use std::env;
use std::fs::File;
use std::io::Write;


/*
This should be running the command like

cod Build "NAME OF FILE" --optional_flags



possile for doc creating

cod document "NAME OF FILE" --optional_flags


optional_flags are for
creating new arguments for specific file types or project types!

*/


fn main() {
    let args: Vec<String> = env::args().collect();

    //dbg!(args);

    if args.len() > 1 {
        // This is the main build command. 
    if &args[1].to_lowercase() == "build" {
        if args.len() > 2 {
            match args.get(2).map(|s| s.as_str()) {
            Some("bolt") => {
                if args.len() > 4 {
                    let _ = build_bolt_project(&args[3],&args[4]);
                } else {
                    helper(2);
                }
            },
            _ => helper(2)
        }
        }else{
            helper(2)
        }
        
        
    }else{
        helper(1)
    }
    }else{
        helper(1)
    }
    
}



fn helper(id:u8){
    let ansi_error: &str = "\x1b[1;31m";
    let ansi_escape: &str = "\x1b[0m";
    let ansi_white: &str = "\x1b[0;97m";
    if id == 1 {
        println!("\n{}Cod Commands:\n{}  cod build{}\n",ansi_white,ansi_error,ansi_escape)
    }
    if id == 2 {
        println!("\n{}Cod Build Commands:\n{}  cod build bolt \"String project name\" \"Description of project\"{}\n",ansi_white,ansi_error,ansi_escape)
    }

}






fn build_bolt_project(name : &str, description : &str) -> std::io::Result<()>{
    //This is the beet json file
    let mut beet_json = File::options()
        .create(true)
        .write(true)
        .open("beet.json")?;

    let starting_json:String  = format!("{{\n \"name\":\"{}\",\n  \"description\":\"{}\",",name,description);
    let ending_json: &str = r#"

    "require": [
        "bolt"
    ],

    "data_pack":{
        "load": ["src"]
    },
    
    "pipeline": [
        "mecha"
    ],
    


    "output": "build",

    "meta":{
        "bolt":{
            "entrypoint":["example:main"]
        }
    }
    
}"#;
    
    writeln!(&mut beet_json, "{}{}",starting_json,ending_json)?;
    Ok(())
}