FROM ubuntu:26.04@sha256:6a06111c3b0a8600feadb270aec771dc239bf407b25625f93f44ab00e11f4d3c

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
