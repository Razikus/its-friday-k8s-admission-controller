use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, ec_private_keys, pkcs8_private_keys, rsa_private_keys};

use chrono::Datelike;
use core::panic;
use std::{default, fs::File, io::BufReader};

use std::env;

struct AppState {
    pod_name: String,
    excluded_days: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    status: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct K8SStatus {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct K8SResponse {
    uid: String,
    allowed: bool,
    status: K8SStatus,
}

#[derive(Debug, Serialize, Deserialize)]
struct AdmissionReview {
    apiVersion: String,
    kind: String,
    response: K8SResponse,
}

#[get("/health")]
async fn health() -> impl Responder {
    let status = Message { status: true };
    return HttpResponse::Ok().json(status);
}

#[get("/ready")]
async fn ready() -> impl Responder {
    let status = Message { status: true };
    return HttpResponse::Ok().json(status);
}

fn generateAdmission(
    uid: String,
    status: bool,
    apiVersion: String,
    message: String,
) -> AdmissionReview {
    let mut code = 403;
    if (status == true) {
        code = 200;
    }

    let response = AdmissionReview {
        apiVersion: apiVersion,
        kind: String::from("AdmissionReview"),
        response: K8SResponse {
            uid: uid,
            allowed: status,
            status: K8SStatus {
                code: code,
                message: message,
            },
        },
    };
    return response;
}

#[post("/validate")]
async fn validate(data: web::Data<AppState>, body: web::Bytes) -> impl Responder {
    match serde_json::from_slice::<Value>(&body) {
        Ok(val) => {
            let mapped = val.as_object().unwrap();
            let request = mapped.get("request").unwrap().as_object().unwrap();

            let mut shouldPassBecauseOfParentHolder = false;
            let object = request.get("object");
            if(object.is_some()) {
                let metadata = object.unwrap().get("metadata");
                if(metadata.is_some()) {
                    let ownerReferences = metadata.unwrap().get("ownerReferences");
                    if(ownerReferences.is_some()) {
                        let arrayOfReferences = ownerReferences.unwrap().as_array();
                        if(arrayOfReferences.is_some()) {
                            let sizeOfOwners = arrayOfReferences.unwrap().len();
                            if(sizeOfOwners > 0) {
                                shouldPassBecauseOfParentHolder = true;
                            }
                        }
                    }
                }
            }
            let apiVersion = mapped.get("apiVersion");
            let apiVersionString: String;
            if (apiVersion.is_some()) {
                apiVersionString = apiVersion.unwrap().as_str().unwrap().into();
            } else {
                apiVersionString = "v1".into();
            }
            let uidStr = request.get("uid").unwrap().as_str().unwrap();
            if(shouldPassBecauseOfParentHolder) {
                return HttpResponse::Ok().json(generateAdmission(
                    uidStr.into(),
                    true,
                    apiVersionString,
                    format!("{}: Accepted request {}, owner references found (automated job)!", data.pod_name, uidStr).into(),
                ));
            }

            let current_time = chrono::offset::Local::now();
            let currentweekday = current_time.weekday();
            let weekday: u32 = currentweekday.num_days_from_monday();
            for disabled in data.excluded_days.iter() {
                if (*disabled == weekday) {
                    return HttpResponse::Ok().json(generateAdmission(
                        uidStr.into(),
                        false,
                        apiVersionString,
                        format!("{}: Rejected request {}, ITS {currentweekday}!", data.pod_name, uidStr).into(),
                    ));
                }
            }
            return HttpResponse::Ok().json(generateAdmission(
                uidStr.into(),
                true,
                apiVersionString,
                format!("{}: Accepted request {}, ITS {currentweekday}!", data.pod_name, uidStr).into(),
            ));
        }
        Err(_) => {
            panic!("Admission request not json deserializable");
        }
    }
}

fn load_rustls_config(crtfile: String, keyfile: String) -> rustls::ServerConfig {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    println!("{} ", crtfile);
    println!("{}", keyfile);
    let cert_file = &mut BufReader::new(File::open(crtfile).unwrap());
    let key_file = &mut BufReader::new(File::open(keyfile).unwrap());

    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    let mut keys: Vec<PrivateKey> = rsa_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    if keys.is_empty() {
        eprintln!("Could not locate RSA private keys.");
        keys = ec_private_keys(key_file)
            .unwrap()
            .into_iter()
            .map(PrivateKey)
            .collect();

        if keys.is_empty() {
            eprintln!("Could not locate EC private keys.");
            keys = pkcs8_private_keys(key_file)
                .unwrap()
                .into_iter()
                .map(PrivateKey)
                .collect();
            if keys.is_empty() {
                eprintln!("Could not locate PKCS 8 private keys. Exiting program");
                std::process::exit(1);
            }
        }
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let crtfile = &args[1];
    let keyfile = &args[2];
    // let vec = Vec::new();

    let config = load_rustls_config(crtfile.to_string(), keyfile.to_string());
    HttpServer::new(|| {
        App::new()
            .service(health)
            .service(ready)
            .service(validate)
            .app_data(web::Data::new(AppState {
                pod_name: String::from(env::var("APP_POD_NAME").unwrap()),
                excluded_days: env::var("MY_FRIDAY")
                    .unwrap()
                    .split(",")
                    .map(|s| s.parse().unwrap())
                    .collect(),
            }))
    })
    .bind_rustls(("0.0.0.0", 443), config)?
    .run()
    .await
}
