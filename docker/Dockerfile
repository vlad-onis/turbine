# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the entire source code to the container
COPY . .

RUN cargo build --release

EXPOSE 12345

CMD ["./target/release/turbine"]
