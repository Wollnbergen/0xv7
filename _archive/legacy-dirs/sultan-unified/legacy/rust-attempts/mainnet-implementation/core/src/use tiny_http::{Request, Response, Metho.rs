use tiny_http::{Request, Response, Method, Header};

fn cors_headers() -> Vec<Header> {
    vec![
        Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
        Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"POST, OPTIONS"[..]).unwrap(),
        Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Authorization, Content-Type"[..]).unwrap(),
    ]
}

fn handle_request(request: &Request) -> Response {
    let method = request.method().clone();
    if method == &Method::Options {
        // Reply to CORS preflight
        let mut resp = Response::empty(204);
        for h in cors_headers() {
            resp.add_header(h.clone());
        }
        let _ = request.respond(resp);
        return Response::empty(0);
    }

    // When building normal responses, add the CORS headers before sending.
    // Example: replace creating `response` with adding headers:
    let mut response = Response::from_string("Hello, World!");
    for h in cors_headers() {
        response.add_header(h.clone());
    }
    let _ = request.respond(response);

    Response::empty(0)
}