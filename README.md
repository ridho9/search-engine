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

## Possible improvement

1. Use embeddings for better, contextual, search
2. Resumable and extendable scraping job
3. Etc

## On The Tech Stacks

### Tantivy vs Vespa

After testing (or atleast trying) both of the given library, I went with tantivy because tantivy is much approachable to me
and I could understand how it works because it is low level enough. Tantivy is also in Rust and I have an interest in Rust 
so I thought this is a good opportunity to learn Rust.

Vespa on the other hand, I could not manage to set it up despite trying a few times from scratch. 
The documentation is not very clear and hard to understand, which is a shame, because feature-wise
it seems that vespa is powerful compared to tantivy which only uses BM25. 
Where with Vespa you could do better matching with not only BM25, but using embedding (like WordNet)
that could give much more intelligent search result.

### Scrapy

I went for scrapy for scraper because I rather familiar with it and Python. With scrapyd it is also easily scalable to handle more load.

The way that the scraper works is that for each page that it scrape, it would be cleaned and have the main body extracted.
After that it would call the tantivy engine to store the document.

By the way that the scraper works now, everytime there is a change to the scraper, all data would have to be cleaned and rescraped.
This could possibly mitigated by storing the scraped raw html in storage.

### Page Cleaning

To clean the page, I mainly use beautifulsoup for general html processing, Trafilatura and Readabilipy to extract the main information of the page.
Trafilatura is used to extract all text from a pages.
Readibilipy is the python wrapper of nodejs library `@mozilla/readiblity` which used by Firefox's reader mode. 
This library can extract the main content of the pages and left out things like navigation, sidebar, footer, etc.

### Page Indexing

On my first iteration, it stored the pages in `jsonl` files, because I thought that it could be consumed by the tantivy-cli later.
But after seeing that it have some overhead, I went with writing a rust server that wraps the tantivy engine that could write and query the docs.

The program itself uses two document collections, `main-index` and `page-index`. 
The `main-index` stores the whole page that Scrapy scraped, and is used for the main search query.
The `page-index` stores the sentences snippet group for each of the pages (so that each pages would generate multiple snippets)
which then later can be queried to find which snippet is the most relevant for the current query.

With the way that the indexer works now, multiple scraper invocation could generate duplicate pages. 
This could be mitigated by adding a layer in the middle that keeps track of the indexed url, either by querying the current index or by using redis.


### Frontend

For frontend is just a normal react page, which I think personally overkill as even a normal html with some ajax would do
but personally is the easiest for me to set up.