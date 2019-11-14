# rust-csv2beancount

A simple tool for converting a csv file of transactions to beancount format.

## Example

In `config.yaml`:

```yaml
csv:
  date: 0
  date_format: "%m/%d/%Y"
  amount_out: 3
  amount_in: 3
  description: 2
  default_account: Expenses:Unknown
  processing_account: Assets:Chequing
  currency: CAD
  skip: 1
transactions:
  "INTEREST CREDIT":
    account: "Income:Interest"
    info: "This is interest income"
```

In `transactions.csv`:

```csv
Date,Description,Original Description,Amount,Transaction Type,Category,Account Name,Labels,Notes
2/28/2018,Cell Phone Top-Up,PHONE COMPANY,-11.30,debit,Mobile Phone,MY ACCOUNT,,
3/02/2018,Interest,INTEREST CREDIT,0.06,credit,Interest Income,MY ACCOUNT,,
```

```shell
$ rust-csv2beancount -y config.yaml -c transactions.csv
2018-02-28 * "PHONE COMPANY" 
  Assets:Chequing -11.3 CAD
  Expenses:Unknown 11.3 CAD

2018-03-02 * "INTEREST CREDIT" "This is interest income"
  Assets:Chequing 0.06 CAD
  Income:Interest -0.06 CAD
```

## Configuration Format

```yaml
csv:
  # (Required) The currency that all transactions are assumed to use
  currency: CAD
  # (Required) The account that all transactions are depositing to or withdrawing from
  processing_account: Assets:Chequing
  # (Required) The default "other" account
  default_account: Expenses:Unknown
  # (Required) The date format, in strftime format
  date_format: "%m/%d/%Y"
  # (Required) The column index (from 0) containing the transaction date
  date: 0
  # (Required) The column index (from 0) containing the amount transferred to the "other" account
  amount_in: 3
  # (Required) The column index (from 0) containing the amount transferred from the processing account
  amount_out: 3
  # (Required) The column index (from 0) containing the transaction description
  description: 2
  # The csv delimiter (default ",")
  delimiter: ","
  # The number of csv rows to skip at the start (default 0)
  skip: 0
  # Whether by default the transaction moves the currency from the "other" account to the processing account
  toggle_sign: false
transactions:
  # The description to match
  "INTEREST CREDIT":
    # The "other" account name for transactions with the description
    account: "Income:Interest"
    # An optional "info" field for this type of transaction
    info: "This is interest income"
```

## Compatability

This tool tries to mirror https://github.com/PaNaVTEC/csv2beancount exactly, except:

- `date_format` requires `strftime` format, not the bespoke one the original uses
- `info` in a `transactions` rule actually does something


