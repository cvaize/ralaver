FROM rust:1.82.0 as build

RUN mkdir -p /app
WORKDIR /app
#COPY . /app

#RUN cargo install --debug --path .
RUN cargo install diesel_cli

# Install Node.js
ENV NODE_VERSION=22.13.1
RUN apt install -y curl
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"
RUN node --version
RUN npm --version

ENTRYPOINT [ "sleep", "infinity" ]
