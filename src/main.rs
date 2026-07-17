use std::{env, io};
use std::fs::File;
use std::io::Write;
use std::fs;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use std::thread;

mod error;
use error::error_msg;
mod build_file;
use build_file::{build_bolt_file_basic,build_bolt_file_resourcepack,build_bolt_file_version,build_bolt_file_all};
/*
This should be running the command like

cod Build "NAME OF FILE" --optional_flags



possile for doc creating

cod document "NAME OF FILE" --optional_flags


optional_flags are for
creating new arguments for specific file types or project types!

Honestly the setup is actually so dope, im tweaking tf out.

*/

const ANSI_ESCAPE: &str = "\x1b[0m";
const ANSI_WHITE: &str = "\x1b[0;97m";
const ANSI_GRAY: &str = "\x1b[0;38;5;8m";
const ANSI_GREEN: &str = "\x1b[0;92m";

/*
Main function for calling, grabbing and processing the CLI commands,
main things for this are the args being used and the checks for errors.

*/
fn main() {
    let args: Vec<String> = env::args().collect();

    //dbg!(&args);


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
        if !is_python_installed() {
                warning_message(1);
                return;
            }
        create_virtual_env();
        
        if !is_beet_installed(){
            install_dependency("beet");
        }
        if !is_bolt_installed(){
            install_dependency("bolt");
        }

    }
    else if &args[1].to_lowercase() == "doc" {

        if let Err(e) = run_doc_generation(Path::new("./src/data")) {
            eprintln!("Documentation generation failed: {}", e);
        }
    }


    else{
        helper(1)
    }
    }else {
        helper(1)
    }
    
}






/*

Below is the doc command being used
Tried my best to make it really fast for them big projects.


*/





fn create_doc(path:&Path) -> io::Result<()>{
    let bolt = fs::read_to_string(path)?;
    let markdown = parse_bolt_to_md(&bolt);

    handle(fs::write(path.with_extension("md"), markdown),21)?;

    Ok(())
}

// Chat, this parses that bolt code into md, just by making a few changes for now!
// Post update, ts is faster than vscode can render :o
struct BoltDocParser {
    output: String,
    in_code: bool,
}

//Pov 
impl BoltDocParser {
    fn new(capacity: usize) -> Self {
        Self {
            // Gives allocated memory so I don't have any slowdowns for big projects!
            output: String::with_capacity(capacity + 512),
            in_code: false,
        }
    }

    fn open_code(&mut self) {
        if !self.in_code {
            self.output.push_str("```python\n");
            self.in_code = true;
        }
    }

    fn close_code(&mut self) {
        if self.in_code {
            self.output.push_str("```\n\n");
            self.in_code = false;
        }
    }

    // Lets lines that are commented return as .md formatting!
    fn parse_line(&mut self, line: &str) {
        let trimmed = line.trim_start();

        match trimmed {
            s if s.starts_with("#h1") => {
                self.close_code();
                self.output.push_str("#");
                self.output.push_str(s.strip_prefix("#h1").unwrap());
                self.output.push_str("  \n");
            }
            s if s.starts_with("#h2") => {
                self.close_code();
                self.output.push_str("##");
                self.output.push_str(s.strip_prefix("#h2").unwrap());
                self.output.push_str("  \n");
            }
            s if s.starts_with("#h3") => {
                self.close_code();
                self.output.push_str("###");
                self.output.push_str(s.strip_prefix("#h3").unwrap());
                self.output.push_str("  \n");
            }
            s if s.starts_with("\\#") => {
                self.open_code();
                self.output.push_str(&s[1..]);
                self.output.push_str("  \n");
            }
            s if s.starts_with('#') => {
                self.close_code();
                self.output.push_str(s.strip_prefix('#').unwrap().trim_start());
                self.output.push_str("  \n");
            }
            _ => {
                self.open_code();
                self.output.push_str(line);
                self.output.push_str("  \n");
            }
        }
    }

    fn finish(mut self) -> String {
        self.close_code();
        self.output
    }
}

