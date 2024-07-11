## which-allowed
A cli tool to check allowed actions for IAM entities.


## Usage
```
CLI tool to check allowed actions for IAM entities.
Use it inside an environment where the cli can retrieve IAM credentials,
which has IAMReadOnly or above permissions.

Usage: which-allowed --entity-type <ENTITY_TYPE> --entity-name <ENTITY_NAME> --action-name <ACTION_NAME>

Options:
      --entity-type <ENTITY_TYPE>  The type of IAM Entity: Is Either "role" or "user". [possible values: user, role]
      --entity-name <ENTITY_NAME>  The name of IAM Entity
      --action-name <ACTION_NAME>  The name of action IAM entity performed
  -h, --help                       Print help
```



## Downloading and Using the Release

You can download the pre-built binaries from the [Releases](https://github.com/runjivu/which-allowed/releases) page on GitHub. 

Choose the appropriate binary for your operating system (Linux, macOS, Windows) and download it.

## Build
If you would like to build the project manually, you need to have Rust installed. 

Follow the instructions below to build the project:

```bash
git clone https://github.com/runjivu/which-allowed.git
cd which-allowed
cargo build --release
./target/release/which-allowed --entity-type <ENTITY_TYPE> --entity-name <ENTITY_NAME> --action-name <ACTION_NAME>
```
