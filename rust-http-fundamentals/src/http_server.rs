#[path = "request.rs"]
mod request;
use request::{HttpRequest, RequestContext};
use std::{collections::HashMap, io::{BufRead, BufReader, Read}, net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream}};
use std::thread;


/*Small http sample */
fn main() -> std::io::Result<()>{
    println!("http_server bin target is ready");
    

    // use defoult localhost from ipv4 + the port
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 7878))?;

    /*.incoming() is just a wrapper for calling accept within the loop */
    // for stream in listener.incoming()
    // {

    // }

    // Loop thorugh infinity listening to connections
    loop {
        let (stream, addr) = match listener.accept()
        {
            Ok((stream, addr)) => {
                println!("New Client at port {addr:?}");
                (stream, addr)

            }
            Err(e) =>{
                eprintln!("Failed to accept: {e}");
                continue;
            }
        };
        // handle stream
        /* What mov || does is a anoymous function that gives the ownership to handle client
            so that they don't lose that data */
        let thread_join_handle = thread::spawn(move ||{
            let request = handle_client(stream); 
        });
    }
    Ok(())
}

/*The handle client function that it will receive a raw Request and it will parse and return the Request St
ruct for easy manipualtion of data or it will return error */
pub fn handle_client(stream: TcpStream) -> Result<HttpRequest, Box<dyn std::error::Error>>{

    /* Creatinga buffReader, it a rust struct that it have the job off
        copying the whole message in a buffer instaed of us manually doing
        syscalls over and over (which is expensive)*/
    let reader = BufReader::new(&stream);

    // Create the iterator once - we reuse it with .by_ref() so ownership isn't lost
    let mut lines = reader.lines();

    /* This part will take care of the request line */
    let request_line: String = lines.next().ok_or("empty request")??;

    let mut parts = request_line.split_whitespace();
    let method  = parts.next().ok_or("missing method")?;
    let path    = parts.next().ok_or("missing path")?;
    let version = parts.next().ok_or("missing version")?;

    /*This part will take care of the headers mapping them to the hashmap */
    let headers: HashMap<String, String> = lines
    .by_ref()                             // borrow the iterator, don't consume it
    .map(|l| l.unwrap())
    .take_while(|line| !line.is_empty())  // stop at blank line
    .filter_map(|line| {
        let mut parts = line.splitn(2, ": ");   // split in 2 because the values can contain the ':' itself
        let key = parts.next()?.to_string();
        let val = parts.next()?.to_string();
        Some((key, val))
    })
    .collect();

    /* This part will be if there's a body */
    let mut body = None;
    let mut content_lenght:Option<usize> = None;
    /* This will check if there's a body and use it */
    if let Some(cl) = headers.get("Content-Length") {

        let len:usize = cl.parse()?;

        content_lenght = Some(len);
        //  claude recommendation to separate the result into lines on a vector
        // instaed of putting all in one string
        body = Some(lines.collect::<Result<Vec<String>, _>>()?.join("\n"));
    }

    let http_req = HttpRequest::new(
        method.to_string(),
        path.to_string(),
        version.to_string(),
        headers,
        content_lenght,
        body
    );
    Ok(http_req)
}

/*Get the call context
    Which includes the request and the SocketAddress */
pub fn get_context(req: HttpRequest, addr: SocketAddr) -> Result<RequestContext, Box< dyn std::error::Error>>
{
    let context = RequestContext::new(req, addr);
    Ok(context)
}

pub fn print_request(request: RequestContext){
    println!("========== Incoming HTTP Request ==========");
    println!("Remote Address: {}", request.addr());
    println!("--- Request Line ---");
    let req = request.request();

    println!("Method:   {:?}", req.method());
    println!("Path:     {}", req.path());
    println!("Version:  {}", req.version());

    println!("--- Headers ---");
    for (key, value) in req.headers() {
        println!("{}: {}", key, value);
    }

    if let Some(len) = req.content_length() {
        println!("Content-Length: {}", len);
    }

    match req.body() {
        Some(body) => {
            println!("--- Body ---");
            println!("{}", body);
        }
        None => {
            println!("--- No Body ---");
        }
    }
    println!("==========================================");
}




