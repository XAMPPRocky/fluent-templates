# `fluent-template-helper`

![CI Status](https://github.com/XAMPPRocky/fluent-template-helper/workflows/Rust/badge.svg?branch=master&event=push)
![Current Version](https://img.shields.io/crates/v/fluent-template-helper.svg)
[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/fluent-template-helper.svg)](#license)


This crate provides you with the ability to create [Fluent](https://docs.rs/fluent) loaders that implement [Handlebars](https://docs.rs/handlebars/)' `handlebars::HelperDef` and [Tera](https://docs.rs/tera) `tera::Function`. Allowing you to easily add localisation to your templating engines.

All template engine implementations are optional and can be disabled with features.
