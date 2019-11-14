use chrono::NaiveDate;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use structopt::StructOpt;
use serde::Deserialize;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "csv2beancount",
    about = "convert transactions in CSV to beancount format"
)]
struct Opt {
    #[structopt(short = "c")]
    csv_path: PathBuf,
    #[structopt(short = "y")]
    yaml_path: PathBuf,
}



#[derive(Debug, Deserialize)]
struct YamlConfig {
    csv : Config,
    transactions: Option<HashMap<String, TransactionRule>>,
}

#[derive(Debug, Deserialize)]
struct Config {
    currency: String,
    processing_account: String,
    default_account: String,
    date_format: String,
    date: i64,
    amount_in: i64,
    amount_out: i64,
    description: i64,
    delimiter: Option<u8>,
    skip: Option<i64>,
    toggle_sign: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TransactionRule {
    account: Option<String>,
    info: Option<String>,
}

#[derive(Debug)]
struct Transaction<'a> {
    date: String,
    processing_account: &'a str,
    other_account: &'a str,
    currency: &'a str,
    magnitude: f64,
    description: &'a str,
    info: Option<&'a str>,
}

impl<'a> std::fmt::Display for Transaction<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            r#"{} * "{}" {}
  {} {} {}
  {} {} {}"#,
            self.date,
            self.description,
            if let Some(info) = self.info {
                format!(r#""{}""#, info)
            } else {
                "".into()
            },
            self.processing_account,
            self.magnitude,
            self.currency,
            self.other_account,
            self.magnitude * -1.0,
            self.currency
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let yaml_file = std::fs::File::open(&opt.yaml_path)?;
    let root_config: YamlConfig = serde_yaml::from_reader(yaml_file)?;
    let config = root_config.csv;
    let transaction_rules = root_config.transactions;
    let csv_file = std::fs::File::open(opt.csv_path)?;

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(config.delimiter.unwrap_or(b','))
        .has_headers(false)
        .from_reader(csv_file);

    let mut first = true;
    for result in rdr.records().skip(config.skip.unwrap_or(0) as usize) {
        let record = result.unwrap();

        if first {
            first = false;
        } else {
            println!();
        }

        let description = &record[config.description as usize];
        let date =
            NaiveDate::parse_from_str(&record[config.date as usize], &config.date_format)?;

        let t = Transaction {
            date: date.to_string(),
            description,
            info: {
                if let Some(rules) = transaction_rules.as_ref() {
                    match rules.get(description) {
                        Some(rule) => rule.info.as_ref().map(|s| s.as_str()),
                        None => None,
                    }
                } else {
                    None
                }
            },
            processing_account: &config.processing_account,
            other_account: {
                let specific = {
                    if let Some(rules) = transaction_rules.as_ref() {
                        match rules.get(description) {
                            Some(rule) => rule.account.as_ref(),
                            None => None,
                        }
                    } else {
                        None
                    }
                };

                if let Some(acc) = specific {
                    &acc
                } else {
                    &config.default_account
                }
            },
            magnitude: {
                let in_amount = &record[config.amount_in as usize];
                let out_amount = &record[config.amount_out as usize];
                let toggle = if config.toggle_sign.is_some() && config.toggle_sign.unwrap() {
                    -1.0
                } else {
                    1.0
                };

                if let Ok(amt) = in_amount.parse::<f64>() {
                    amt * toggle
                } else if let Ok(amt) = out_amount.parse::<f64>() {
                    amt * toggle * -1.0
                } else {
                    Err(format!("Could not parse either in or out amounts for {}", description))?
                }
            },
            currency: &config.currency,
        };

        println!("{}", t)
    }

    Ok(())
}
