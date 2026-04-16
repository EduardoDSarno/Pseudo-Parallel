
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



struct Request{

    method: Method,
    direct: String,
    version: f32,

}


impl Request{

    fn new(method:Method, direct:String, version:f32)-> Request{

        Request r = {}
    }
}