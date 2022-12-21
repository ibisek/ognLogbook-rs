#!/bin/bash

while true
do
    clear;export RUST_BACKTRACE=1 && cargo run --release 2>&1 |tee -a app.log
done
