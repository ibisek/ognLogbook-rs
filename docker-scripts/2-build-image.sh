#!/bin/bash

cd ..

# docker build --tag ogn_logbook_rs:0.1 .
docker build --no-cache --tag ogn_logbook_rs:0.1 .

cd -
