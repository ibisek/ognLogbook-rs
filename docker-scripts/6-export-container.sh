#!/bin/bash

DIR=/tmp/00/

# docker export ogn_logbook_rs |xz > $DIR/ogn_logbook_rs.container.tar.xz
docker export ogn_logbook_rs |bzip2 > $DIR/ogn_logbook_rs.container.tar.bz2
