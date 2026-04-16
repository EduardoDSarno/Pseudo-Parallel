
#[derive(Debug)]
enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Trace,
    Connect,
    Patch,
}



use std::collections::HashMap;

pub struct HttpRequest {
    method: Method,
    path: String,
    version: String,
    headers: HashMap<String, String>,  // arbitrary number of headers
    content_length: Option<usize>,
    body: Option<String>,
}


impl HttpRequest{

    pub fn new(method:String, path:String, version:String,
              headers: HashMap<String, String>, content_length: Option<usize>, body:Option<String>)
              -> HttpRequest
    {
        let method_e = match method.to_uppercase().as_str() {
            "GET"     => Method::Get,
            "POST"    => Method::Post,
            "PUT"     => Method::Put,
            "DELETE"  => Method::Delete,
            "HEAD"    => Method::Head,
            "OPTIONS" => Method::Options,
            "TRACE"   => Method::Trace,
            "CONNECT" => Method::Connect,
            "PATCH"   => Method::Patch,
            _ => panic!("Unsupported HTTP method: {}", method),
        };
   
        let httpreq = HttpRequest
        {
            method : method_e, 
            path   : path, 
            version: version,
            headers: headers,
            content_length,
            body:    body
        };
        httpreq
    }
}