# serde-matcher

Implements a matcher for `serde_json::Value`s using the MongoDB query language in Rust.

Currently supports `$eq`, `$in`, `$ne`, `$nin`, `$and`, `$not`, `$or`, `$type` and `$nor`.
