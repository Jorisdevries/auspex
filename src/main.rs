use reqwest::{blocking::Client, header::{HeaderMap, CONTENT_TYPE}};
use std::env;
use std::error::Error;
use std::io::{self, Write};

struct ChatBot {
    api_key: String,
    client: Client,
    headers: HeaderMap,
    messages: Vec<Message>,
    responses: Vec<Response>,
}

impl ChatBot {
    fn new(api_key: String) -> Self {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        Self {
            api_key,
            client,
            headers,
            messages: Vec::new(),
            responses: Vec::new(),
        }
    }

    fn send_message(&mut self, role: &str, message: &str) -> reqwest::Result<()> {
        let url = "https://api.openai.com/v1/chat/completions";
        let headers = self.headers.clone();

        self.messages.push(Message { 
            role: Some(String::from(role)),
            content: (message.to_string()) 
        });

        let request = Request {
            model: String::from("gpt-3.5-turbo"),
            messages: (*self.messages).to_vec(), 
        };

        let response = self.client.post(url)
            .headers(headers)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()?
            .json::<Response>()?;

        self.responses.push(response);
        Ok(())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Message {
    role: Option<String>,
    content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Response {
    id: Option<String>,
    object: Option<String>,
    created: Option<i64>,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Choice {
    index: usize,
    message: Message,
    finish_reason: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

fn get_first_choice_content(response: &Response) -> Option<&str> {
    response.choices
        .first() // Get a reference to the first choice
        .map(|choice| choice.message.content.trim()) // Get the content string and trim whitespace
}

fn get_user_input() -> String {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}


fn main() -> Result<(), Box<dyn Error>> {
    let mut chatbot = ChatBot::new(env::var("OPENAI_API_KEY")?);
    println!("INFO: Export your API key as OPENAI_API_KEY. Enter 'q', quit' or 'exit' to quit");

    println!("> Provide a system instruction. Leave blank to skip.");
    let system_instruction = get_user_input();
    chatbot.send_message("system", system_instruction.as_str())?;

    loop {
        print!("> Enter your message: ");
        let input = get_user_input();

        if input == "q" || input == "quit" || input == "exit" {
            break;
        }

        chatbot.send_message("user", input.as_str())?;
        let response = get_first_choice_content(chatbot.responses.last().unwrap()).unwrap();
        println!("{}", response); 
    }

    Ok(())
}
