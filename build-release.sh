#!/bin/bash

cargo build --release
strip target/release/khonsu
