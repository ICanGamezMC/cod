use std::env;
use std::fs::File;
use std::io::Write;
use std::fs;
use std::process::{Command, Stdio};
use std::path::Path;
/*
This should be running the command like

cod Build "NAME OF FILE" --optional_flags



possile for doc creating

cod document "NAME OF FILE" --optional_flags


optional_flags are for
creating new arguments for specific file types or project types!

Honestly the setup is actually so dope, im tweaking tf out.

*/

const ANSI_ERROR: &str = "\x1b[1;31m";
const ANSI_ESCAPE: &str = "\x1b[0m";
const ANSI_WHITE: &str = "\x1b[0;97m";
const ANSI_GRAY: &str = "\x1b[0;38;5;8m";
const ANSI_YELLOW_UNDERLINE: &str = "\x1b[4;93m";
const ANSI_GREEN: &str = "\x1b[0;92m";


fn main() {
    let args: Vec<String> = env::args().collect();

    debugger();
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
                    helper(3);
                }
            },
            _ => helper(3)
        }
        }else{
            helper(2)
        }
    }else if &args[1].to_lowercase() == "setup" {
        create_virtual_env();
        
        if !is_beet_installed(){
            install_dependency("beet");
        }
        if !is_bolt_installed(){
            install_dependency("bolt");
        }

    }
    else{
        helper(1)
    }
    }else{
        helper(1)
    }
    
}



