*This project has been deprecated as an upstream solution now exists for migrations. Check it out [here](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)*

<h1 align="center">Sea Migrations</h1>
<div align="center">
 <strong>
   Effortless database migrations for <a href="https://www.sea-ql.org/SeaORM/">SeaORM</a>!
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/sea-migrations">
    <img src="https://img.shields.io/crates/v/sea-migrations.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/sea-migrations">
    <img src="https://img.shields.io/crates/d/sea-migrations.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/sea-migrations">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>
<br/>

This crate aims to provide a simple solution to doing database migrations with [SeaORM](https://www.sea-ql.org/SeaORM/).

Features:
 - Automatically create database tables from your SeaORM entities
 - Write your migration code in Rust
 - Supports all SeaORM database backends
 - Basic protections against migration inconsistencies

## Beta Warning

This project is in beta and could have major changes to API or behavior in future updates. Please refer to [issue #1](https://github.com/oscartbeaumont/sea-migrations/issues/1) for more information.

## Install

Add `sea-migrations` to your dependencies:

```toml
[dependencies]
# ...
sea-migrations= "0.0.1"
```
## Usage

Check out [this example application](https://github.com/oscartbeaumont/sea-migrations/tree/main/example).
