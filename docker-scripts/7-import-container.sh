#!/bin/bash

DIR=/tmp/00/
DATA_DIR=/var/www/ognLogbook/data

# unxz -k $DIR/ogn_logbook_rs.container.tar.xz
bunzip2 -k $DIR/ogn_logbook_rs.container.tar.bz2

# docker import $DIR/ogn_logbook_rs.container.tar ogn_logbook_rs -c 'CMD ["./dummyLoop.sh"]' -c 'WORKDIR /app'
docker import $DIR/ogn_logbook_rs.container.tar ogn_logbook_rs -c 'CMD ["./ogn_logbook"]' -c 'WORKDIR /app'
#-c ENV=PYTHONPATH=$PYTHONPATH:/app:/app/src