//This just uses that struct to parse through each line.
fn parse_bolt_to_md(source: &str) -> String {
    let mut parser = BoltDocParser::new(source.len());

    for line in source.lines() {
        parser.parse_line(line);
    }

    parser.finish()
}


//This is a godsend, does an reading for every .bolt file in dir
fn find_bolt_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    // Get the folder name to check if we should skip it cuz it makes it like 1 micro second faster
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip hidden folders (.git, .venv), rust build targets, and node_modules
                        if name.starts_with('.') 
                            || name == "target" 
                            || name == "node_modules" 
                            || name == "venv" 
                        {
                            
                            continue; 
                        }
                    }
                    stack.push(path);
                } else if path.extension().is_some_and(|ext| ext == "bolt") {
                    paths.push(path);
                }
            }
        }
    }
    Ok(paths)
}


//Learned how to use multithreading in rust for this one
//
pub fn run_doc_generation(dir: &Path) -> io::Result<()> {
    let paths = find_bolt_files(dir)?;
    if paths.is_empty() {
        warning_message(5);
        return Ok(());
    }

    // Making multiple threads, and if failed, just make it 4
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    // standard library cmp came in clutch here, but only here :p
    let chunk_size = std::cmp::max(1, (paths.len() + num_threads - 1) / num_threads);

    // Scoped threads let this run work in parallel using references!!
    thread::scope(|s| {
        for chunk in paths.chunks(chunk_size) {
            s.spawn(move || {
                for path in chunk {
                    println!("{}", path.display());
                    if let Err(e) = create_doc(path) {
                        eprintln!("Failed to document {}: {}", path.display(), e);
                    }
                }
            });
        }
    });
    helper(5);
    Ok(())
}



