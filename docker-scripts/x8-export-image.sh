#!/bin/bash

DIR=/tmp/00/

id=`docker images |grep ogn_logbook_rs| cut -d' ' -f 11`

docker save $id |xz > $DIR/ogn_logbook_rs.image.tar.xz

