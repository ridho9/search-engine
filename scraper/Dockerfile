FROM python AS build-stage

ARG PIP_NO_CACHE_DIR=1

# RUN pip install poetry
# RUN poetry config virtualenvs.create false

WORKDIR /workdir

# COPY ./pyproject.toml ./poetry.lock ./
# RUN poetry export --without-hashes --format=requirements.txt > requirements.txt

RUN pip install --no-cache-dir scrapyd-client

COPY . .

RUN scrapyd-deploy --build-egg=myproject.egg

# Build the image.

FROM python:alpine

# Install Scrapy dependencies - and any others for your project.

RUN apk --no-cache add --virtual build-dependencies \
    gcc \
    musl-dev \
    libffi-dev \
    libressl-dev \
    libxml2-dev \
    libxslt-dev \
    && pip install --no-cache-dir scrapyd \
    && apk del build-dependencies \
    && apk add \
    libressl \
    libxml2 \
    libxslt
RUN apk --no-cache add nodejs npm

COPY ./requirements.txt requirements.txt
RUN pip install -r ./requirements.txt

RUN cd $(pip show readabilipy | grep Loc | sed 's/Location: //')/readabilipy/javascript && npm install && npm install

# Mount two volumes for configuration and runtime.
WORKDIR /

VOLUME /etc/scrapyd/ /var/lib/scrapyd/

RUN mkdir -p /src/eggs/myproject

COPY --from=build-stage /workdir/myproject.egg /src/eggs/myproject/1.egg

COPY ./scrapyd.conf /etc/scrapyd/

EXPOSE 6800

ENTRYPOINT ["scrapyd", "--pidfile="]