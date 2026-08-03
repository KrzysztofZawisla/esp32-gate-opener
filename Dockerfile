# Match the IDF version the crate set is pinned against (esp-idf-hal 0.45 /
# esp-idf-svc 0.51 / esp-idf-sys 0.36 are the official 5.3 line).
FROM espressif/idf:v5.3.5 AS build
ENV IDF_PATH=/opt/esp/idf
ENV IDF_TOOLS_PATH=/opt/esp
ENV ESP_IDF_TOOLS_INSTALL_DIR=fromenv
ENV PATH=/root/.cargo/bin:${PATH}
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
RUN curl -sSf https://raw.githubusercontent.com/esp-rs/espup/main/espup/install.sh | sh && \
    espup install --toolchain-version 1.90.0.0 --targets esp32 --std && \
    rustup default esp
RUN cargo install ldproxy --locked
WORKDIR /app
COPY . .
# espup bundles clippy; it is already present in the toolchain. The firmware
# size budget is enforced separately in CI via scripts/check-size.ts.
RUN bash -c "source \$IDF_PATH/export.sh && cargo build --release && cargo clippy --release -- -D warnings"
FROM scratch AS release
COPY --from=build /app/target/xtensa-esp32-espidf/release/esp32-gate-opener /esp32-gate-opener
