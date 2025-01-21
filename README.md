# Webserver
A simple webserver written entirely in Rust.

If you just want to run the webserver, get the <a href="https://github.com/Stephano1221/webserver/releases/latest">latest binary</a>, set up a [configuration file](#configuration) and run the executable.

## Configuration
Configuration files are written in <a href="https://toml.io">TOML</a>.

Configuration file locations:
- Windows: `C:\Users\%USERNAME%\AppData\Local\Webserver\config.toml`
- Unix: `/etc/webserver/config.toml`

Ensure that a configuration file is in a supported location. If values aren't provided, defaults will be used instead.

### Example config.toml
```toml
# Global configuration, which applies to all virtual hosts (currently, only global configuration
# is supported).
[global]

# The primary domain names, which are all treated as being identical to each other.
primary_domain_names = ["example.com", "www.example.com"]

# The port on which the server listens for incoming requests. Usually 80 for HTTP and 443 for HTTPS.
# Currently, HTTPS is unsupported.
port = 80

# A directory that contains one folder containing the primary domains' files, and one folder
# containing folders for each subdomain. This can be either relative or absolute.
parent_directory = "C:\ProgramData\Website"

# The name of the folder inside of the parent directory that contains the files that are served
# for the primary domains. This is where index.html for the primary domains would go.
primary_domain_folder_name = "primary_domain"

# The name of the folder inside of the parent directory that contains the folders for each subdomain.
# Each folder in here should have a name of a subdomain, inside of which index.html for that
# subdomain would go.
subdomains_folder_name = "subdomains"

# The default filename to retrieve if none is specified in the URL.
default_filename = "index.html"

# The default filename to retrieve if the requested file is not found.
not_found_filename = "404.html"

# The minimum amount of time that a request will be processed for.
# The actual processing time may be higher.
minimum_timeout_seconds = 5

# The initial buffer size, in kilobytes, that is allocated for each request.
initial_buffer_size_kilobytes = 16

# The maximum buffer size, in kilobytes, that is allocated for each request.
# If the request exceeds this size, it will be dropped without further processing
# or sending of a response.
maximum_buffer_size_kilobytes = 1024
```

Using the above example `config.toml`, the website folder structure could look something like:
```
C:/ProgramData/Website/         Example domains
├── primary_domain/
│   ├── index.html              (www.)example.com
│   ├── 404.html                (www.)example.com/hello
│   └── rust/
│       ├── index.html          (www.)example.com/rust
│       ├── hello.html          (www.)example.com/rust/hello.html
│       └── images/
│           └── index.html      (www.)example.com/rust/images
└── subdomains/
    └── shop/
        ├── index.html          shop.example.com
        ├── 404.html            shop.example.com/bread
        └── uk/
            ├── index.html      uk.shop.example.com | shop.example.com/uk
            └── 404.html        uk.shop.example.com/bread
```

> [!NOTE]
> The webserver can't yet distinguish between an identical, but inverted, subdomain and path (e.g., `uk.shop.example.com` and `shop.example.com/uk` - both point to the same file). This may change in the future, as subdomain folders could themselves contain a subdomain folder.

Note that if your HTML contains relative links to other files, the URL path (before the query string, which begins with `?`) may need to end with a forward slash so that web browsers treat the final element as a folder instead of a resource/file.

## Coding
If you want to modify the code yourself, follow the below steps:

1. Install <a href="https://www.rust-lang.org">Rust</a>. You can find instructions on how to do this <a href="https://www.rust-lang.org/learn/get-started">here</a>.
2. Install an IDE of your choice (such as <a href = "https://code.visualstudio.com">Visual Studio Code</a> with the <a href="https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer">rust-analyzer</a> and <a href="https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb">CodeLLDB</a> extensions)
3. <a href="https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository">Clone</a> the repository to your computer.
4. Open the repository in your IDE and begin writing!

### Building
To create an executable file, build the code by running:

```shell
cargo build
```

By default, the dev profile will be used, which has quicker build times, but the resulting executable will be slower.

To build a more optimised executable, run:

```shell
cargo build --release
```

For more information on building, visit the <a href="https://doc.rust-lang.org/cargo/commands/cargo-build.html">Cargo Book</a>.

### Testing
Unit tests are inside the file that they are testing. Integration tests are in the `tests` folder, on the same level as the `src` folder.

You can run all tests using:

```shell
cargo test
```

To run only library tests, run:

```shell
cargo test --lib
```

To run only doc tests, run:

```shell
cargo test --doc
```

For more information on running tests, visit the <a href="https://doc.rust-lang.org/cargo/commands/cargo-test.html">Cargo Book</a>.
