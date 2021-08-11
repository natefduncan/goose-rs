# Goose

## Installation

`cargo install --path .`

## Usage 

`goose --help`

```
Goose 0.1.1
Nate D.
Query Duck Duck Go to get location data.

USAGE:
    goose [OPTIONS] <QUERY> <LOCATION>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --concurrency <CONCURRENCY>    Set request concurrency. Default is 1.
    -d, --distance <DISTANCE>          Set the search distance. Default is 10 miles.
    -f, --file-type <FILE-TYPE>        Set the output file_type. Default is json. Options: csv, json. 

ARGS:
    <QUERY>       Sets your search value (e.g. Restaurant, Park, etc). Multiple queries in the same location can be
                  separated by comma.
    <LOCATION>    Sets your location (e.g. Dallas, TX).
```