fn helper(id:u8){

    if id == 1 {
        println!("\n
{ANSI_WHITE}Cod Commands:
{ANSI_GREEN}  cod build {ANSI_GRAY} //Used for building a bolt/beet project.
{ANSI_GREEN}  cod setup {ANSI_GRAY} //Used for installing bolt/beet in a python virtual environment.
{ANSI_ESCAPE}\n")
    }
    if id == 2 {
        println!("\n
{ANSI_WHITE}Cod Build Commands:\n
{ANSI_GREEN}  cod build bolt{ANSI_ESCAPE}
{ANSI_GREEN}            ++++\n")
    }
    if id == 3 {
        println!("\n
{ANSI_WHITE}Cod Build Commands:\n
{ANSI_GREEN}  cod build bolt \"Project name\" \"Description of project\"{ANSI_ESCAPE}
{ANSI_GREEN}                 +++++++++++++++++++++++++++++++++++++++\n")
    }
        if id == 4 {
        println!("\n
{ANSI_WHITE}USE THIS COMMAND:
{ANSI_GREEN}  cod setup {ANSI_GRAY}//Used for installing bolt/beet in a python virtual environment.{ANSI_ESCAPE}
")
    }

}



fn debugger() {
    if is_python_installed() {
        println!("{}Python installed{}",ANSI_GRAY,ANSI_ESCAPE)
    } else{
        warning_message(1);
    }

    if is_beet_installed() {
        println!("{}Beet installed{}",ANSI_GRAY,ANSI_ESCAPE)
    } else{
        warning_message(2)
    }
    if is_bolt_installed() {
        println!("{}Bolt installed{}",ANSI_GRAY,ANSI_ESCAPE)
    } else{
        warning_message(3)
    }
}


fn warning_message(id:u8){
    // ERROR MESSAGE FOR Python INSTALL
    if id == 1{
        println!(
        "\n{ANSI_ERROR}WARNING{ANSI_WHITE}: fatal python error
    {ANSI_ERROR}Python is not installed.{ANSI_ESCAPE}
    {ANSI_GRAY}Visit this site to download python: {ANSI_YELLOW_UNDERLINE}https://www.python.org/downloads/{ANSI_ESCAPE}
    {ANSI_GRAY}Or on linux install via command: {ANSI_YELLOW_UNDERLINE}sudo apt install python3{ANSI_ESCAPE} \n
        "
    )
    }
    // ERROR MESSAGE FOR Beet INSTALL
    if id == 2{
        println!(
        "\n{ANSI_ERROR}WARNING{ANSI_WHITE}: fatal beet error
    {ANSI_ERROR}Beet is not installed.{ANSI_ESCAPE}
    {ANSI_GRAY}Visit this site to install beet: {ANSI_YELLOW_UNDERLINE}https://mcbeet.dev/quick-start/get-started/#installation{ANSI_ESCAPE}
    {ANSI_GRAY}Or on virtual environment install via command: {ANSI_YELLOW_UNDERLINE}pip install beet{ANSI_ESCAPE} \n
        "
    )
    }
    // ERROR MESSAGE FOR Bolt INSTALL
    if id == 3{
        println!(
        "\n{ANSI_ERROR}WARNING{ANSI_WHITE}: fatal bolt error
    {ANSI_ERROR}Bolt is not installed.{ANSI_ESCAPE}
    {ANSI_GRAY}On virtual environment install via command: {ANSI_YELLOW_UNDERLINE}pip install bolt{ANSI_ESCAPE} \n
        "
    )
    }

    if id == 3 || id == 4 {
        helper(4)
    }
}




fn build_bolt_project(name : &str, description : &str) -> std::io::Result<()>{

    let project_dir = format!("src/data/{}/modules",name.to_lowercase().replace(" ", "_"));
    fs::create_dir_all(project_dir)?;

    let main_bolt = format!("src/data/{}/modules/main.bolt",name.to_lowercase().replace(" ", "_"));

    //This is the beet json file
    let mut beet_json = File::options()
        .create(true)
        .write(true)
        .open("beet.json")?;

    let mut demo_bolt = File::options()
        .create(true)
        .write(true)
        .open(main_bolt)?;

    let starting_json:String  = format!("{{\n \"name\":\"{}\",\n  \"description\":\"{}\",",name,description);
    let middle_json: &str = r#"

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
        "#;

    let ending_json:String  = format!("        \"bolt\":{{\n            \"entrypoint\":[\"{}:main\"]\n        }}\n    }}\n}}",name.to_lowercase().replace(" ", "_"));

    
    writeln!(&mut beet_json, "{}{}{}",starting_json,middle_json,ending_json)?;
    writeln!(&mut demo_bolt, "function template:main:\n  say Hello World")?;
    Ok(())
}



fn is_python_installed() -> bool {
    
    for cmd in &["python3","python"] {
        let status = Command::new(cmd)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if let Ok(exit_status) = status {
            if exit_status.success(){
                return true;
            }
        }
    }
    false

}

fn is_beet_installed() -> bool {
    is_package_installed("beet")
}

fn is_bolt_installed() -> bool {
    is_package_installed("bolt")
}

fn create_virtual_env(){
    let python_exe = if cfg!(target_os = "windows"){
        "python"
    }else{
        "python3"
    };
    let mut cmd = Command::new(python_exe);
    cmd.arg("-m").arg("venv").arg(".venv");

    match cmd.status() {
        Ok(status) if status.success() => {
            println!("Venv created")
        }
        Ok(status) =>{
            eprintln!("Error {status}")
        }
        Err(e) => {
            eprintln!("Error {e}")
        }

    }
}


fn get_pip_path() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        ".venv\\Scripts\\pip.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        ".venv/bin/pip"
    }
}

fn get_python_path() -> &'static str {
    #[cfg(target_os = "windows")] 
    { 
        ".venv\\Scripts\\python.exe" 
    }
    #[cfg(not(target_os = "windows"))] 
    {
         ".venv/bin/python" 
    }
}

//Pov the caffien kicked in here

fn is_package_installed(package_name: &str) -> bool {
    let python_path = get_python_path();

    // If the .venv doesn't even exist yet, nothing is installed
    if !Path::new(python_path).exists() {
        return false;
    }

    // Run the local python to see if we can import the package
    let status = Command::new(python_path)
        .args(["-c", &format!("import {}", package_name)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(exit_status) => exit_status.success(),
        Err(_) => false,
    }
}

fn install_dependency(package_name: &str) -> bool{

   let pip_path = get_pip_path();

   println!("Installing {}...",package_name);

   let status = Command::new(pip_path)
        .args(&["install", package_name])
        .status()
        .expect("Failed to execute pip process");

    status.success()
}