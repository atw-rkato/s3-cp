ARG BASE_IMAGE_TAG
FROM joseluisq/rust-linux-darwin-builder:${BASE_IMAGE_TAG}
ENV CC="o64-clang"
ENV CXX="o64-clang++"
ARG APP_NAME
ARG BUILD_TARGET=x86_64-apple-darwin

WORKDIR /work
COPY Cargo.toml /work/Cargo.toml
COPY src /work/src
RUN cargo build --release -j 6 --target="${BUILD_TARGET}"
RUN mkdir -p "/work/dist" \
  && cp "/work/target/${BUILD_TARGET}/release/${APP_NAME}" "/work/dist/${APP_NAME}" \
  && cargo clean
WORKDIR /dist
ENV APP_NAME=${APP_NAME}
ENTRYPOINT cp "/work/dist/${APP_NAME}" "/dist/${APP_NAME}"
