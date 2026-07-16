use std::{env, io};
use std::fs::File;
use std::io::Write;
use std::fs;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use std::thread;
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
                    if !is_beet_installed() || !is_bolt_installed() {
                        debugger();
                    }
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
        helper(5);
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

    fs::write(path.with_extension("md"), markdown)?;

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

    Ok(())
}



fn helper(id:u8){

    if id == 1 {
        println!("\n
{ANSI_WHITE}Cod Commands:
{ANSI_GREEN}  cod build {ANSI_GRAY} //Used for building a bolt/beet project.
{ANSI_GREEN}  cod setup {ANSI_GRAY} //Used for installing bolt/beet in a python virtual environment.
{ANSI_GREEN}  cod doc {ANSI_GRAY} //Used to auto document code in a bolt/beet project.
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
    if id == 5 {
        println!("\n
{ANSI_WHITE}Finished Cod Doc:
{ANSI_GRAY}     *Vscode or intellij will take a bit to update the space.{ANSI_ESCAPE}
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

    //This is for grabbing more data when building the project
    let mut input = String::new();



    io::stdin().read_line(&mut input).expect("Failed to read line");



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