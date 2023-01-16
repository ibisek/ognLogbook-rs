# FROM ubuntu:latest
FROM ubuntu:focal

RUN apt-get update && apt-get install -y libgdal26 gdal-bin && rm -rf /var/lib/apt/lists/*

RUN mkdir /app
RUN mkdir /app/data

WORKDIR /app

# COPY ./target/release/ogn_logbook .
COPY ./target/debug/ogn_logbook .

CMD ["./ogn_logbook"]

# COPY ./dummyLoop.sh .
# CMD ["./dummyLoop.sh"]
