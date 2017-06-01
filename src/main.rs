extern crate envy;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate hyper;
extern crate hyper_native_tls;
extern crate url;

use hyper::Client;
use hyper::header::{Authorization, Basic};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use std::time::Duration;
use url::Url;

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
    result: Option<String>,
    timestamp: u64,
}

fn main() {
    match envy::from_env::<Config>() {
        Ok(config) => {
            let mut url = Url::parse(
                &format!(
                    "{host}/job/{job}/api/json",
                    host = config.jenkins_host,
                    job = config.job
                )
            )
                    .unwrap();
            url.query_pairs_mut()
                .extend_pairs(
                    vec![
                        ("pretty", "true"),
                        ("tree",
                         "builds[number,id,timestamp,result,\
                                                      duration]{0,}"),
                    ]
                );
            let res = Client::with_connector(HttpsConnector::new(NativeTlsClient::new().unwrap()))
                .get(url)
                .header(
                    Authorization(
                        Basic {
                            username: config.jenkins_username,
                            password: Some(config.jenkins_password),
                        }
                    )
                )
                .send()
                .unwrap();
            let builds = serde_json::from_reader::<_, Builds>(res).unwrap().builds;
            let successes = builds
                .iter()
                .filter(|b| b.result.iter().filter(|r| *r == "SUCCESS").next().is_some())
                .collect::<Vec<_>>();
            let sum = successes.iter().fold(0, |res, build| res + build.duration);
            println!(
                "{job}.build_count {value}",
                job = config.job,
                value = successes.len()
            );
            println!(
                "{job}.build_duration {value}",
                job = config.job,
                value = Duration::from_millis(sum / successes.len() as u64).as_secs() / 60
            );
        }
        Err(err) => println!("error: {}", err),
    }

}
