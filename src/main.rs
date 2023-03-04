use reqwest::{blocking::Client, header::{HeaderMap, CONTENT_TYPE}};
use std::env;
use std::error::Error;
use std::io::{self, Write};

struct ChatBot {
    api_key: String,
    client: Client,
    headers: HeaderMap,
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
            responses: Vec::new(),
        }
    }

    fn send_message(&mut self, message: &str) -> reqwest::Result<()> {
        let url = "https://api.openai.com/v1/chat/completions";
        let request = Request {
        model: String::from("gpt-3.5-turbo"),
            messages: vec![Message {
                role: String::from("user"),
                content: message.to_string(),
            }],
        };
        let headers = self.headers.clone();
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Response {
    id: String,
    object: String,
    created: i64,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Choice {
    index: usize,
    message: Message,
    finish_reason: String,
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


fn main() -> Result<(), Box<dyn Error>> {
    let mut chatbot = ChatBot::new(env::var("OPENAI_API_KEY")?);
    println!("Export your API key as OPENAI_API_KEY. Enter 'exit' to quit");

    loop {
        print!("> Enter your message: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input == "exit" {
            break;
        }

        chatbot.send_message(input)?;
        let response = get_first_choice_content(chatbot.responses.last().unwrap()).unwrap();
        println!("{}", response); 
    }

    Ok(())
}
