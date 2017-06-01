extern crate envy;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate hyper;
extern crate hyper_native_tls;

use std::time::Duration;
use hyper::Client;
use hyper::header::{Authorization, Basic};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

#[derive(Deserialize)]
struct Config {
    jenkins_host: String,
    jenkins_username: String,
    jenkins_password: String,
    job: String,
}

#[derive(Deserialize, Debug)]
struct Builds {
    builds: Vec<Build>,
}

#[derive(Deserialize, Debug)]
struct Build {
    duration: u64,
    number: u32,
    result: String,
    timestamp: u64,
}

fn main() {
    match envy::from_env::<Config>() {
        Ok(config) => {
            let res = Client::with_connector(HttpsConnector::new(NativeTlsClient::new()
                    .unwrap()))
                .get(
                    &format!(
                        "{host}/job/{job}/api/json?pretty=true&tree=builds%5Bnumber%2Cid%2Ctimestamp%2Cresult%2Cduration%5D",
                        host = config.jenkins_host, job = config.job
                    )
                 )
                .header(Authorization(Basic {
                    username: config.jenkins_username,
                    password: Some(config.jenkins_password),
                }))
                .send()
                .unwrap();
            let builds = serde_json::from_reader::<_, Builds>(res).unwrap().builds;
            let sum = builds.iter().fold(0, |res, build| res + build.duration);
            println!("build count: {}", builds.len());
            println!("avg duration: {}",
                     Duration::from_millis(sum / builds.len() as u64).as_secs() / 60);
        }
        Err(err) => println!("error: {}", err),
    }

}
