macro_rules! get_url! { 
    ($path:expr, $param:expr) => ({

        // request local api, get article json object
        let url = "http://127.0.0.1:1337/".to_owned() + $path + "?" + $param;

        let mut client = Client::new();
        let mut cres = client.get(&url).send().unwrap();

        // Read the Response.
        let mut cbody = String::new();
        cres.read_to_string(&mut cbody).unwrap();
        println!("{}", cbody);

        match serde_json::from_str(cbody) {
            Ok(val) => {
                Some(val)
            },
            Err(_) => {
                None
            }
        }
    })
}

macro_rules! post_url! { 
    ($path:expr, $param:expr) => ({

        // request local api, get article json object
        let url = "http://127.0.0.1:1337/".to_owned() + $path;

        let mut client = Client::new();
        let mut cres = client.post(&url)
            .body($param) 
            .send().unwrap();

        // Read the Response.
        let mut cbody = String::new();
        cres.read_to_string(&mut cbody).unwrap();
        println!("{}", cbody);

        match serde_json::from_str(cbody) {
            Ok(val) => {
                Some(val)
            },
            Err(_) => {
                None
            }
        }
    })
}


mod article_page;

