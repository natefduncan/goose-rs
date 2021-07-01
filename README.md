# Goose

## Installation

`cargo install --path .`

## Usage 

`goose --help`

```
Goose 1.0
Nate D.
Query Duck Duck Go to get location data.

USAGE:
    goose [OPTIONS] <QUERY> <LOCATION> [SAVE]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --concurrency <CONCURRENCY>    Set request concurrency. Default is 2.
    -d, --distance <DISTANCE>          Set the search distance. Default is 10 miles.
    -f, --file_type <FILE_TYPE>        Set the output file_type. Default is csv. Options: csv, json.

ARGS:
    <QUERY>       Sets your search value (e.g. Restaurant, Park, etc).
    <LOCATION>    Sets your location (e.g. Dallas, TX).
    <SAVE>        Sets your save location. Default is current directory.
```