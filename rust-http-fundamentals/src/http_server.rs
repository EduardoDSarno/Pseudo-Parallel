
use std::{fmt::{Debug, Error}, io::{BufRead, BufReader}, net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream}};
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
            handle_client(stream, addr); 
        });
    }
    Ok(())
}


pub fn handle_client(stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>>{

    /* Creatinga buffReader, it a rust struct that it have the job off
        copying the whole message in a buffer instaed of us manually doing
        syscalls over and over (which is expensive)*/
    let reader = BufReader::new(&stream);

    let request_line = reader.lines().next(); // "GET /hello HTTP/1.1"

    Request 

    Ok(())
}



