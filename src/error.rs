



const ANSI_BOLD_RED: &str = "\x1b[1;91;49m";
const ANSI_END: &str = "\x1b[0m";
const ANSI_WHITE: &str = "\x1b[0;97;49m";
const ANSI_YELLOW: &str = "\x1b[0;93;49m";
const ANSI_GRAY: &str = "\x1b[1;2;90;49m";



/// test
/// 
/// testing 
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct error_msg {
    message_type: String,
    message_description: String,
    helper: String,
    error_number: u32,
}


impl error_msg {

    pub fn new(message_type: &str)   -> Self {
        Self {
        message_type: message_type.to_string(),
        message_description: "".to_string(),
        error_number: 1,
        helper: "".to_string(),
        } 
    }

    pub fn number(mut self, num: u32) -> Self{
        self.error_number = num;
        self
    }

    pub fn description(mut self, description: &str) -> Self{
        self.message_description = description.to_string();
        self
    }

    pub fn helper(mut self, helper: &str) -> Self{
        self.helper = helper.to_string();
        self
    }

    pub fn print(&self) -> &u32{
        let message_type: &String = &self.message_type;
        let message_description: &String = &self.message_description;
        let error_number: &u32 = &self.error_number;
        let helper: &String = &self.helper;

        let mut error_message: String = format!("
{ANSI_GRAY}=============================================================={ANSI_END}
{ANSI_BOLD_RED}         [{message_type}] ERROR CODE : {error_number}{ANSI_END}
{ANSI_GRAY}--------------------------------------------------------------{ANSI_END}
{ANSI_WHITE}Details:  {message_description}{ANSI_END}
");

        if helper != ""{
            let helper: String = format!("
For more help on {ANSI_BOLD_RED}Error {error_number}{ANSI_END}:
{ANSI_YELLOW}{helper}{ANSI_END}
{ANSI_GRAY}=============================================================={ANSI_END}
");
            error_message.push_str(&helper.to_string());
        } else{
            error_message.push_str(&format!("{ANSI_GRAY}=============================================================={ANSI_END}").to_string())
        }

        println!("{error_message}\n");
        return error_number;
    }
}

