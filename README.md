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

## Beta Warning

This project is in beta and could have major changes to API or behavior in future updates. Below are some issues the project currently has:

Internal issues:
 - Doesn't have unit tests
 - Uses unsafe code to access private variables from SeaORM
 - If migrations are not all run sequentially (user upgrades by 2 or more versions of the application at once) the migration version will get out of date with the migration code.

Missing features:
 - Creating join tables
 - Automatically doing basic migrations such as adding a column based on the SeaORM Entity

## Install

Add `sea-migrations` to your dependencies:

```toml
[dependencies]
# ...
sea-migrations= "0.0.1"
```
## Usage

Check out [this example application](https://github.com/oscartbeaumont/sea-migrations/tree/main/example).