extern crate envy;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate hyper;
extern crate hyper_native_tls;
extern crate url;
extern crate hdrsample;

use hdrsample::Histogram;
use hyper::Client;
use hyper::header::{Authorization, Basic};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use std::time::Duration;
use url::Url;

pub mod errors {
    error_chain!{
        errors {
            InvalidJob(name: String)
        }
        foreign_links {
            Config(::envy::Error);
            TLS(::hyper_native_tls::native_tls::Error);
            JSON(::serde_json::Error);
            HTTP(::hyper::Error);
            URL(::url::ParseError);
            //Sample(::hdrsample::RecordError);
        }
    }
}
use errors::{Result, ResultExt};

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

fn run() -> Result<()> {
    let config = envy::from_env::<Config>()
        .chain_err(|| "Invalid config")?;
    let mut url = Url::parse(
        &format!(
            "{host}/job/{job}/api/json",
            host = config.jenkins_host,
            job = config.job
        ),
    )?;
    url.query_pairs_mut()
        .extend_pairs(
            vec![
                ("pretty", "true"),
                ("tree",
                 "builds[number,id,timestamp,result,\
                                              duration]{0,}"),
            ],
        );
    let res = Client::with_connector(HttpsConnector::new(NativeTlsClient::new()?))
        .get(url)
        .header(
            Authorization(
                Basic {
                    username: config.jenkins_username,
                    password: Some(config.jenkins_password),
                },
            ),
        )
        .send()?;
    if !res.status.is_success() {
        return Err(errors::ErrorKind::InvalidJob(config.job).into());
    }
    let builds = serde_json::from_reader::<_, Builds>(res)
        .chain_err(|| "failed to parse builds")?
        .builds;
    let successes = builds
        .iter()
        .filter(|b| b.result.iter().find(|r| *r == "SUCCESS").is_some())
        .collect::<Vec<_>>();
    let mut histo = Histogram::<u64>::new(2).unwrap();
    for build in successes {
        histo.record(build.duration).unwrap();
    }
    //let sum = successes
    //    .iter()
    //    .fold(0, |res, build| res + build.duration);
    println!(
        "{job}.build_count {value}",
        job = config.job,
        value = histo.count()
    );
    println!(
        "{job}.build_duration.mean {value}",
        job = config.job,
        value = Duration::from_millis(histo.mean() as u64).as_secs() / 60 //
    );
    println!(
        "{job}.build_duration.max {value}",
        job = config.job,
        value = Duration::from_millis(histo.max()).as_secs() / 60 //
    );
    println!(
        "{job}.build_duration.min {value}",
        job = config.job,
        value = Duration::from_millis(histo.min()).as_secs() / 60 //
    );
    println!(
        "{job}.build_duration.stdev {value}",
        job = config.job,
        value = Duration::from_millis(histo.stdev() as u64).as_secs() / 60 //
    );
    println!(
        "{job}.build_duration.90p {value}",
        job = config.job,
        value = Duration::from_millis(histo.value_at_percentile(90.0) as u64).as_secs() / 60 //
    );
    Ok(())
}
fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        use error_chain::ChainedError; // trait which holds `display`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display()).expect(errmsg);
        ::std::process::exit(1);
    }
}
