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

## Build
`cargo build --release`

If you are in MAC OS X environment, you can use the binary /target/release/which-allowed.
