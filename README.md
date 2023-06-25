# Unofficial Ydb Client library

There is an alternative of [`ydb`]

[`ydb`]: https://crates.io/crates/ydb
### Targets:

- more usability
- more freedom to use wrappers or raw objects from grpc bindings
- ability to create your own implementation of common traits, like `Credentials`
- easy to use pool objects

### Features

- [x] YQL Query for data (like DML)
- [x] YQL Query for sheme (like DDL)
- [x] Connect over grpcs (with tls)
- [ ] Connect over grps (without tls) - not worked, unknown cause
- [x] Connection pool (with [`deadpool`])
- [x] Token authentication
- [ ] Service account key authentication
- [ ] Metadata authentication
- [ ] Query helpers (a lot of)
- [ ] Connection string 

[`deadpool`]: https://crates.io/crates/deadpool