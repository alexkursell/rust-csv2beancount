use chrono::NaiveDate;
use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;
use yaml_rust::YamlLoader;

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

#[derive(Debug)]
struct YamlConfig {
    delimiter: u8,
    currency: String,
    processing_account: String,
    default_account: String,
    date_index: i64,
    amount_in_index: i64,
    amount_out_index: i64,
    skip_lines: i64,
    date_format: String,
    toggle_sign: bool,
    description_index: i64,
    transaction_rules: HashMap<String, TransactionRule>,
}

#[derive(Debug)]
struct TransactionRule {
    account: Option<String>,
    info: Option<String>,
}

#[derive(Debug)]
struct Transaction {
    date: String,
    processing_account: String,
    other_account: String,
    currency: String,
    magnitude: f64,
    description: String,
    info: Option<String>,
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            r#"{} * "{}"
  {} {} {}
  {} {} {}"#,
            self.date,
            self.description,
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

    let mut yaml_string = "".to_owned();
    let mut yaml_file = std::fs::File::open(&opt.yaml_path)?;
    let _ = yaml_file.read_to_string(&mut yaml_string)?;
    let raw_yaml = YamlLoader::load_from_str(&yaml_string)?;
    let raw_config = &raw_yaml[0]["csv"];
    let raw_transactions = &raw_yaml[0]["transactions"];

    let config = YamlConfig {
        delimiter: raw_config["delimiter"]
            .as_str()
            .map_or(b',', |s| s.bytes().collect::<Vec<u8>>()[0]),
        currency: raw_config["currency"].as_str().unwrap().into(),
        processing_account: raw_config["processing_account"].as_str().unwrap().into(),
        default_account: raw_config["default_account"].as_str().unwrap().into(),
        date_index: raw_config["date"].as_i64().unwrap(),
        amount_in_index: raw_config["amount_in"].as_i64().unwrap(),
        amount_out_index: raw_config["amount_out"].as_i64().unwrap(),
        skip_lines: raw_config["skip"].as_i64().unwrap_or(0),
        date_format: raw_config["date_format"]
            .as_str()
            .unwrap_or("%m/%d/%Y")
            .into(),
        toggle_sign: raw_config["toggle_sign"].as_bool().unwrap_or(false),
        description_index: raw_config["description"].as_i64().unwrap(),
        transaction_rules: {
            if let Some(transactions) = raw_transactions.as_hash() {
                transactions
                    .iter()
                    .map(|(key, val)| {
                        let key: String = key.as_str().unwrap().into();
                        let val = TransactionRule {
                            account: val["account"].as_str().map(|s| s.into()),
                            info: val["info"].as_str().map(|s| s.into()),
                        };
                        (key, val)
                    })
                    .collect::<HashMap<String, TransactionRule>>()
            } else {
                HashMap::new()
            }
        },
    };

    let csv_file = std::fs::File::open(opt.csv_path)?;

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(config.delimiter)
        .has_headers(false)
        .from_reader(csv_file);

    let mut first = true;
    for result in rdr.records().skip(config.skip_lines as usize) {
        let record = result.unwrap();

        if first {
            first = false;
        } else {
            println!();
        }

        let description = &record[config.description_index as usize];
        let date =
            NaiveDate::parse_from_str(&record[config.date_index as usize], &config.date_format)?;

        let t = Transaction {
            date: date.to_string(),
            description: description.into(),
            info: {
                match config.transaction_rules.get(description) {
                    Some(rule) => rule.info.clone(),
                    None => None,
                }
            },
            processing_account: config.processing_account.clone(),
            other_account: config
                .transaction_rules
                .get(description)
                .map(|r| r.account.clone().unwrap_or(config.default_account.clone()))
                .unwrap_or(config.default_account.clone()),
            magnitude: {
                let in_amount = &record[config.amount_in_index as usize];
                let out_amount = &record[config.amount_out_index as usize];
                let toggle = if config.toggle_sign { -1.0 } else { 1.0 };

                if let Ok(amt) = in_amount.parse::<f64>() {
                    amt * toggle
                } else if let Ok(amt) = out_amount.parse::<f64>() {
                    amt * toggle * -1.0
                } else {
                    panic!()
                }
            },
            currency: config.currency.clone(),
        };

        println!("{}", t)
    }

    Ok(())
}
