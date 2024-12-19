# Mini Doc Search Engine

Deployment at `https://search.ridho.dev/` (if I still host it)

## Setup

### Prerequisites

1. Docker
2. Python 3 with `requests` installed


### Running The Service

`docker compose up -d`

Locally, the web will be accessible on `localhost:5000`.

### Running crawler

`python3 ./scripts/run_list.py`


## Architecture

### Engine Backend

Built using Rust and Tantivy. Used to index documents and query it.

### Scraper

Built using Python and Scrapy. Deployed with Scrapyd. Used to crawl a given domain and forward the page to the engine backend to be indexed.

### Frontend

Built using React.


## Challenges

1. Learning about the libraries
2. Scraping relevant part of the page
   1. Use trafilatura
   2. Use `readabilipy`
3. Picking relevant sentence from page
   1. Load sentences of the result as document to score with tf-idf/bm25?
   2. Filter on frontend with winkjs or backend with tantivy?
      1. On frontend would be simpler but the query format is different from tantivy
      2. On backend would be faster and uses the same query format
         1. Precompute index per page also
            1. Good latency, but more storage will be used
         2. Or on runtime
            1. 9~20x latency compared to base

## On Proxy

When using proxy, we will have a list of proxy that could be used to send the requests.
For some paid proxy provider, either they have their own list, or they have the endpoint that can be used to query a page.
Sometime they have features like captcha detection and even rendering js pages.
