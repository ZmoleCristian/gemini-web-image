pub const GOOGLE: &str = "https://www.google.com";

pub const ROTATE: &str = "https://accounts.google.com/RotateCookies";

pub fn app(authuser: u32) -> String {
    format!("https://gemini.google.com/u/{authuser}/app")
}

pub fn generate(authuser: u32) -> String {
    format!("https://gemini.google.com/u/{authuser}/_/BardChatUi/data/assistant.lamda.BardFrontendService/StreamGenerate")
}

pub fn batchexecute(authuser: u32) -> String {
    format!("https://gemini.google.com/u/{authuser}/_/BardChatUi/data/batchexecute")
}