fn helper(id:u8){

    if id == 1 {
        println!("
{ANSI_WHITE}Cod Commands:
{ANSI_GREEN}  cod build {ANSI_GRAY} //Used for building a bolt/beet project.
{ANSI_GREEN}  cod setup {ANSI_GRAY} //Used for installing bolt/beet in a python virtual environment.
{ANSI_GREEN}  cod doc {ANSI_GRAY} //Used to auto document code in a bolt/beet project.
{ANSI_ESCAPE}\n")
    }
    if id == 2 {
        println!("
{ANSI_WHITE}Cod Build Commands:\n
{ANSI_GREEN}  cod build bolt{ANSI_ESCAPE}
{ANSI_GREEN}            ++++\n")
    }
    if id == 3 {
        println!("
{ANSI_WHITE}Cod Build Commands:\n
{ANSI_GREEN}  cod build bolt \"Project name\" \"Description of project\"{ANSI_ESCAPE}
{ANSI_GREEN}                 +++++++++++++++++++++++++++++++++++++++\n")
    }
        if id == 4 {
        println!("
{ANSI_WHITE}USE THIS COMMAND:
{ANSI_GREEN}  cod setup {ANSI_GRAY}//Used for installing bolt/beet in a python virtual environment.{ANSI_ESCAPE}
")
    }
    if id == 5 {
        println!("
{ANSI_WHITE}Finished Cod Doc:
{ANSI_GRAY}     *Vscode or intellij will take a bit to update the space.{ANSI_ESCAPE}
")
    }
    if id == 6 {
        println!("
{ANSI_WHITE}Type number for settings:
{ANSI_GREEN}[0]  {ANSI_GRAY}//Skip This Step{ANSI_ESCAPE}
{ANSI_GREEN}[1]  {ANSI_GRAY}//Create with template resourcepack{ANSI_ESCAPE}
{ANSI_GREEN}[2]  {ANSI_GRAY}//Create version controlled datapack{ANSI_ESCAPE}
{ANSI_GREEN}[3]  {ANSI_GRAY}//All the above{ANSI_ESCAPE}
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


/*
Error messages and warning stuff are the same in my book

1-19 is user generated errors, can be fixed
20-29 is software generated errors, just reload or restart
40-59 is errors that could break the program, repeatable and broken

*/


fn warning_message(id:u8){
    // ERROR MESSAGE FOR Python INSTALL
    if id == 1{

        error_msg::new("Fatal Python Error")
        .description("Python is not installed.")
        .number(1)
        .helper("Visit this site to download python: https://www.python.org/downloads/\nOr on linux install via command: sudo apt install python3")
        .print();
        
    }
    // ERROR MESSAGE FOR Beet INSTALL
    if id == 2{
        error_msg::new("Fatal Beet Error")
        .description("Beet is not installed.")
        .number(2)
        .helper("Visit this site to install beet:  https://mcbeet.dev/quick-start/get-started/#installation\nOr on virtual environment install via command: pip install beet")
        .print();
    }
    // ERROR MESSAGE FOR Bolt INSTALL
    if id == 3{
        error_msg::new("Fatal Bolt Error")
        .description("Beet is not installed.")
        .number(3)
        .helper("On virtual environment install via command: pip install bolt")
        .print();
    }

    if id == 4{
        error_msg::new("Settings Error")
        .description("Typed an incorrect value for cod build settings.\nDefaulting to generic template.")
        .number(4)
        .print();
    }

    if id == 5{
        error_msg::new("Documentation Error")
        .description("There is no .bolt files to document.")
        .number(5)
        .print();
    }
    if id == 6{
        error_msg::new("Overwrite Error")
        .description("Ran cod build twice and it can have a chance of overwriting data where is should not be.\nDouble check beet.json for errors.")
        .number(6)
        .print();
    }
    if id == 21{
        error_msg::new("Fatal Create File Error")
        .description("This error .")
        .number(21)
        .print();
    }
    if id == 2 || id == 3 {
        helper(4)
    }
}



fn handle<T>(result: io::Result<T>, id: u8) -> io::Result<T>{
    result.map_err(|e| {
        warning_message(id);
        e
    })
}











fn build_bolt_project(name : &str, description : &str) -> std::io::Result<()>{

    //This is for grabbing more data when building the project
    let mut build_id = String::new();


    helper(6);
    io::stdin().read_line(&mut build_id).expect("Failed to read line");
    

    let asset_dir = "src/assets/minecraft/textures";
    let project_dir = format!("src/data/{}/modules",name.to_lowercase().replace(" ", "_"));
    fs::create_dir_all(project_dir)?;

    let main_bolt = format!("src/data/{}/modules/main.bolt",name.to_lowercase().replace(" ", "_"));

    //This is the beet json file
    let mut beet_json = File::options()
        .append(false)
        .create(true)
        .write(true)
        .open("beet.json")?;

    let mut demo_bolt = File::options()
        .append(false)
        .create(true)
        .write(true)
        .open(main_bolt)?;
    

    let json = match *&build_id.to_string().as_str().trim(){
        "0" => {
            build_bolt_file_basic(name,description)
        }

        "1" => {
            fs::create_dir_all(asset_dir)?;
            build_bolt_file_resourcepack(name,description)

        }
        "2" => {
            println!("Input Pack Format: ");
            let mut version: String = String::new();
            io::stdin().read_line(&mut version).expect("Failed to read line");
            build_bolt_file_version(name,description,&version.to_string().as_str().trim())
        }
        "3" => {
            println!("Input Pack Format: ");
            let mut version: String = String::new();
            io::stdin().read_line(&mut version).expect("Failed to read line");
            fs::create_dir_all(asset_dir)?;
            build_bolt_file_all(name,description,&version.to_string().as_str().trim())
        }
        _ => {
            warning_message(4);
            build_bolt_file_basic("template","template description")
        }
    };
    
    if !is_beet_installed() || !is_bolt_installed() {
                        debugger();
    }

    if Path::new("beet.json").is_file() {
        warning_message(6);
    }

    handle(writeln!(&mut beet_json, "{}",json),21)?;
    handle(writeln!(&mut demo_bolt, "function template:main:\n  say Hello World"),21)?;
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