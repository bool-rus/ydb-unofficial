# Unofficial Ydb Client library

There is an alternative of [`ydb`]

[`ydb`]: https://crates.io/crates/ydb
### Targets:

- more usability
- more freedom to use wrappers or raw objects from grpc bindings
- ability to create your own implementation of common traits, like `Credentials`
- easy to use pool objects

### Goals:

- [x] YQL Query for data (like DML)
- [x] YQL Query for sheme (like DDL)
- [x] Connect over grpcs (with tls)
- [ ] Connect over grpc (without tls) - not worked, unknown cause
- [x] Connection pool (with [`deadpool`]) (feature `pool`)
- [x] Token authentication
- [x] Service account key authentication (feature `auth-sa`)
- [ ] Metadata authentication
- [ ] Query helpers (a lot of)
- [`sqlx`] integration - partially done (feature `sqlx`):
    - [x] Connection string 
    - [x] connection 
    - [x] binding parameters
    - [x] preparing statements
    - [x] transaction manager
    - [x] DML (data) operations
    - [x] DDL (scheme) operations
    - [x] primitive types (bool, i8, i32, i64, u8, u32, u64, f32, f64, Vec\<u8\>, String)
    - [x] date types (Date, Datetime, Timestamp, Interval)
    - [x] json type
    - [ ] Decimal type
    - [ ] connection pool balancing for discovery
    - [ ] compile-time checked queries
    - [x] migrations
    - [ ] multiple transaction modes
    - [x] log statements
- [ ] operation parameters

[`deadpool`]: https://crates.io/crates/deadpool
[`sqlx`]: https://crates.io/crates/sqlx
