#!/bin/bash

export DB_NAME=ogn_logbook_rs
export REDIS_PORT=6380

clear;export RUST_BACKTRACE=1 && cargo run

