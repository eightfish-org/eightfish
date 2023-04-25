#== First stage: this is the build stage for Substrate. Here we create the binary.
FROM eightfish-m2-build as builder

#== Second stage: 
FROM docker.io/library/ubuntu:20.04
LABEL description="EightFish:subxtproxy"

WORKDIR /eightfish

COPY --from=builder /eightfish/subxtproxy/target/release/subxtproxy /usr/local/bin
COPY --from=builder /eightfish/subxtproxy/metadata.scale /eightfish/
