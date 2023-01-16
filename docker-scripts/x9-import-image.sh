#!/bin/bash

DIR=/tmp/00/

unxz -k $DIR/ogn_logbook_rs.image.tar.xz

# docker image import $DIR/ogn_logbook_rs.image.tar ogn_logbook_rs:0.1
docker image import $DIR/ogn_logbook_rs.image.tar ogn_logbook_rs
