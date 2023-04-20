#== First stage: this is the build stage for Substrate. Here we create the binary.
FROM eightfish-m2-build as builder

#== Second stage: 
FROM docker.io/library/ubuntu:20.04
LABEL description="EightFish:Subnode"

WORKDIR /eightfish

COPY --from=builder /eightfish/subnode/target/release/eightfish-subnode /usr/local/bin

