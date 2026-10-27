FROM ubuntu:25.10@sha256:7cc5e35f6567ee8c66d2abb4aab0fd866669e6207c237c3a8f0947a5c7f17092

ARG DIFFYN_VERSION
ARG DIFFYN_REVISION
ARG DIFFYN_PACKAGING_REVISION
LABEL org.opencontainers.image.title="Diffyn" \
      org.opencontainers.image.description="Local communication tools and guidance over MCP stdio" \
      org.opencontainers.image.source="https://github.com/nathcymru/Diffyn" \
      org.opencontainers.image.url="https://nathcymru.github.io/Diffyn/" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="${DIFFYN_VERSION}" \
      org.opencontainers.image.revision="${DIFFYN_REVISION}" \
      io.diffyn.packaging.revision="${DIFFYN_PACKAGING_REVISION}"

COPY --chown=0:0 . /opt/diffyn/
WORKDIR /opt/diffyn
USER 65532:65532
ENTRYPOINT ["/opt/diffyn/diffyn"]
CMD []
