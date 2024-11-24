# Owl-Panda

A toy web engine in Rust. Supports only limited set of HTML tags and css properties.
Some code for cascading and inheritance of css properties are put in place but
not tested. Feel free to fork and add inheritance, specificity etc.

## Instructions

1. [Install Rust 1.0 or newer version](https://www.rust-lang.org/tools/install)
2. Clone the repo.
3. Run cargo build to build Owl-Panda, and cargo run to run it.

By default it will load test.html and test.css along with default.css
from files directory.

```
./target/debug/owl-panda -h ./files/test.html -c ./files/test.css -o output.png
```

The page rendered will be saved as output.png
