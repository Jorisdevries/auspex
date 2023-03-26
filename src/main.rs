use colored::Colorize;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, CONTENT_TYPE},
};
use std::env;
use std::io::{self, Write};
use std::time::Duration;

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

struct ChatBot {
    api_key: String,
    client: Client,
    headers: HeaderMap,
    n_retries: u32,
    retry_delay: Duration,
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
            n_retries: 3,
            retry_delay: Duration::from_secs(1),
            messages: Vec::new(),
            responses: Vec::new(),
        }
    }

    fn send_message(&mut self, role: &str, message: &str) -> reqwest::Result<()> {
        let url = "https://api.openai.com/v1/chat/completions";
        let headers = self.headers.clone();

        // add the message to the conversation
        self.messages.push(Message {
            role: Some(String::from(role)),
            content: (message.to_string()),
        });

        // send a request and get a response
        let request = Request {
            model: String::from("gpt-3.5-turbo"),
            messages: (*self.messages).to_vec(),
        };

        let response = self
            .client
            .post(url)
            .headers(headers)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()?
            .json::<Response>()?;


        // add the assistant response to the conversation
        let response_content = response
            .choices
            .first() 
            .map(|choice| choice.message.content
            .trim()
            .trim_end_matches('\n')); 

        self.messages.push(Message {
            role: Some(String::from("assistant")),
            content: (response_content.unwrap().to_string()),
        });

        self.responses.push(response);

        Ok(())
    }

    fn retry_send_message(&mut self, role: &str, message: &str) -> reqwest::Result<()> {
        for i in 0..=self.n_retries {
            match self.send_message(role, message) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if i == self.n_retries {
                        panic!("Failed after {} retries with error: {}", self.n_retries, e);
                    } else {
                        std::thread::sleep(self.retry_delay);
                    }
                }
            }
        }
        unreachable!();
    }

    fn get_latest_response(&self) -> Option<&str> {
        if let Some(last) = self.responses.last() {
            if last.choices.len() >= 1 {
                last.choices
                    .first() // Get a reference to the first choice
                    .map(|choice| choice.message.content
                    .trim()
                    .trim_end_matches('\n')) 
            } else {
                return None;
            }
        } else {
            panic!("No responses found");
        }
    }
}

fn get_user_input() -> String {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim()
    .trim_end_matches('\n')
    .to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("INFO: Export your API key as OPENAI_API_KEY. Enter 'q', quit' or 'exit' to quit");

    if let Some(_var) = env::var_os("OPENAI_API_KEY") {
    } else {
        panic!("The environment variable OPENAI_API_KEY does not exist");
    }

    let mut chatbot = ChatBot::new(env::var("OPENAI_API_KEY")?);

    println!("> Provide a system instruction. Leave blank to skip.");
    let system_instruction = get_user_input();

    if system_instruction != "" {
        chatbot.retry_send_message("system", system_instruction.as_str())?;
    }

    loop {
        print!("> Enter your message: ");
        let input = get_user_input();

        if input == "q" || input == "quit" || input == "exit" {
            break;
        }

        chatbot.retry_send_message("user", input.as_str())?;
        let response = chatbot.get_latest_response();

        if let Some(answer) = response {
            println!("{}", answer.blue());
        } else {
            println!("{}", "No response found".red());
        }
    }

    Ok(())
}
