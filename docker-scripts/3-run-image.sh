#!/bin/bash

CONTAINER_NAME='ogn_logbook_rs'
IMAGE="$CONTAINER_NAME:0.1"

DATA_DIR='/var/www/ognLogbook-data'

docker container kill $CONTAINER_NAME
docker rm $CONTAINER_NAME

docker run --detach --name $CONTAINER_NAME \
    --env-file=../.env \
    -v $DATA_DIR:/app/data:Z \
    $IMAGE

docker ps -a
