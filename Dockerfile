FROM rust:latest

RUN useradd -m -s /bin/bash bobuser

WORKDIR /app

COPY . .

RUN cargo build

USER bobuser

RUN mkdir -p ~/.config/bob && echo '{"version_sync_file_location": "/home/bobuser/.config/nvim/nvim.version"}' > ~/.config/bob/config.json
RUN mkdir -p ~/.config/nvim

USER root

RUN cp target/debug/bob /usr/local/bin/

USER bobuser
ENV USER=bobuser

CMD ["echo", "Use 'bob' to start the project"]
