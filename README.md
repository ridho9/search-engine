# Mini Doc Search Engine

## Setup

### Prerequisites

1. Docker
2. Python 3 with `requests` installed


### Running The Service

`docker compose up -d`

### Running crawler

`python3 ./scripts/run_list.py`


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
         2. Or on runtime
            1. 9~20x latency compared to base 
4. 

