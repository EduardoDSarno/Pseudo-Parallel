use std::{collections::HashMap, io::Write, net::TcpStream};
pub struct HttpResponse
{
    status_code :u8,
    status_text: String,
    headers: HashMap<String, String>,
    body:   Option<String>
}

impl HttpResponse {

    pub fn new(status_code :u8, status_text: String,
               headers: HashMap<String, String>,body: Option<String>) -> HttpResponse
    {

        let http_response = HttpResponse
        {
            status_code: status_code,
            status_text: status_text,
            headers : headers,
            body : body
        };

        http_response
    }

    pub fn respond(response: String, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>>{

        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
    pub fn format_response(response: HttpResponse) -> String
    {
    let body_str = response.body.unwrap_or_else(|| "".to_string());
    let content_length = body_str.len();

    // Start with the status line
    let mut response_text = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status_code, response.status_text
    );

    // Add headers, but Content-Length is auto-appended/overrides any set in .headers
    for (key, value) in &response.headers {
        // Don't print duplicate Content-Length (let's add ours below)
        if key.to_lowercase() != "content-length" {
            response_text.push_str(&format!("{}: {}\r\n", key, value));
        }
    }

    // Always ensure Content-Length is correct
    response_text.push_str(&format!("Content-Length: {}\r\n", content_length));

    // End of headers
    response_text.push_str("\r\n");

    // Append the body if any
    response_text.push_str(&body_str);

    // For convenience, return the result (could be used as bytes with .as_bytes())
    // Option 1: return full String (usable for writing to a stream)
    response_text

    }
}