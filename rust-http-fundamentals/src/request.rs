use std::{collections::HashMap, net::SocketAddr};

#[derive(Debug)]
pub enum Method {
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

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    path: String,
    version: String,
    headers: HashMap<String, String>,  // arbitrary number of headers
    content_length: Option<usize>,
    body: Option<String>,
}


impl HttpRequest{

    pub fn method(&self) -> &Method { &self.method }
    pub fn path(&self) -> &str { &self.path }
    pub fn version(&self) -> &str { &self.version }
    pub fn headers(&self) -> &HashMap<String, String> { &self.headers }
    pub fn content_length(&self) -> Option<usize> { self.content_length }
    pub fn body(&self) -> Option<&str> { self.body.as_deref() }

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

/* CONTEXT OF REQUEST */
#[derive(Debug)]
pub struct RequestContext {
    request: HttpRequest,
    addr: SocketAddr,
}

impl RequestContext{

    pub fn request(&self) -> &HttpRequest { &self.request }
    pub fn addr(&self) -> &SocketAddr { &self.addr }

    pub fn new(request: HttpRequest, addr: SocketAddr,)->RequestContext{

        let req_cntx = RequestContext{
            request: request,
            addr: addr
        };

        req_cntx
    }
}