#!/bin/bash

clear;export RUST_BACKTRACE=1 && . .env-local && cargo run 2>&1 | tee -a app.log

