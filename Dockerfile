FROM debian:buster as dependencies

## All dependencies so that we can install
## - Python
## - Poetry
## - PyEnv

RUN apt-get update \ 
        && apt-get install -y --no-install-recommends make build-essential libssl-dev zlib1g-dev libbz2-dev libreadline-dev libsqlite3-dev wget ca-certificates curl llvm libncurses5-dev xz-utils tk-dev libxml2-dev libxmlsec1-dev libffi-dev liblzma-dev mecab-ipadic-utf8 git

# Add new user with ID 1001
# TODO: Reduce scope
RUN useradd -rm -d /home/debian -s /bin/bash -g root -G sudo -u 1001 debian
USER debian
WORKDIR /home/debian

ENV PYTHON_VERSION 3.10.7
ENV PYENV_ROOT /home/debian/.pyenv
ENV POETRY_ROOT /home/debian/.local/bin
ENV PATH $PYENV_ROOT/shims:$PYENV_ROOT/bin:$POETRY_ROOT:$PATH

# Install pyenv
RUN set -ex \
    && curl https://pyenv.run | bash \
    && pyenv update \
    && pyenv install $PYTHON_VERSION \
    && pyenv global $PYTHON_VERSION \
    && pyenv rehash

# TODO: Re-enable once dev is done!
# Install previous Python versions up to Python3.7
# RUN pyenv install 3.9.14
# RUN pyenv install 3.8.13
# RUN pyenv install 3.7.13

# Install Poetry in the latest version
RUN curl -sSL https://install.python-poetry.org | python3.10 -

FROM rust:1.64.0-buster as build 
RUN cargo install cargo-build-dependencies
RUN USER=root cargo new --bin containerless
WORKDIR /containerless
COPY Cargo.toml Cargo.lock ./
RUN cargo build-dependencies --release 
COPY ./src ./src 
RUN cargo build --release 

FROM dependencies as app

COPY --from=build /containerless/target/release/containerless .

CMD ["./containerless"]