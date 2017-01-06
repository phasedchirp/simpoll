#[macro_use] extern crate nickel;
extern crate rand;
extern crate rustc_serialize;
extern crate rusqlite;
// extern crate chrono;


use nickel::{Nickel, HttpRouter, MediaType, FormBody};
use nickel::status::StatusCode;

use rusqlite::Connection;

use std::collections::HashMap;
use std::fs::{File,remove_file,read_dir};
use std::io::{Read,Write};
use std::sync::{Arc,RwLock,Mutex};

mod utils;
use utils::*;



fn survey_from_file(survey_file: &str) -> Result<Vec<Question>,u32> {
    match File::open(survey_file) {
        Ok(mut f) => {
            let mut buf = String::new();
            f.read_to_string(&mut buf);
            let qs: Vec<&str> = buf.trim().split("\r\n").collect();
            Ok(make_questions(&qs))
        },
        Err(_) => Err(400)
    }
}



fn main() {
    let mut server = Nickel::new();
    let mut conn_arc = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));

    let mut surveys = HashMap::new();
    let paths = read_dir("surveys/").unwrap();
    for path in paths {
        if let Ok(p) = path {
            let id = p.file_name().into_string().unwrap();
            let survey_file = format!("surveys/{}",&id);
            surveys.insert(id.clone(),survey_from_file(&survey_file).unwrap());
        }
    }

    let mut surveys_arc = Arc::new(RwLock::new(surveys));

    // log requests to stdout:
    server.utilize(middleware! { |request|
        println!("logging request: {:?}", request.origin.uri);
    });

    // route for survey creation page:
    server.get("/survey/new", middleware! { |_, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        return resp.send_file("resources/makeSurvey.html");
    });

    // route (plus setup) for adding user-created surveys:
    let surveys_clone_make = surveys_arc.clone();
    let conn_clone = conn_arc.clone();
    server.post("/survey/created", middleware!{ |req, mut resp|
        resp.set(StatusCode::Ok);
        resp.set(MediaType::Html);
        let form_data = try_with!(resp,req.form_body());
        let survey_id = new_id(6);

        let mut surveys = surveys_clone_make.write().unwrap();
        let conn = conn_clone.lock().unwrap();


        let file_name = format!("surveys/{}",&survey_id);
        let mut fr = File::create(file_name);
        match fr {
            Ok(mut f) => {
                let qs = form_data.get("questions").unwrap();
                surveys.insert(survey_id.clone(),
                        make_questions(&(qs.split("\r\n").collect())));

                // conn.execute(&insert_survey(surveys.get(&survey_id).unwrap())).unwrap();
                f.write_all(&qs.as_bytes());
                let mut data = HashMap::new();
                data.insert("path",format!("survey/{}",survey_id));
                return resp.render("resources/path.tpl", &data);
            },
            Err(e) => {println!("{:?}",e);}
        }
    });

    // route for taking a survey
    let surveys_clone_take = surveys_arc.clone();
    server.get("/survey/:foo", middleware!{ |req, mut resp|
        let survey_id = req.param("foo").unwrap().to_string();
        let surveys = surveys_clone_take.read().unwrap();
        let mut qs_parsed = String::new();
        let mut data = HashMap::new();

        match surveys.get(&survey_id) {
            Some(qs) => {
                resp.set(StatusCode::Ok);
                resp.set(MediaType::Html);
                data.insert("id",survey_id);
                qs_parsed = parse_survey(&qs);
                data.insert("questions",qs_parsed);
                return resp.render("resources/takeSurvey.tpl",&data);
            },
            None => {
                resp.set(StatusCode::NotFound);
                "That survey ID doesn't seem to exist"
            }
        }
    });

    // route for submitting completed survey
    server.post("survey/:foo/submit", middleware!{ |req, mut resp|
        let conn = conn_arc.lock().unwrap();
        let survey_id = req.param("foo").unwrap().to_owned();
        let form_data = try_with!(resp,req.form_body());
        let surveys = surveys_arc.read().unwrap();
        let user_id = new_id(10);
        println!("{:?}", form_data);

    });



    server.listen("127.0.0.1:6767");
}
